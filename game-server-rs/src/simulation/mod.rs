use std::sync::Arc;
use std::time::{Duration, Instant};

use elura::gameplay::simulation::{FixedStepClock, SimulationConfig, SimulationStep};
use rand::Rng;
use tracing::warn;

use crate::gameplay::entity_replication::make_versioned_state;
use crate::gameplay::netcode::CombatSnapshot;
use crate::reconnect::ReconnectManager;
use crate::replication::{build_self_vitals, build_character_packet, build_npc_packet, build_delete_character_packet};
use crate::world::{EntityId, GameWorld};
use crate::ServerMetrics;

const TICK_RATE: u64 = 60;

pub fn run_game_loop(world: Arc<GameWorld>, reconnect_mgr: Arc<ReconnectManager>, metrics: Arc<ServerMetrics>, rt_handle: tokio::runtime::Handle) {
    let mut config = SimulationConfig::default();
    config.step = Duration::from_nanos(1_000_000_000 / TICK_RATE);
    config.max_steps_per_update = 10;
    config.max_accumulated_time = Duration::from_millis(500);

    let mut clock = FixedStepClock::new(config).expect("valid simulation config");

    tracing::info!("Game loop started at {} TPS (Elura FixedStepClock)", TICK_RATE);

    let mut last_instant = Instant::now();

    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(last_instant);
        last_instant = now;

        let world_ref = &world;
        let reconnect_ref = &reconnect_mgr;
        let metrics_ref = &metrics;
        let rt_ref = &rt_handle;
        let result = clock.advance::<std::convert::Infallible, _>(elapsed, |step: SimulationStep| {
            let tick = step.tick as u64;
            metrics_ref.current_tick.store(tick, std::sync::atomic::Ordering::Relaxed);
            let tick_start = Instant::now();

            let tick_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                process_tick(world_ref, reconnect_ref, tick, rt_ref);
            }));
            if let Err(panic_info) = tick_result {
                let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                tracing::error!("GAME LOOP PANIC at tick {}: {}", tick, msg);
            }

            let tick_us = tick_start.elapsed().as_micros() as u64;
            metrics_ref.tick_time_sum_us.fetch_add(tick_us, std::sync::atomic::Ordering::Relaxed);
            metrics_ref.tick_time_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            metrics_ref.tick_time_max_us.fetch_max(tick_us, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        });

        match result {
            Ok(report) => {
                if report.dropped_time > Duration::ZERO {
                    warn!(
                        "Game loop dropped {}ms at tick {}",
                        report.dropped_time.as_millis(),
                        clock.tick()
                    );
                }
            }
            Err(e) => match e {},
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}

fn process_tick(world: &GameWorld, reconnect_mgr: &ReconnectManager, tick: u64, rt: &tokio::runtime::Handle) {
    // ── Per-tick work (always runs) ─────────────────────────────
    record_combat_snapshots(world, tick);
    update_input_receiver_ticks(world, tick);
    process_buff_ticks(world);
    process_cc_expiry(world, tick);
    process_dead_world_transitions(world);
    process_summon_expiry(world);

    if tick.is_multiple_of(3) {
        process_entity_replication(world, tick);
    }

    // ── Configurable intervals (read once per tick, Relaxed is fine) ──
    let regen_interval = world.runtime_timings.regen_ticks.load(std::sync::atomic::Ordering::Relaxed);
    if regen_interval > 0 && tick.is_multiple_of(regen_interval) {
        process_hp_mana_regen(world);
        process_jail_release(world);
        reset_outbound_pressure(world);
    }

    let npc_ai_interval = world.runtime_timings.npc_ai_ticks.load(std::sync::atomic::Ordering::Relaxed);
    if npc_ai_interval > 0 && tick.is_multiple_of(npc_ai_interval) {
        process_npc_ai(world, tick);
    }

    if tick.is_multiple_of(30) {
        process_admin_bot_heal(world);
    }

    if tick.is_multiple_of(60 * 30) {
        process_npc_respawn(world);
    }

    if tick.is_multiple_of(60 * 60) {
        process_market_expiry(world, rt);
        reconnect_mgr.evict_expired();
    }

    if tick.is_multiple_of(60 * 5) {
        process_idle_log(world, tick);
    }

    if tick.is_multiple_of(60 * 30) {
        broadcast_live_leaderboard(world);
    }

    if tick.is_multiple_of(60 * 60) {
        process_floor_item_cleanup(world);
    }

    if tick.is_multiple_of(60 * 5) {
        process_duplicate_account_policy(world);
    }

    // Territory capture tick — advance capture for clans with members present
    if tick.is_multiple_of(60 * 10) {
        process_territory_capture(world);
    }

    // SQLite backup every 30 minutes
    if tick.is_multiple_of(60 * 60 * 30) && tick > 0 {
        let db = world.db.clone();
        rt.spawn(async move {
            process_sqlite_backup(&db).await;
        });
    }
}

/// Update the current server tick on all input receivers so they validate
/// incoming packets against the correct tick window.
fn update_input_receiver_ticks(world: &GameWorld, tick: u64) {
    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        for entry in scene.input_receivers.iter() {
            if let Ok(mut receiver) = entry.value().try_lock() {
                receiver.set_tick(tick);
            }
        }
    }
}

/// Reconcile each observer's visible entity set using Elura's ReplicationSender.
/// Generates efficient spawn/despawn/keyframe batches and sends them via personal_tx.
fn process_entity_replication(world: &GameWorld, tick: u64) {
    use elura::gameplay::replication::ReplicationEvent;

    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        if scene.players.is_empty() {
            continue;
        }

        let observer_ids: Vec<u32> = scene.replicators.iter().map(|e| *e.key()).collect();

        for observer_id in observer_ids {
            let observer_pos = match scene.players.get(&observer_id) {
                Some(p) => p.pos.clone(),
                None => continue,
            };

            let nearby = scene.entities_in_range(&observer_pos);

            let mut visible = Vec::new();
            for &eid in &nearby {
                if eid == observer_id { continue; }

                if let Some(player) = scene.players.get(&eid) {
                    if !player.invisible {
                        let state_data = build_character_packet(&player);
                        let version = tick;
                        visible.push((eid, make_versioned_state(version, state_data)));
                    }
                } else if let Some(npc) = scene.npcs.get(&eid)
                    && !npc.dead {
                        let state_data = build_npc_packet(&npc, &world.gd());
                        let version = tick;
                        visible.push((eid, make_versioned_state(version, state_data)));
                    }
            }

            let replicator_entry = match scene.replicators.get(&observer_id) {
                Some(entry) => entry,
                None => continue,
            };
            let mut replicator = match replicator_entry.try_lock() {
                Ok(r) => r,
                Err(_) => continue,
            };

            let batches_queued = replicator.reconcile(tick, visible);
            if batches_queued == 0 {
                continue;
            }

            let packet = replicator.build_packet();
            for batch in &packet.batches {
                for event in &batch.events {
                    match event {
                        ReplicationEvent::Spawn { entity, state, .. } => {
                            if !replicator.is_broadcast_announced(*entity) {
                                scene.send_to_player(observer_id, state.to_vec());
                                send_entity_visuals(scene, world, observer_id, *entity);
                            }
                        }
                        ReplicationEvent::Despawn { entity } => {
                            let pkt = build_delete_character_packet(*entity);
                            scene.send_to_player(observer_id, pkt);
                        }
                        ReplicationEvent::Keyframe { entity: _, state, .. } => {
                            scene.send_to_player(observer_id, state.to_vec());
                        }
                        ReplicationEvent::Update { entity: _, delta, .. } => {
                            scene.send_to_player(observer_id, delta.to_vec());
                        }
                    }
                }

                let ack = elura::gameplay::replication::ReplicationAck {
                    acknowledged_sequence: batch.sequence,
                    applied_tick: batch.tick,
                };
                let _ = replicator.acknowledge(ack);
            }
        }
    }
}

/// Send equipment visuals and color for a spawned player entity to an observer.
fn send_entity_visuals(scene: &crate::world::Scene, _world: &GameWorld, observer_id: u32, entity_id: u32) {
    use crate::gateway::packets::{build_act_color_name, build_change_equipment, get_name_color};
    use openao_protocol::opcodes::client_packet_id;

    if let Some(player) = scene.players.get(&entity_id) {
        let color = get_name_color(player.criminal, &player.faction, false);
        scene.send_to_player(observer_id, build_act_color_name(entity_id, color));
        if player.id_head > 0 {
            scene.send_to_player(observer_id, build_change_equipment(client_packet_id::CHANGE_ROPA, entity_id, player.id_head));
        }
        if player.id_body > 0 {
            scene.send_to_player(observer_id, build_change_equipment(client_packet_id::CHANGE_BODY, entity_id, player.id_body));
        }
        if player.id_weapon > 0 {
            scene.send_to_player(observer_id, build_change_equipment(client_packet_id::CHANGE_WEAPON, entity_id, player.id_weapon));
        }
        if player.id_helmet > 0 {
            scene.send_to_player(observer_id, build_change_equipment(client_packet_id::CHANGE_HELMET, entity_id, player.id_helmet));
        }
        if player.id_shield > 0 {
            scene.send_to_player(observer_id, build_change_equipment(client_packet_id::CHANGE_SHIELD, entity_id, player.id_shield));
        }
    }
}

fn process_jail_release(world: &GameWorld) {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        for mut player_ref in scene.players.iter_mut() {
            let player = player_ref.value_mut();
            if player.jail_until_ms > 0 && now_ms >= player.jail_until_ms {
                player.jail_until_ms = 0;
                scene.send_to_player(player.id, crate::gateway::packets::build_console_message("Has sido liberado de la cárcel."));
            }
        }
    }
}

fn process_buff_ticks(world: &GameWorld) {
    use crate::gameplay::buffs::BuffType;

    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        for mut player_ref in scene.players.iter_mut() {
            let player = player_ref.value_mut();
            let expired = player.buffs.tick();
            for buff_type in expired {
                let msg = match buff_type {
                    BuffType::Strength => "Tu buff de Fuerza ha expirado.",
                    BuffType::Agility => "Tu buff de Agilidad ha expirado.",
                };
                scene.send_to_player(player.id, crate::gateway::packets::build_console_message(msg));
            }
        }
    }
}

fn process_cc_expiry(world: &GameWorld, current_tick: u64) {
    let now_ms = world.uptime_ms();
    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();

        let mut hidden_expired: Vec<u32> = Vec::new();

        for mut player_ref in scene.players.iter_mut() {
            let player = player_ref.value_mut();

            if player.paralizado && now_ms >= player.paralizado_until_ms {
                player.paralizado = false;
                player.paralizado_until_ms = 0;
                scene.send_to_player(player.id, crate::gateway::packets::build_console_message("Ya no estás paralizado."));
            }

            if player.inmovilizado && now_ms >= player.inmovilizado_until_ms {
                player.inmovilizado = false;
                player.inmovilizado_until_ms = 0;
                let pos = player.pos.clone();
                scene.send_to_player(player.id, crate::gateway::packets::build_inmo(pos.x, pos.y, 0));
                scene.send_to_player(player.id, crate::gateway::packets::build_console_message("Ya no estás inmovilizado."));
            }

            if player.invisible_spell && now_ms >= player.invisible_spell_until_ms {
                player.invisible_spell = false;
                player.invisible_spell_until_ms = 0;
                scene.send_to_player(player.id, crate::gateway::packets::build_console_message("Tu invisibilidad se desvaneció."));
            }

            if player.hidden_skill && player.hidden_skill_expire_tick > 0 && current_tick >= player.hidden_skill_expire_tick {
                hidden_expired.push(player.id);
            }
        }

        for eid in hidden_expired {
            crate::gateway::inventory::stop_hidden_skill(eid, &scene, current_tick, 0);
        }
    }
}

fn process_summon_expiry(world: &GameWorld) {
    let now_ms = world.uptime_ms();
    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        let mut expired: Vec<(EntityId, crate::world::Position)> = Vec::new();
        for entry in scene.npcs.iter() {
            let npc = entry.value();
            if npc.summoned_by.is_some() && npc.summon_expires_at_ms > 0 && now_ms >= npc.summon_expires_at_ms {
                expired.push((*entry.key(), npc.pos.clone()));
            }
        }
        for (npc_id, pos) in &expired {
            scene.npcs.remove(npc_id);
            scene.aoi_remove(*npc_id);
            let del = crate::replication::build_delete_character_packet(*npc_id);
            scene.broadcast_in_range(0, pos, del);
        }
        if !expired.is_empty() {
            let expired_ids: std::collections::HashSet<EntityId> = expired.iter().map(|(id, _)| *id).collect();
            for mut p in scene.players.iter_mut() {
                p.summons.retain(|sid| !expired_ids.contains(sid));
            }
        }
    }
}

fn process_admin_bot_heal(world: &GameWorld) {
    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        let bot_ids: Vec<EntityId> = scene.npcs.iter()
            .filter(|e| e.value().admin_bot_owner.is_some() && !e.value().dead && e.value().hp < e.value().max_hp)
            .map(|e| *e.key())
            .collect();
        for npc_id in bot_ids {
            let Some(mut npc) = scene.npcs.get_mut(&npc_id) else { continue; };
            let heal = (npc.max_hp as f64 * 0.05) as i32;
            npc.hp = (npc.hp + heal).min(npc.max_hp);
            let hp = npc.hp;
            let max_hp = npc.max_hp;
            let pos = npc.pos.clone();
            drop(npc);
            let pkt = crate::replication::build_entity_vitals_delta(npc_id, hp, max_hp, 0, 0);
            scene.broadcast_in_range(0, &pos, pkt);
        }
    }
}

fn process_dead_world_transitions(world: &GameWorld) {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        for mut player_ref in scene.players.iter_mut() {
            let player = player_ref.value_mut();
            if player.dead
                && !player.dead_world_active
                && player.dead_world_transition_at_ms > 0
                && now_ms >= player.dead_world_transition_at_ms
            {
                player.dead_world_active = true;
                player.dead_world_transition_at_ms = 0;
            }
        }
    }
}

fn record_combat_snapshots(world: &GameWorld, tick: u64) {
    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        if scene.players.is_empty() && scene.npcs.is_empty() {
            continue;
        }

        let mut snapshots = Vec::new();

        for entry in scene.players.iter() {
            let p = entry.value();
            snapshots.push(CombatSnapshot {
                entity_id: p.id,
                pos: p.pos.clone(),
                hp: p.hp,
                dead: p.dead,
            });
        }

        for entry in scene.npcs.iter() {
            let n = entry.value();
            if n.max_hp > 0 {
                snapshots.push(CombatSnapshot {
                    entity_id: n.id,
                    pos: n.pos.clone(),
                    hp: n.hp,
                    dead: n.dead,
                });
            }
        }

        if let Ok(mut history) = scene.lag_history.try_lock() {
            history.record_tick(tick, snapshots);
        }
    }
}

fn process_hp_mana_regen(world: &GameWorld) {
    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        for mut player_ref in scene.players.iter_mut() {
            let player = player_ref.value_mut();
            if player.dead {
                continue;
            }

            let mut changed = false;

            let hp_regen = (player.max_hp as f32 * 0.02).max(1.0) as i32;
            if player.hp < player.max_hp {
                player.hp = (player.hp + hp_regen).min(player.max_hp);
                changed = true;
            }

            let mana_regen = (player.max_mana as f32 * 0.03).max(1.0) as i32;
            if player.mana < player.max_mana {
                player.mana = (player.mana + mana_regen).min(player.max_mana);
                changed = true;
            }

            if changed {
                let pkt = build_self_vitals(player.hp, player.max_hp, player.mana, player.max_mana);
                scene.send_to_player(player.id, pkt);
            }
        }
    }
}

fn process_npc_respawn(world: &GameWorld) {
    let current_tick = world.uptime_ms();

    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        let map_id = scene.map_id;

        let dead_npcs: Vec<(u32, i32)> = scene.npcs.iter()
            .filter(|e| e.value().dead)
            .map(|e| (*e.key(), e.value().npc_type))
            .collect();

        for (id, npc_type) in &dead_npcs {
            scene.aoi_remove(*id);
            scene.npcs.remove(id);
            let cd_key = (map_id, *npc_type);
            let gd = world.gd();
            let is_dragon = gd.get_npc(*npc_type).map(|t| t.npc_type == 6).unwrap_or(false);
            let cooldown_ms: u64 = if is_dragon { 3_600_000 } else { 30_000 };
            world.npc_respawn_cooldowns.entry(cd_key).or_insert(current_tick + cooldown_ms);
        }

        let gd = world.gd();
        let Some(spawns) = gd.get_map_spawns(map_id) else {
            continue;
        };

        let alive_types: Vec<i32> = scene.npcs.iter()
            .filter(|e| !e.value().dead)
            .map(|e| e.value().npc_type)
            .collect();

        for spawn in spawns {
            if alive_types.contains(&spawn.npc_index) {
                continue;
            }

            let cd_key = (map_id, spawn.npc_index);
            if let Some(cooldown_until) = world.npc_respawn_cooldowns.get(&cd_key) {
                if current_tick < *cooldown_until {
                    continue;
                }
            }
            world.npc_respawn_cooldowns.remove(&cd_key);

            let Some(template) = gd.get_npc(spawn.npc_index) else {
                continue;
            };

            let id = world.next_id();
            let pos = crate::world::Position { map: map_id, x: spawn.x, y: spawn.y };
            scene.aoi_insert(id, &pos);
            let movement = spawn.movement.unwrap_or(template.movement);
            let npc_spells: Vec<crate::world::NpcSpellSlot> = template.spells.iter()
                .map(|s| crate::world::NpcSpellSlot { spell_id: s.id_spell })
                .collect();
            scene.npcs.insert(id, crate::world::NpcState {
                id,
                npc_type: spawn.npc_index,
                pos,
                heading: 3,
                hp: template.max_hp,
                max_hp: template.max_hp,
                min_hit: template.min_hit,
                max_hit: template.max_hit,
                defense: template.def,
                exp_reward: template.exp,
                movement,
                dead: template.max_hp <= 0,
                paralizado: false,
                inmovilizado: false,
                cc_expire_tick: 0,
                aggro_target: None,
                spells: npc_spells,
                spell_cast_interval_ms: template.spell_cast_interval_ms.unwrap_or(2000),
                last_spell_cast_at: 0,
                spell_range: template.spell_range.unwrap_or(8),
                magic_def: template.magic_def,
                magic_resistance: template.magic_resistance,
                summoned_by: None,
                summon_expires_at_ms: 0,
                admin_bot_owner: None,
            });

            if let Some(npc_ref) = scene.npcs.get(&id) {
                let pkt = crate::replication::build_npc_packet(&npc_ref, &world.gd());
                let npc_pos = npc_ref.pos.clone();
                drop(npc_ref);
                scene.broadcast_in_range(0, &npc_pos, pkt);
            }
        }
    }
}

fn process_npc_ai(world: &GameWorld, tick: u64) {
    let mut rng = rand::rng();

    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();

        let player_positions: Vec<(u32, i32, i32, bool, bool)> = scene.players.iter()
            .map(|p| (p.id, p.pos.x, p.pos.y, p.dead, p.invisible || p.hidden_skill || p.invisible_spell))
            .collect();

        if player_positions.is_empty() {
            continue;
        }

        let npc_ids: Vec<u32> = scene.npcs.iter().map(|e| *e.key()).collect();

        for npc_id in npc_ids {
            let mut npc = match scene.npcs.get_mut(&npc_id) {
                Some(n) => n,
                None => continue,
            };
            if npc.dead { continue; }
            if npc.max_hp <= 0 || npc.max_hit <= 0 { continue; }

            if npc.cc_expire_tick > 0 && tick >= npc.cc_expire_tick {
                npc.paralizado = false;
                npc.inmovilizado = false;
                npc.cc_expire_tick = 0;
            }

            if npc.paralizado { continue; }

            let npc_movement = npc.movement;
            let npc_inmovilizado = npc.inmovilizado;
            if npc_movement <= 1 && !npc_inmovilizado { continue; }

            let attack_range = 5;

            let aggro_id = npc.aggro_target;
            let mut closest_player: Option<(u32, i32)> = None;

            if let Some(atgt) = aggro_id {
                if let Some(&(_, px, py, pdead, phidden)) = player_positions.iter().find(|(id, _, _, _, _)| *id == atgt) {
                    if !pdead && !phidden {
                        let dist = (npc.pos.x - px).abs() + (npc.pos.y - py).abs();
                        if dist <= attack_range {
                            closest_player = Some((atgt, dist));
                        }
                    }
                }
            }

            if closest_player.is_none() {
                let mut best_score = i32::MAX;
                let npc_x = npc.pos.x;
                let npc_y = npc.pos.y;
                let current_target = npc.aggro_target;
                for &(pid, px, py, pdead, phidden) in &player_positions {
                    if pdead || phidden { continue; }
                    let dist = (npc_x - px).abs() + (npc_y - py).abs();
                    if dist > attack_range { continue; }
                    let is_adjacent = if dist == 1 { 1 } else { 0 };
                    let is_aggro = if aggro_id == Some(pid) { 1 } else { 0 };
                    let is_current = if current_target == Some(pid) { 1 } else { 0 };
                    let escape_tiles = count_escape_tiles(px, py, world, npc.pos.map);
                    let attack_tiles = count_attack_tiles(px, py, npc_x, npc_y, world, npc.pos.map);
                    let score = dist * 3
                        + escape_tiles * 2
                        - attack_tiles * 4
                        - is_adjacent * 6
                        - is_current * 8
                        - is_aggro * 14;
                    if score < best_score {
                        best_score = score;
                        closest_player = Some((pid, dist));
                    }
                }
            }

            if let Some((target_pid, dist)) = closest_player {
                let npc_spells_available = !npc.spells.is_empty();
                let npc_spell_range = npc.spell_range;
                let npc_spell_interval = npc.spell_cast_interval_ms;
                let npc_last_cast = npc.last_spell_cast_at;
                let npc_hp_ratio = if npc.max_hp > 0 { npc.hp as f64 / npc.max_hp as f64 } else { 1.0 };
                let npc_spells_clone: Vec<crate::world::NpcSpellSlot> = npc.spells.clone();
                let npc_npc_hp = npc.hp;
                let npc_npc_max_hp = npc.max_hp;

                if dist <= 1 {
                    let mut cast_spell_instead = false;

                    if npc_spells_available {
                        let now_ms = world.uptime_ms();
                        if now_ms >= npc_last_cast + npc_spell_interval {
                            cast_spell_instead = try_npc_cast_spell(
                                world, &scene, npc_id, target_pid, &npc_spells_clone,
                                npc_npc_hp, npc_npc_max_hp, npc_hp_ratio,
                                npc_spell_range, dist, &mut rng,
                            );
                            if cast_spell_instead {
                                if let Some(mut n) = scene.npcs.get_mut(&npc_id) {
                                    n.last_spell_cast_at = now_ms;
                                }
                            }
                        }
                    }

                    if !cast_spell_instead {
                        let damage = if npc.min_hit < npc.max_hit {
                            rng.random_range(npc.min_hit..=npc.max_hit)
                        } else {
                            npc.max_hit.max(1)
                        };

                        let npc_name_for_msg: String = world.gd().get_npc(npc.npc_type)
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| "NPC".to_string());
                        drop(npc);

                        apply_npc_melee_damage(world, &scene, npc_id, target_pid, damage, &npc_name_for_msg);
                    } else {
                        drop(npc);
                    }
                } else if npc_spells_available && dist <= npc_spell_range {
                    let now_ms = world.uptime_ms();
                    if now_ms >= npc_last_cast + npc_spell_interval {
                        let did_cast = try_npc_cast_spell(
                            world, &scene, npc_id, target_pid, &npc_spells_clone,
                            npc_npc_hp, npc_npc_max_hp, npc_hp_ratio,
                            npc_spell_range, dist, &mut rng,
                        );
                        if did_cast {
                            if let Some(mut n) = scene.npcs.get_mut(&npc_id) {
                                n.last_spell_cast_at = now_ms;
                            }
                        }
                    }
                    drop(npc);
                    if !npc_inmovilizado {
                        try_npc_move_towards(world, &scene, npc_id, target_pid, &player_positions, tick);
                    }
                } else if !npc_inmovilizado {
                    drop(npc);
                    try_npc_move_towards(world, &scene, npc_id, target_pid, &player_positions, tick);
                }
            } else if !npc_inmovilizado && rng.random_range(0..10) < 3 {
                let heading: u8 = match rng.random_range(0..4) {
                    0 => 1, // up
                    1 => 2, // down
                    2 => 3, // right
                    _ => 4, // left
                };
                let (dx, dy) = crate::gateway::packets::heading_to_delta(heading);
                let map = npc.pos.map;
                let (map_w, map_h) = world.gd().get_map_bounds(map);
                let new_x = (npc.pos.x + dx).clamp(1, map_w);
                let new_y = (npc.pos.y + dy).clamp(1, map_h);
                if world.gd().is_blocked_tile(map, new_x, new_y) {
                    drop(npc);
                } else {
                    npc.pos.x = new_x;
                    npc.pos.y = new_y;
                    drop(npc);
                    let new_pos = crate::world::Position { map, x: new_x, y: new_y };
                    scene.aoi_move(npc_id, &new_pos);
                    let server_tick = tick as u16;
                    let move_pkt = crate::replication::build_move_entity_packet_with_tick(
                        npc_id, new_x, new_y, heading, server_tick,
                    );
                    scene.broadcast_in_range(0, &new_pos, move_pkt);
                }
            }
        }
    }
}

fn process_market_expiry(world: &GameWorld, rt: &tokio::runtime::Handle) {
    let db = world.db.clone();
    rt.spawn(async move {
        match db.expire_market_listings().await {
            Ok(count) if count > 0 => {
                tracing::info!("Expired {} market listing(s), items returned to sellers", count);
            }
            Err(e) => {
                tracing::warn!("Market expiry failed: {e}");
            }
            _ => {}
        }
    });
}

fn reset_outbound_pressure(world: &GameWorld) {
    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        scene.reset_outbound_pressure();
        for rep_entry in scene.replicators.iter() {
            if let Ok(mut rep) = rep_entry.value().try_lock() {
                rep.reset_broadcast_announced();
            }
        }
    }
}

fn process_idle_log(world: &GameWorld, tick: u64) {
    let total_players: usize = world.scenes.iter().map(|s| s.players.len()).sum();
    if total_players > 0 {
        tracing::debug!(
            "Tick {}: {} player(s) across {} scene(s)",
            tick,
            total_players,
            world.scenes.len()
        );
    }
}

const FLOOR_ITEM_LIFETIME_MS: u64 = 5 * 60 * 1000;

fn process_floor_item_cleanup(world: &GameWorld) {
    let now_ms = world.uptime_ms();
    let mut total_removed = 0u32;
    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        let expired_keys: Vec<(i32, i32)> = scene.ground_items.iter()
            .filter(|entry| now_ms.saturating_sub(entry.value().dropped_at_ms) > FLOOR_ITEM_LIFETIME_MS)
            .map(|entry| *entry.key())
            .collect();

        for key in &expired_keys {
            if let Some((_, item)) = scene.ground_items.remove(key) {
                let del_pkt = crate::replication::build_delete_ground_item(item.x, item.y);
                let pos = crate::world::Position { map: *scene_ref.key(), x: item.x, y: item.y };
                scene.broadcast_in_range(0, &pos, del_pkt);
                total_removed += 1;
            }
        }
    }
    if total_removed > 0 {
        tracing::debug!("Floor cleanup: removed {} expired ground items", total_removed);
    }
}

fn broadcast_live_leaderboard(world: &GameWorld) {
    use crate::gateway::packets::build_console_message;

    let mut entries: Vec<(String, i32, i32)> = Vec::new();
    for scene in world.scenes.iter() {
        for player in scene.players.iter() {
            entries.push((player.name.clone(), player.level, player.gold));
        }
    }

    if entries.is_empty() {
        return;
    }

    entries.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
    entries.truncate(5);

    let mut msg = String::from("§ Top 5 Online: ");
    for (i, (name, level, _gold)) in entries.iter().enumerate() {
        if i > 0 { msg.push_str(" | "); }
        msg.push_str(&format!("{}. {} Nv.{}", i + 1, name, level));
    }

    let pkt = build_console_message(&msg);
    for scene in world.scenes.iter() {
        for tx in scene.personal_tx.iter() {
            let _ = tx.value().send(pkt.clone());
        }
    }
}

fn process_duplicate_account_policy(world: &GameWorld) {
    let penalized = world.get_duplicate_account_penalized_entities();
    if penalized.is_empty() {
        return;
    }

    let msg = crate::gateway::packets::build_console_message(
        "Tu cuenta tiene otra sesión activa trabajando. Esta sesión está penalizada."
    );
    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        for eid in &penalized {
            if let Some(tx) = scene.personal_tx.get(eid) {
                let _ = tx.value().send(msg.clone());
            }
        }
    }
}

async fn process_sqlite_backup(db: &crate::persistence::Database) {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = format!("openao_backup_{}.db", timestamp);
    let query = format!("VACUUM INTO '{}'", backup_path);

    match sqlx::query(&query).execute(db.pool()).await {
        Ok(_) => {
            tracing::info!("SQLite backup created: {backup_path}");
        }
        Err(e) => {
            tracing::warn!("SQLite backup failed: {e}");
        }
    }
}

fn apply_npc_melee_damage(
    world: &GameWorld,
    scene: &crate::world::Scene,
    _npc_id: EntityId,
    target_pid: EntityId,
    damage: i32,
    npc_name: &str,
) {
    if let Some(mut player) = scene.players.get_mut(&target_pid) {
        player.hp = (player.hp - damage).max(0);
        let dead = player.hp <= 0;
        if dead {
            player.dead = true;
            player.dead_world_active = false;
            let now_ms = world.uptime_ms();
            player.dead_world_transition_at_ms = now_ms + crate::gameplay::combat_formulas::DEAD_WORLD_DELAY_MS;
        }
        let player_pos = player.pos.clone();
        let (hp, max_hp, mana, max_mana) = (player.hp, player.max_hp, player.mana, player.max_mana);
        drop(player);

        scene.send_to_player(target_pid, build_self_vitals(hp, max_hp, mana, max_mana));
        let entity_vitals = crate::replication::build_entity_vitals_delta(target_pid, hp, max_hp, mana, max_mana);
        scene.broadcast_in_range(0, &player_pos, entity_vitals);

        let hit_msg = format!("{} te golpea por {} de daño", npc_name, damage);
        scene.send_to_player(target_pid, crate::gateway::packets::build_console_message(&hit_msg));

        if dead {
            let (dh, db, dhel, dw, ds) = scene.players.get(&target_pid)
                .map(|p| (p.id_head, p.id_body, p.id_helmet, p.id_weapon, p.id_shield))
                .unwrap_or((0, 0, 0, 0, 0));
            let death_pkt = crate::gateway::packets::build_put_body_and_head_dead(
                target_pid, dh, db, dhel, dw, ds,
            );
            scene.broadcast_in_range(0, &player_pos, death_pkt);

            let death_msg = format!("{} te ha matado", npc_name);
            scene.send_to_player(target_pid, crate::gateway::packets::build_console_message(&death_msg));
        }
    }
}

fn try_npc_move_towards(
    world: &GameWorld,
    scene: &crate::world::Scene,
    npc_id: EntityId,
    target_pid: EntityId,
    player_positions: &[(EntityId, i32, i32, bool, bool)],
    tick: u64,
) {
    let Some(mut npc) = scene.npcs.get_mut(&npc_id) else { return; };

    let target = player_positions.iter()
        .find(|(id, _, _, _, _)| *id == target_pid);
    let Some((_, px, py, _, _)) = target else { return; };

    let raw_dx = (*px - npc.pos.x).signum();
    let raw_dy = (*py - npc.pos.y).signum();

    if raw_dx == 0 && raw_dy == 0 { return; }

    let map = npc.pos.map;
    let (map_w, map_h) = world.gd().get_map_bounds(map);

    // Cardinal movement only: prefer the axis with greater distance, fallback to the other
    let dist_x = (*px - npc.pos.x).abs();
    let dist_y = (*py - npc.pos.y).abs();

    let candidates: [(i32, i32); 2] = if dist_x >= dist_y {
        [(raw_dx, 0), (0, raw_dy)]
    } else {
        [(0, raw_dy), (raw_dx, 0)]
    };

    for (dx, dy) in candidates {
        if dx == 0 && dy == 0 { continue; }
        let new_x = (npc.pos.x + dx).clamp(1, map_w);
        let new_y = (npc.pos.y + dy).clamp(1, map_h);
        if world.gd().is_blocked_tile(map, new_x, new_y) { continue; }

        npc.pos.x = new_x;
        npc.pos.y = new_y;
        drop(npc);

        let new_pos = crate::world::Position { map, x: new_x, y: new_y };
        scene.aoi_move(npc_id, &new_pos);

        let heading: u8 = if dx > 0 { 3 } else if dx < 0 { 4 } else if dy > 0 { 2 } else { 1 };
        let move_pkt = crate::replication::build_move_entity_packet_with_tick(
            npc_id, new_x, new_y, heading, tick as u16,
        );
        scene.broadcast_in_range(0, &new_pos, move_pkt);
        return;
    }
}

/// NPC AI spell casting. Uses `get_spell_data` for consistent damage derivation.
/// If NPC HP < 50% and has a heal spell, self-heals. Otherwise picks a random
/// offensive spell and applies magic-resistance-reduced damage to target.
fn try_npc_cast_spell(
    world: &GameWorld,
    scene: &crate::world::Scene,
    npc_id: EntityId,
    target_pid: EntityId,
    npc_spells: &[crate::world::NpcSpellSlot],
    _npc_hp: i32,
    npc_max_hp: i32,
    hp_ratio: f64,
    _spell_range: i32,
    _dist: i32,
    rng: &mut rand::rngs::ThreadRng,
) -> bool {
    use rand::Rng;

    if npc_spells.is_empty() { return false; }

    let gd = world.gd();

    let mut heal_spell_id: Option<i32> = None;
    let mut offensive_spells: Vec<i32> = Vec::new();

    for slot in npc_spells {
        if let Some(sp) = gd.get_spell(slot.spell_id) {
            if sp.sube_hp > 0 {
                heal_spell_id = Some(slot.spell_id);
            } else {
                let info = crate::replication::get_spell_data(&gd, slot.spell_id as u16);
                if info.max_damage > 0 {
                    offensive_spells.push(slot.spell_id);
                }
            }
        }
    }

    if hp_ratio < 0.5 {
        if let Some(heal_id) = heal_spell_id {
            if let Some(sp) = gd.get_spell(heal_id) {
                let heal_amount = if sp.max_hp > sp.min_hp {
                    rng.random_range(sp.min_hp..=sp.max_hp)
                } else {
                    sp.max_hp.max(1)
                };
                if let Some(mut npc) = scene.npcs.get_mut(&npc_id) {
                    npc.hp = (npc.hp + heal_amount).min(npc_max_hp);
                    let new_hp = npc.hp;
                    drop(npc);

                    let npc_pos = scene.npcs.get(&npc_id).map(|n| n.pos.clone());
                    if let Some(pos) = npc_pos {
                        if sp.fx_grh > 0 {
                            let fx_pkt = crate::replication::build_anim_fx(npc_id, sp.fx_grh);
                            scene.broadcast_in_range(0, &pos, fx_pkt);
                        }
                        let vitals = crate::replication::build_entity_vitals_delta(
                            npc_id, new_hp, npc_max_hp, 0, 0,
                        );
                        scene.broadcast_in_range(0, &pos, vitals);
                    }
                    return true;
                }
            }
        }
    }

    if offensive_spells.is_empty() { return false; }
    let spell_id = offensive_spells[rng.random_range(0..offensive_spells.len())];

    let spell_info = crate::replication::get_spell_data(&gd, spell_id as u16);

    let base_damage = if spell_info.max_damage > spell_info.min_damage {
        rng.random_range(spell_info.min_damage..=spell_info.max_damage)
    } else {
        spell_info.max_damage.max(1)
    };

    let npc_pos = scene.npcs.get(&npc_id).map(|n| n.pos.clone());
    let target_pos = scene.players.get(&target_pid).map(|p| p.pos.clone());

    if let (Some(np), Some(tp)) = (&npc_pos, &target_pos) {
        if spell_info.fx_id > 0 {
            let fx_pkt = crate::replication::build_anim_fx(npc_id, spell_info.fx_id);
            scene.broadcast_in_range(0, np, fx_pkt);
        }
        let proj_pkt = crate::replication::build_spell_projectile(
            np.x, np.y, tp.x, tp.y, spell_id as u16,
        );
        scene.broadcast_in_range(0, np, proj_pkt);
    }

    if let Some(mut player) = scene.players.get_mut(&target_pid) {
        let final_damage = crate::gameplay::combat_formulas::apply_magic_resistance_to_user(
            base_damage, 1, player.level, player.id_clase, 0, 0,
        ).max(1);

        player.hp = (player.hp - final_damage).max(0);
        let dead = player.hp <= 0;
        if dead {
            player.dead = true;
            player.dead_world_active = false;
            player.dead_world_transition_at_ms = world.uptime_ms()
                + crate::gameplay::combat_formulas::DEAD_WORLD_DELAY_MS;
        }
        let player_pos = player.pos.clone();
        let (hp, max_hp, mana, max_mana) = (player.hp, player.max_hp, player.mana, player.max_mana);
        drop(player);

        scene.send_to_player(target_pid, build_self_vitals(hp, max_hp, mana, max_mana));
        let ev = crate::replication::build_entity_vitals_delta(target_pid, hp, max_hp, mana, max_mana);
        scene.broadcast_in_range(0, &player_pos, ev);

        if spell_info.fx_id > 0 {
            let target_fx = crate::replication::build_anim_fx(target_pid, spell_info.fx_id);
            scene.broadcast_in_range(0, &player_pos, target_fx);
        }

        let npc_name = world.gd().get_npc(
            scene.npcs.get(&npc_id).map(|n| n.npc_type).unwrap_or(0),
        ).map(|t| t.name.clone()).unwrap_or_else(|| "NPC".to_string());

        let msg = format!("{} te lanza un hechizo por {} de daño", npc_name, final_damage);
        scene.send_to_player(target_pid, crate::gateway::packets::build_console_message(&msg));

        if dead {
            let (dh, db, dhel, dw, ds) = scene.players.get(&target_pid)
                .map(|p| (p.id_head, p.id_body, p.id_helmet, p.id_weapon, p.id_shield))
                .unwrap_or((0, 0, 0, 0, 0));
            let death_pkt = crate::gateway::packets::build_put_body_and_head_dead(
                target_pid, dh, db, dhel, dw, ds,
            );
            scene.broadcast_in_range(0, &player_pos, death_pkt);
            let death_msg = format!("{} te ha matado con magia", npc_name);
            scene.send_to_player(target_pid, crate::gateway::packets::build_console_message(&death_msg));
        }

        return true;
    }

    false
}

/// Advance territory capture for clans with members on territory maps.
fn process_territory_capture(world: &GameWorld) {
    let mut mgr = match world.territories.lock() {
        Ok(m) => m,
        Err(_) => return,
    };

    let territory_maps: Vec<(i32, i32)> = mgr.territories.iter().map(|(tid, t)| (*tid, t.map_id)).collect();

    for (territory_id, map_id) in territory_maps {
        let scene = match world.scenes.get(&map_id) {
            Some(s) => s,
            None => continue,
        };

        // Find the dominant clan on this map (most members present)
        let mut clan_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for entry in scene.players.iter() {
            if entry.dead { continue; }
            if let Some(ref clan_id) = entry.clan_id {
                if !clan_id.is_empty() {
                    *clan_counts.entry(clan_id.clone()).or_insert(0) += 1;
                }
            }
        }

        if let Some((dominant_clan, _count)) = clan_counts.iter().max_by_key(|(_, c)| *c) {
            let captured = mgr.territories.get_mut(&territory_id)
                .map(|t| t.advance_capture(dominant_clan))
                .unwrap_or(false);
            if captured {
                // Broadcast territory capture to all players on the map
                let msg = format!("¡El clan {} ha capturado el territorio!", dominant_clan);
                let pkt = crate::gateway::packets::build_console_message(&msg);
                for entry in scene.players.iter() {
                    scene.send_to_player(*entry.key(), pkt.clone());
                }
            }
        }
    }
}

/// Count walkable tiles adjacent to a player (how easily they can escape).
fn count_escape_tiles(px: i32, py: i32, world: &GameWorld, map: i32) -> i32 {
    let gd = world.gd();
    let (mw, mh) = gd.get_map_bounds(map);
    let mut count = 0;
    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let nx = px + dx;
        let ny = py + dy;
        if nx >= 1 && nx <= mw && ny >= 1 && ny <= mh && !gd.is_blocked_tile(map, nx, ny) {
            count += 1;
        }
    }
    count
}

/// Count tiles adjacent to the player that the NPC could attack from.
fn count_attack_tiles(px: i32, py: i32, npc_x: i32, npc_y: i32, world: &GameWorld, map: i32) -> i32 {
    let gd = world.gd();
    let (mw, mh) = gd.get_map_bounds(map);
    let mut count = 0;
    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let nx = px + dx;
        let ny = py + dy;
        if nx >= 1 && nx <= mw && ny >= 1 && ny <= mh
            && !gd.is_blocked_tile(map, nx, ny)
            && (nx - npc_x).abs() + (ny - npc_y).abs() <= 5
        {
            count += 1;
        }
    }
    count
}
