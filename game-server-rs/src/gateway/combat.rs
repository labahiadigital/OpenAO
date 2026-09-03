use openao_protocol::opcodes::client_packet_id;

use super::packets::*;
use super::GameSession;
use crate::error::{GameError, GameErrorCode, HandlerResult};
use crate::gateway::WsSink;
use crate::world::{EntityId, Position, Scene};

const NPC_TYPE_DRAGON: i32 = 6;
const MAX_SUMMONS_PER_USER: usize = 3;
const SUMMON_DURATION_MS: u64 = 60_000;

enum CombatTarget {
    Npc(u32),
    Player(u32),
}

/// Validate a ranged/spell target against lag-compensated history.
/// Returns `true` if the hit is confirmed valid, or if no history is
/// available (graceful fallback — never blocks a hit due to infra gaps).
fn lag_validate_target(scene: &Scene, target_id: u32, attacker_pos: &Position, max_range: i32) -> bool {
    let Ok(mut history) = scene.lag_history.try_lock() else {
        return true;
    };
    let Some(current_tick) = history.current_tick() else {
        return true;
    };
    let rewind_ticks: u64 = 3;
    let query_tick = current_tick.saturating_sub(rewind_ticks);
    match history.query_entity_at_tick(query_tick, target_id) {
        Some(snap) => {
            if snap.dead { return false; }
            let dist = (snap.pos.x - attacker_pos.x).abs() + (snap.pos.y - attacker_pos.y).abs();
            dist <= max_range
        }
        None => true,
    }
}

impl GameSession {
    /// Find closest attackable NPC using AOI grid for O(1) cell lookups,
    /// then Manhattan distance filter within max_range.
    fn find_closest_npc(scene: &Scene, pos: &Position, max_range: i32) -> Option<u32> {
        let nearby = scene.entities_in_range(pos);
        let mut closest_id: Option<u32> = None;
        let mut closest_dist = i32::MAX;
        for eid in nearby {
            if let Some(npc) = scene.npcs.get(&eid) {
                if npc.dead || npc.max_hp <= 0 {
                    continue;
                }
                let dist = (npc.pos.x - pos.x).abs() + (npc.pos.y - pos.y).abs();
                if dist <= max_range && dist < closest_dist {
                    closest_dist = dist;
                    closest_id = Some(npc.id);
                }
            }
        }
        closest_id
    }

    /// Find closest attackable player using AOI grid for O(1) cell lookups.
    fn find_closest_player(
        scene: &Scene,
        pos: &Position,
        max_range: i32,
        exclude_id: u32,
    ) -> Option<u32> {
        let nearby = scene.entities_in_range(pos);
        let mut closest_id: Option<u32> = None;
        let mut closest_dist = i32::MAX;
        for eid in nearby {
            if eid == exclude_id {
                continue;
            }
            if let Some(p) = scene.players.get(&eid) {
                if p.dead {
                    continue;
                }
                let dist = (p.pos.x - pos.x).abs() + (p.pos.y - pos.y).abs();
                if dist <= max_range && dist < closest_dist {
                    closest_dist = dist;
                    closest_id = Some(p.id);
                }
            }
        }
        closest_id
    }

    fn find_target_at_tile(
        scene: &Scene,
        tile_pos: &Position,
        attacker_id: u32,
        safe_map: bool,
    ) -> Option<CombatTarget> {
        // Check NPCs at the exact tile
        for entry in scene.npcs.iter() {
            let npc = entry.value();
            if npc.dead || npc.max_hp <= 0 { continue; }
            if npc.pos.x == tile_pos.x && npc.pos.y == tile_pos.y {
                return Some(CombatTarget::Npc(npc.id));
            }
        }
        if safe_map { return None; }
        let attacker_safe = scene.players.get(&attacker_id)
            .map(|p| (p.seguro_activado, p.seguro_clan_activado, p.clan_id.clone()))
            .unwrap_or((false, false, None));
        if attacker_safe.0 { return None; }
        let attacker_faction = scene.players.get(&attacker_id)
            .map(|p| p.faction.clone()).unwrap_or_default();
        let attacker_criminal = scene.players.get(&attacker_id)
            .map(|p| p.criminal).unwrap_or(false);
        for entry in scene.players.iter() {
            let p = entry.value();
            if p.id == attacker_id || p.dead { continue; }
            if p.pos.x != tile_pos.x || p.pos.y != tile_pos.y { continue; }
            if p.invisible { continue; }
            if attacker_safe.1 {
                if let Some(ref aclan) = attacker_safe.2 {
                    if p.clan_id.as_deref() == Some(aclan.as_str()) { continue; }
                }
            }
            let is_citizen = |fac: &str, crim: bool| {
                fac == "armada" || (fac != "caos" && !crim)
            };
            if is_citizen(&attacker_faction, attacker_criminal) && is_citizen(&p.faction, p.criminal) {
                if let (Some(aclan), Some(pclan)) = (&attacker_safe.2, &p.clan_id) {
                    if aclan == pclan { continue; }
                }
                if attacker_safe.2.is_some() { continue; }
            }
            return Some(CombatTarget::Player(p.id));
        }
        None
    }

    fn find_target(
        scene: &Scene,
        pos: &Position,
        max_range: i32,
        attacker_id: u32,
        safe_map: bool,
    ) -> Option<CombatTarget> {
        if let Some(npc_id) = Self::find_closest_npc(scene, pos, max_range) {
            return Some(CombatTarget::Npc(npc_id));
        }
        if safe_map {
            return None;
        }
        let attacker_safe = scene.players.get(&attacker_id)
            .map(|p| (p.seguro_activado, p.seguro_clan_activado, p.clan_id.clone()))
            .unwrap_or((false, false, None));

        if attacker_safe.0 {
            return None;
        }

        let attacker_faction = scene.players.get(&attacker_id)
            .map(|p| p.faction.clone()).unwrap_or_default();
        let attacker_criminal = scene.players.get(&attacker_id)
            .map(|p| p.criminal).unwrap_or(false);

        if let Some(player_id) =
            Self::find_closest_player(scene, pos, max_range, attacker_id)
        {
            if attacker_safe.1
                && let Some(ref attacker_clan) = attacker_safe.2 {
                    let target_clan = scene.players.get(&player_id)
                        .and_then(|p| p.clan_id.clone());
                    if target_clan.as_deref() == Some(attacker_clan.as_str()) {
                        return None;
                    }
                }

            let is_attacker_citizen = attacker_faction == "armada" || (!attacker_criminal && attacker_faction == "none");
            if is_attacker_citizen && attacker_safe.2.is_some() {
                if let Some(target) = scene.players.get(&player_id) {
                    let is_target_citizen = target.faction == "armada" || (!target.criminal && target.faction == "none");
                    if is_target_citizen {
                        return None;
                    }
                }
            }

            return Some(CombatTarget::Player(player_id));
        }
        None
    }

    async fn attack_npc(
        &self,
        entity_id: u32,
        target_id: u32,
        damage: i32,
        scene: &Scene,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (npc_dead, npc_hp, npc_max_hp, npc_pos, npc_type, npc_exp) = {
            let mut npc = match scene.npcs.get_mut(&target_id) {
                Some(n) => n,
                None => return Ok(()),
            };
            npc.hp = (npc.hp - damage).max(0);
            let dead = npc.hp <= 0;
            if dead {
                npc.dead = true;
            }
            npc.aggro_target = Some(entity_id);
            (dead, npc.hp, npc.max_hp, npc.pos.clone(), npc.npc_type, npc.exp_reward)
        };

        let gd = self.world.gd();
        let npc_name = gd
            .get_npc(npc_type)
            .map(|t| t.name.as_str())
            .unwrap_or("NPC");

        let npc_vitals = crate::replication::build_entity_vitals_delta(target_id, npc_hp, npc_max_hp, 0, 0);
        scene.broadcast_in_range(0, &npc_pos, npc_vitals);

        let msg = format!(
            "Golpeas a {} por {} de daño (HP: {})",
            npc_name, damage, npc_hp
        );
        self.send_to_client(sink, build_console_message(&msg))
            .await?;

        if npc_dead {
            tracing::info!(
                target: "activity",
                category = "combat", action = "npc_kill",
                player = ?self.character_name, entity = entity_id,
                npc = npc_name, npc_type = npc_type,
                "NPC_KILL"
            );
            let kill_msg = format!("Has matado a {}", npc_name);
            self.send_to_client(sink, build_console_message(&kill_msg))
                .await?;

            let del_pkt = crate::replication::build_delete_character_packet(target_id);
            scene.broadcast_in_range(entity_id, &npc_pos, del_pkt.clone());
            self.send_to_client(sink, del_pkt).await?;

            scene.aoi_remove(target_id);
            scene.npcs.remove(&target_id);

            self.drop_npc_loot(scene, npc_type, &npc_pos);
            self.grant_npc_gold(entity_id, npc_type, scene, sink)
                .await?;
            let raw_xp = if npc_exp > 0 { npc_exp } else { 25 };
            let mut xp = raw_xp * crate::gameplay::combat_formulas::NPC_EXP_MULTIPLIER;

            let mid = self.map_id.unwrap_or(0);
            if let Ok(tm) = self.world.territories.try_lock()
                && let Some(t) = tm.get_territory_for_map(mid)
                    && let Some(ref owner) = t.owner_clan
                        && let Some(p) = scene.players.get(&entity_id)
                            && p.clan_id.as_deref() == Some(owner.as_str()) {
                                xp = (xp as f64 * (1.0 + t.bonus_exp_pct as f64 / 100.0)) as i32;
                            }

            self.grant_xp(entity_id, xp, scene, sink).await?;

            self.advance_quest_kills(entity_id, npc_type);

            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                p.achievements.stats.total_npc_kills += 1;
                let defs = crate::gameplay::achievements::default_achievements();
                let level = p.level as u32;
                let gold = p.gold;
                let unlocked_ids = p.achievements.check_and_unlock(&defs, level, gold);
                let names: Vec<String> = unlocked_ids.iter().filter_map(|id| {
                    defs.iter().find(|d| d.id == *id).map(|d| d.name.clone())
                }).collect();
                drop(p);
                for name in names {
                    self.send_to_client(sink, build_console_message(
                        &format!("Logro desbloqueado: {}", name),
                    )).await?;
                }
            }
        }

        Ok(())
    }

    async fn attack_player(
        &self,
        attacker_id: u32,
        target_id: u32,
        damage: i32,
        scene: &Scene,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (target_was_criminal, target_faction) = scene
            .players
            .get(&target_id)
            .map(|p| (p.criminal, p.faction.clone()))
            .unwrap_or((false, "none".to_string()));

        let attacker_faction = scene.players.get(&attacker_id)
            .map(|p| p.faction.clone())
            .unwrap_or_else(|| "none".to_string());

        let is_rival_faction = matches!(
            (attacker_faction.as_str(), target_faction.as_str()),
            ("armada", "caos") | ("caos", "armada")
        );

        if !target_was_criminal && !is_rival_faction
            && let Some(mut attacker) = scene.players.get_mut(&attacker_id)
            && !attacker.criminal
        {
            if attacker.faction == "armada" && target_faction == "none" {
                attacker.faction = "none".to_string();
                attacker.faction_rank = 0;
                attacker.faction_rank_armada = 0;
                drop(attacker);
                self.send_to_client(sink, build_console_message(
                    "Perdiste tu enlistamiento en la Armada por atacar a un ciudadano."
                )).await?;
            } else {
                drop(attacker);
            }

            if let Some(mut attacker) = scene.players.get_mut(&attacker_id) {
                attacker.criminal = true;
            }
            let attacker_faction_for_color = scene.players.get(&attacker_id)
                .map(|p| p.faction.clone()).unwrap_or_else(|| "none".to_string());
            self.send_to_client(
                sink,
                build_console_message("Ahora eres criminal por atacar a un ciudadano."),
            )
            .await?;
            let color = get_name_color(true, &attacker_faction_for_color, false);
            let color_pkt = build_act_color_name(attacker_id, color);
            self.send_to_client(sink, color_pkt.clone()).await?;
            let attacker_pos_for_color = scene.players.get(&attacker_id).map(|p| p.pos.clone());
            if let Some(ref pos) = attacker_pos_for_color {
                scene.broadcast_in_range(attacker_id, pos, color_pkt);
            } else {
                scene.broadcast(attacker_id, color_pkt);
            }
        }

        let (target_dead, target_hp, target_max_hp, target_mana, target_max_mana, target_name) = {
            let mut target = match scene.players.get_mut(&target_id) {
                Some(p) => p,
                None => return Ok(()),
            };
            if target.logout_expires_at_ms > 0 {
                target.logout_expires_at_ms = 0;
                scene.send_to_player(target_id, build_console_message("[Servidor] La salida se canceló porque recibiste un ataque."));
            }
            target.hp = (target.hp - damage).max(0);
            let dead = target.hp <= 0;
            if dead {
                target.dead = true;
                target.dead_world_active = false;
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                target.dead_world_transition_at_ms = now_ms + crate::gameplay::combat_formulas::DEAD_WORLD_DELAY_MS;
            }
            (dead, target.hp, target.max_hp, target.mana, target.max_mana, target.name.clone())
        };

        let vitals_pkt =
            crate::replication::build_entity_vitals_delta(target_id, target_hp, target_max_hp, target_mana, target_max_mana);
        let target_pos = scene.players.get(&target_id).map(|p| p.pos.clone());
        if let Some(ref pos) = target_pos {
            scene.broadcast_in_range(0, pos, vitals_pkt.clone());
        } else {
            scene.broadcast(0, vitals_pkt.clone());
        }
        self.send_to_client(sink, vitals_pkt).await?;

        let msg = format!(
            "Golpeas a {} por {} de daño (HP: {})",
            target_name, damage, target_hp
        );
        self.send_to_client(sink, build_console_message(&msg))
            .await?;

        let target_msg = format!(
            "{} te golpea por {} de daño",
            self.character_name.as_deref().unwrap_or("???"),
            damage
        );
        scene.send_to_player(target_id, build_console_message(&target_msg));

        let self_vitals =
            build_self_vitals(target_hp, target_max_hp, target_mana, target_max_mana);
        scene.send_to_player(target_id, self_vitals);

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let pvp_block = now_ms + 5000;
        if let Some(mut p) = scene.players.get_mut(&attacker_id) {
            p.pvp_block_until_ms = pvp_block;
        }
        if let Some(mut p) = scene.players.get_mut(&target_id) {
            p.pvp_block_until_ms = pvp_block;
        }

        if target_dead {
            tracing::info!(
                target: "activity",
                category = "combat", action = "pvp_kill",
                attacker = ?self.character_name, attacker_entity = attacker_id,
                victim = %target_name, victim_entity = target_id,
                "PVP_KILL"
            );
            let kill_msg = format!("Has matado a {}", target_name);
            self.send_to_client(sink, build_console_message(&kill_msg))
                .await?;

            let death_msg = format!(
                "{} te ha matado",
                self.character_name.as_deref().unwrap_or("???")
            );
            scene.send_to_player(target_id, build_console_message(&death_msg));

            let (death_head, death_body, death_helmet, death_weapon, death_shield) = {
                scene.players.get(&target_id)
                    .map(|p| (p.id_head, p.id_body, p.id_helmet, p.id_weapon, p.id_shield))
                    .unwrap_or((0, 0, 0, 0, 0))
            };
            let death_pkt = build_put_body_and_head_dead(target_id, death_head, death_body, death_helmet, death_weapon, death_shield);
            let death_pos = scene.players.get(&target_id).map(|p| p.pos.clone());
            if let Some(ref pos) = death_pos {
                scene.broadcast_in_range(0, pos, death_pkt.clone());
            } else {
                scene.broadcast(0, death_pkt.clone());
            }
            self.send_to_client(sink, death_pkt).await?;

            if let Some(target_char_id) = self.find_character_id_for_entity(target_id)
                && let Some(ref pos) = death_pos
            {
                self.drop_items_on_death(&target_char_id, pos, scene).await;
            }

            // --- Death cleanup: buffs, CC, meditation ---
            if let Some(mut target) = scene.players.get_mut(&target_id) {
                target.buffs.clear();
                target.paralizado = false;
                target.paralizado_until_ms = 0;
                target.inmovilizado = false;
                target.inmovilizado_until_ms = 0;
                target.meditar = false;
                target.hidden_skill = false;
                target.hidden_skill_expire_tick = 0;
            }

            // --- PvP kill rewards (port from respawn.ts) ---
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let (attacker_level, attacker_faction, attacker_criminal, victim_level, victim_criminal, victim_faction) = {
                let att = scene.players.get(&attacker_id);
                let vic = scene.players.get(&target_id);
                match (att, vic) {
                    (Some(a), Some(v)) => (
                        a.level, a.faction.clone(), a.criminal,
                        v.level, v.criminal, v.faction.clone(),
                    ),
                    _ => (1, "none".to_string(), false, 1, false, "none".to_string()),
                }
            };

            let victim_is_newbie = victim_level <= crate::gameplay::combat_formulas::NEWBIE_MAX_LEVEL;

            let rekill_key = (attacker_id, target_id);
            let is_rekill_blocked = if let Some(last_kill) = self.world.faction_rekill_tracker.get(&rekill_key) {
                now_ms.saturating_sub(*last_kill) < 5 * 60 * 1000
            } else {
                false
            };

            // Kill counters
            if !victim_is_newbie && !is_rekill_blocked {
                if let Some(mut att) = scene.players.get_mut(&attacker_id) {
                    if victim_criminal {
                        att.criminales_matados += 1;
                    } else {
                        att.ciudadanos_matados += 1;
                    }
                }
            }

            // PvP exp/gold rewards
            const PVP_BASE_EXP: i32 = 50;
            const PVP_BASE_GOLD: i32 = 10;
            let mut pvp_exp = PVP_BASE_EXP * crate::gameplay::combat_formulas::NPC_EXP_MULTIPLIER;
            let mut pvp_gold = PVP_BASE_GOLD * crate::gameplay::combat_formulas::NPC_GOLD_MULTIPLIER;

            if self.world.double_exp.load(std::sync::atomic::Ordering::Relaxed) {
                pvp_exp *= 2;
            }
            if self.world.double_gold.load(std::sync::atomic::Ordering::Relaxed) {
                pvp_gold *= 2;
            }

            if !victim_is_newbie {
                if let Some(mut att) = scene.players.get_mut(&attacker_id) {
                    att.exp += pvp_exp;
                    att.gold = crate::gameplay::balance::clamp_gold(att.gold as i64 + pvp_gold as i64) as i32;
                }
                let exp_msg = format!("¡Has ganado {} puntos de experiencia!", pvp_exp);
                self.send_to_client(sink, build_console_message(&exp_msg)).await?;
                let gold_msg = format!("¡Has ganado {} monedas de oro!", pvp_gold);
                self.send_to_client(sink, build_console_message(&gold_msg)).await?;

                // Send updated gold/exp to attacker
                if let Some(att) = scene.players.get(&attacker_id) {
                    self.send_to_client(sink, build_act_gold(att.gold)).await?;
                    self.send_to_client(sink, build_act_exp(att.exp, att.exp_next_level)).await?;
                }

                Self::check_level_up_and_notify(attacker_id, scene, &self.world);
            }

            // Faction score
            let can_count_faction = !victim_is_newbie
                && (attacker_level - victim_level <= 10 || victim_level >= 25);
            if can_count_faction && !is_rekill_blocked {
                const FACTION_SCORE_PER_KILL: i32 = 10;

                let attacker_is_citizen = !attacker_criminal && attacker_faction == "none";
                let attacker_is_armada = attacker_faction == "armada";
                let victim_is_enemy = victim_criminal || victim_faction == "caos";
                let should_award_armada = (attacker_is_citizen || attacker_is_armada) && victim_is_enemy;

                let attacker_is_criminal_no_faction = attacker_criminal && attacker_faction == "none";
                let attacker_is_caos = attacker_faction == "caos";
                let should_award_caos = attacker_is_criminal_no_faction || attacker_is_caos;

                let mut awarded = false;
                if should_award_armada {
                    if let Some(mut att) = scene.players.get_mut(&attacker_id) {
                        att.faction_score_armada += FACTION_SCORE_PER_KILL;
                        att.faction_score += FACTION_SCORE_PER_KILL;
                    }
                    awarded = true;
                }
                if should_award_caos {
                    if let Some(mut att) = scene.players.get_mut(&attacker_id) {
                        att.faction_score_caos += FACTION_SCORE_PER_KILL;
                        att.faction_score += FACTION_SCORE_PER_KILL;
                    }
                    awarded = true;
                }
                if awarded {
                    let score_msg = format!("¡Has ganado {} puntos de facción!", FACTION_SCORE_PER_KILL);
                    self.send_to_client(sink, build_console_message(&score_msg)).await?;
                }

                self.world.faction_rekill_tracker.insert(rekill_key, now_ms);
            }
        }

        Ok(())
    }

    pub(super) async fn handle_attack_melee(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id) = match (self.entity_id, self.map_id) {
            (Some(e), Some(m)) => (e, m),
            _ => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);

        let now = self.world.uptime_ms();
        if let Some(p) = scene.players.get(&entity_id) {
            if !p.action_cooldowns.can_melee(now) {
                return Ok(());
            }
        }

        if let Some(p) = scene.players.get(&entity_id)
            && p.paralizado {
                self.send_to_client(sink, build_console_message("Estás paralizado y no puedes atacar.")).await?;
                return Ok(());
            }

        {
            let keep_hidden = scene.players.get(&entity_id).map(|p| super::inventory::can_keep_hidden_while_acting(&p)).unwrap_or(false);
            if !keep_hidden {
                let tick = self.metrics.current_tick.load(std::sync::atomic::Ordering::Relaxed);
                super::inventory::stop_hidden_skill(entity_id, &scene, tick, 150);
            }
        }

        let (player_pos, player_heading) = match scene.players.get(&entity_id) {
            Some(p) => (p.pos.clone(), p.heading),
            None => return Ok(()),
        };
        let is_safe = self.world.gd().is_safe_position(map_id, player_pos.x, player_pos.y)
            && !crate::gameplay::arenas::is_arena_map(map_id);

        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.action_cooldowns.trigger_melee(now);
        }

        let (dx, dy) = crate::gateway::packets::heading_to_delta(player_heading);
        let target_tile_x = player_pos.x + dx;
        let target_tile_y = player_pos.y + dy;
        let target_pos = crate::world::Position { map: map_id, x: target_tile_x, y: target_tile_y };
        let target =
            Self::find_target_at_tile(&scene, &target_pos, entity_id, is_safe);

        match target {
            Some(CombatTarget::Npc(npc_id)) => {
                let (hit, damage, ds_hit) = self.do_melee_vs_npc(&scene, entity_id, npc_id);
                if hit {
                    if ds_hit {
                        self.consume_dragon_slayer_sword(entity_id, &scene, sink).await?;
                    }
                    self.attack_npc(entity_id, npc_id, damage, &scene, sink)
                        .await?;
                } else {
                    self.send_to_client(sink, build_console_message("¡Fallas!")).await?;
                }
            }
            Some(CombatTarget::Player(pid)) => {
                let (hit, damage, shield_blocked) = self.do_melee_vs_player(&scene, entity_id, pid);
                if hit {
                    self.attack_player(entity_id, pid, damage, &scene, sink)
                        .await?;
                } else if shield_blocked {
                    self.send_to_client(sink, build_console_message("El oponente bloqueó tu ataque con el escudo.")).await?;
                } else {
                    self.send_to_client(sink, build_console_message("¡Fallas!")).await?;
                }
            }
            None => {
                let err = if is_safe {
                    GameError::new(GameErrorCode::SafeZoneBlocked, "No puedes atacar en zona segura.")
                } else {
                    GameError::new(GameErrorCode::TargetNotFound, "No hay enemigo cercano")
                };
                self.send_to_client(sink, err.to_console_packet()).await?;
            }
        }

        Ok(())
    }

    async fn consume_dragon_slayer_sword(
        &self,
        entity_id: u32,
        scene: &Scene,
        sink: &mut WsSink,
    ) -> HandlerResult {
        use crate::gameplay::combat_formulas::DRAGON_SLAYER_SWORD_ITEM_ID;
        let char_id = scene.players.get(&entity_id)
            .map(|p| p.character_id.clone())
            .unwrap_or_default();
        let inv = self.world.cache_get_inventory(&char_id);
        if let Some(slot_row) = inv.iter().find(|r| r.item_id == DRAGON_SLAYER_SWORD_ITEM_ID && r.equipped) {
            let slot = slot_row.slot;
            self.world.cache_delete_slot(&char_id, slot);
            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                p.id_weapon = 0;
                let pos = p.pos.clone();
                drop(p);
                let change_w = build_change_equipment(
                    openao_protocol::opcodes::client_packet_id::CHANGE_WEAPON,
                    entity_id,
                    0,
                );
                scene.broadcast_in_range(entity_id, &pos, change_w);
            }
        }
        self.send_to_client(
            sink,
            build_console_message("La Espada Mata Dragones atraviesa al dragón de un golpe y se consume."),
        ).await?;
        Ok(())
    }

    pub(super) async fn handle_attack_range(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id) = match (self.entity_id, self.map_id) {
            (Some(e), Some(m)) => (e, m),
            _ => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);

        let now = self.world.uptime_ms();
        if let Some(p) = scene.players.get(&entity_id) {
            if !p.action_cooldowns.can_range(now) {
                return Ok(());
            }
        }
        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.action_cooldowns.trigger_range(now);
        }

        if let Some(p) = scene.players.get(&entity_id)
            && p.paralizado {
                self.send_to_client(sink, build_console_message("Estás paralizado y no puedes atacar.")).await?;
                return Ok(());
            }

        {
            let keep_hidden = scene.players.get(&entity_id).map(|p| super::inventory::can_keep_hidden_while_acting(&p)).unwrap_or(false);
            if !keep_hidden {
                let tick = self.metrics.current_tick.load(std::sync::atomic::Ordering::Relaxed);
                super::inventory::stop_hidden_skill(entity_id, &scene, tick, 150);
            }
        }

        let player_pos = match scene.players.get(&entity_id) {
            Some(p) => p.pos.clone(),
            None => return Ok(()),
        };
        let is_safe = self.world.gd().is_safe_position(map_id, player_pos.x, player_pos.y)
            && !crate::gameplay::arenas::is_arena_map(map_id);

        let target =
            Self::find_target(&scene, &player_pos, 8, entity_id, is_safe);

        match target {
            Some(CombatTarget::Npc(npc_id)) => {
                if !lag_validate_target(&scene, npc_id, &player_pos, 8) {
                    self.send_to_client(sink, GameError::new(GameErrorCode::TargetNotFound, "El objetivo se movió fuera de rango").to_console_packet()).await?;
                    return Ok(());
                }
                if let Some(npc) = scene.npcs.get(&npc_id) {
                    let proj = crate::replication::build_create_projectile(
                        player_pos.x, player_pos.y,
                        npc.pos.x, npc.pos.y,
                        5000,
                    );
                    scene.broadcast_in_range(0, &player_pos, proj);
                }
                let (hit, damage, ds_hit) = self.do_melee_vs_npc(&scene, entity_id, npc_id);
                if hit {
                    if ds_hit {
                        self.consume_dragon_slayer_sword(entity_id, &scene, sink).await?;
                    }
                    self.attack_npc(entity_id, npc_id, damage, &scene, sink)
                        .await?;
                } else {
                    self.send_to_client(sink, build_console_message("¡Fallas!")).await?;
                }
            }
            Some(CombatTarget::Player(pid)) => {
                if !lag_validate_target(&scene, pid, &player_pos, 8) {
                    self.send_to_client(sink, GameError::new(GameErrorCode::TargetNotFound, "El objetivo se movió fuera de rango").to_console_packet()).await?;
                    return Ok(());
                }
                if let Some(target_player) = scene.players.get(&pid) {
                    let proj = crate::replication::build_create_projectile(
                        player_pos.x, player_pos.y,
                        target_player.pos.x, target_player.pos.y,
                        5000,
                    );
                    scene.broadcast_in_range(0, &player_pos, proj);
                }
                let (hit, damage, _shield) = self.do_melee_vs_player(&scene, entity_id, pid);
                if hit {
                    self.attack_player(entity_id, pid, damage, &scene, sink)
                        .await?;
                } else {
                    self.send_to_client(sink, build_console_message("¡Fallas!")).await?;
                }
            }
            None => {
                let err = if is_safe {
                    GameError::new(GameErrorCode::SafeZoneBlocked, "No puedes atacar en zona segura.")
                } else {
                    GameError::new(GameErrorCode::TargetNotFound, "No hay enemigo en rango")
                };
                self.send_to_client(sink, err.to_console_packet()).await?;
            }
        }

        Ok(())
    }

    pub(super) async fn handle_attack_spell(
        &mut self,
        spell_slot: u8,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id) = match (self.entity_id, self.map_id) {
            (Some(e), Some(m)) => (e, m),
            _ => return Ok(()),
        };

        let scene_check = self.world.get_or_create_scene(map_id);

        let now = self.world.uptime_ms();
        if let Some(p) = scene_check.players.get(&entity_id) {
            if !p.action_cooldowns.can_spell(now) {
                return Ok(());
            }
        }
        if let Some(mut p) = scene_check.players.get_mut(&entity_id) {
            p.action_cooldowns.trigger_spell(now);
        }

        if let Some(p) = scene_check.players.get(&entity_id)
            && p.paralizado {
                self.send_to_client(sink, build_console_message("Estás paralizado y no puedes lanzar hechizos.")).await?;
                return Ok(());
            }
        {
            let keep_hidden = scene_check.players.get(&entity_id).map(|p| super::inventory::can_keep_hidden_while_acting(&p)).unwrap_or(false);
            if !keep_hidden {
                let tick = self.metrics.current_tick.load(std::sync::atomic::Ordering::Relaxed);
                super::inventory::stop_hidden_skill(entity_id, &scene_check, tick, 150);
            }
        }
        drop(scene_check);

        let spell_id =
            match crate::replication::DEFAULT_SPELLS.iter().find(|(s, _)| *s == spell_slot) {
                Some((_, id)) => *id,
                None => {
                    self.send_to_client(
                        sink,
                        build_console_message("Hechizo no encontrado en ese slot"),
                    )
                    .await?;
                    return Ok(());
                }
            };

        let spell = crate::replication::get_spell_data(&self.world.gd(), spell_id);
        let scene = self.world.get_or_create_scene(map_id);

        {
            let mut player = match scene.players.get_mut(&entity_id) {
                Some(p) => p,
                None => return Ok(()),
            };
            if player.dead {
                let err = GameError::new(GameErrorCode::AlreadyDead, "Estas muerto, no puedes lanzar hechizos");
                self.send_to_client(sink, err.to_console_packet()).await?;
                return Ok(());
            }
            {
                let gd = self.world.gd();
                if let Some(st) = gd.get_spell(spell_id as i32) {
                    let simulated_skill = crate::gameplay::combat_formulas::simulated_skill(player.level);
                    if simulated_skill < st.min_skill {
                        let req_level = ((st.min_skill as f64) / 3.0).ceil() as i32;
                        self.send_to_client(sink, build_console_message(
                            &format!("Necesitas ser nivel {} para lanzar ese hechizo.", req_level),
                        )).await?;
                        return Ok(());
                    }
                }
            }
            if player.mana < spell.mana_cost as i32 {
                let err = GameError::new(GameErrorCode::InsufficientMana, "Mana insuficiente");
                self.send_to_client(sink, err.to_console_packet()).await?;
                return Ok(());
            }
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let spell_id_i32 = spell_id as i32;
            let cd_ms = crate::gameplay::cooldowns::default_spell_cooldown(spell_id_i32);
            if !player.spell_cooldowns.is_ready(spell_id_i32, now_ms) {
                let remaining = player.spell_cooldowns.remaining(spell_id_i32, now_ms);
                self.send_to_client(sink, build_console_message(
                    &format!("Hechizo en cooldown ({:.1}s)", remaining as f64 / 1000.0),
                )).await?;
                return Ok(());
            }
            player.spell_cooldowns.trigger(spell_id_i32, cd_ms, now_ms);
        }

        {
            let gd = self.world.gd();
            if let Some(st) = gd.get_spell(spell_id as i32) {
                if st.num_npc > 0 {
                    drop(scene);
                    self.handle_summon_spell(entity_id, map_id, spell_id, st.num_npc, &spell, sink).await?;
                    return Ok(());
                }
                if st.remover_paralisis == 1 {
                    if let Some(mut p) = scene.players.get_mut(&entity_id) {
                        p.mana -= spell.mana_cost as i32;
                        p.paralizado = false;
                        p.inmovilizado = false;
                        let vitals = build_self_vitals(p.hp, p.max_hp, p.mana, p.max_mana);
                        drop(p);
                        self.send_to_client(sink, vitals).await?;
                        self.send_to_client(sink, build_console_message("Te has liberado de la parálisis.")).await?;
                    }
                    return Ok(());
                }
                if st.invisibilidad == 1 {
                    if let Some(mut p) = scene.players.get_mut(&entity_id) {
                        p.mana -= spell.mana_cost as i32;
                        p.invisible = true;
                        let vitals = build_self_vitals(p.hp, p.max_hp, p.mana, p.max_mana);
                        let pos = p.pos.clone();
                        drop(p);
                        self.send_to_client(sink, vitals).await?;
                        self.send_to_client(sink, build_console_message("Eres invisible.")).await?;
                        let del = crate::replication::build_delete_character_packet(entity_id);
                        scene.broadcast_in_range(entity_id, &pos, del);
                    }
                    return Ok(());
                }
                if st.sube_ag == 1 {
                    if let Some(mut p) = scene.players.get_mut(&entity_id) {
                        p.mana -= spell.mana_cost as i32;
                        let buff_amount = if st.max_ag > st.min_ag {
                            rand::Rng::random_range(&mut rand::rng(), st.min_ag..=st.max_ag)
                        } else { st.max_ag.max(1) };
                        p.buffs.apply(crate::gameplay::buffs::BuffType::Agility, buff_amount, 60 * 30);
                        let vitals = build_self_vitals(p.hp, p.max_hp, p.mana, p.max_mana);
                        drop(p);
                        self.send_to_client(sink, vitals).await?;
                        self.send_to_client(sink, build_console_message(&format!("Agilidad +{}", buff_amount))).await?;
                    }
                    if spell.fx_id > 0 {
                        let pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                        if let Some(pos) = pos {
                            let fx = crate::replication::build_anim_fx(entity_id, spell.fx_id);
                            scene.broadcast_in_range(0, &pos, fx);
                        }
                    }
                    return Ok(());
                }
                if st.sube_fz == 1 {
                    if let Some(mut p) = scene.players.get_mut(&entity_id) {
                        p.mana -= spell.mana_cost as i32;
                        let buff_amount = if st.max_fz > st.min_fz {
                            rand::Rng::random_range(&mut rand::rng(), st.min_fz..=st.max_fz)
                        } else { st.max_fz.max(1) };
                        p.buffs.apply(crate::gameplay::buffs::BuffType::Strength, buff_amount, 60 * 30);
                        let vitals = build_self_vitals(p.hp, p.max_hp, p.mana, p.max_mana);
                        drop(p);
                        self.send_to_client(sink, vitals).await?;
                        self.send_to_client(sink, build_console_message(&format!("Fuerza +{}", buff_amount))).await?;
                    }
                    if spell.fx_id > 0 {
                        let pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                        if let Some(pos) = pos {
                            let fx = crate::replication::build_anim_fx(entity_id, spell.fx_id);
                            scene.broadcast_in_range(0, &pos, fx);
                        }
                    }
                    return Ok(());
                }
            }
        }

        match spell.spell_type {
            crate::replication::SpellType::Heal => {
                let base_heal = {
                    let mut rng = rand::rng();
                    rand::Rng::random_range(&mut rng, spell.min_damage..=spell.max_damage)
                };
                let caster_level = scene.players.get(&entity_id).map(|p| p.level).unwrap_or(1);
                let heal_amount = base_heal + ((base_heal as f64 * 3.0 * caster_level as f64) / 100.0).round() as i32;
                let heal_amount = heal_amount.max(1);

                let (player_pos, caster_faction, caster_criminal) = scene.players.get(&entity_id)
                    .map(|p| (Some(p.pos.clone()), p.faction.clone(), p.criminal))
                    .unwrap_or((None, String::new(), false));
                let is_arena = crate::gameplay::arenas::is_arena_map(map_id);
                let caster_is_citizen = caster_faction == "armada" || (!caster_criminal && caster_faction == "none");
                let target_pid = if let Some(ref pos) = player_pos {
                    let mut best: Option<(u32, i32)> = None;
                    for p in scene.players.iter() {
                        if p.dead { continue; }
                        if *p.key() == entity_id { continue; }
                        if caster_is_citizen && !is_arena && p.criminal { continue; }
                        let dist = (p.pos.x - pos.x).abs() + (p.pos.y - pos.y).abs();
                        if dist <= 8 && best.map_or(true, |(_, d)| dist < d) {
                            best = Some((*p.key(), dist));
                        }
                    }
                    best.map(|(id, _)| id)
                } else { None };

                let heal_target = target_pid.unwrap_or(entity_id);

                if let Some(mut caster) = scene.players.get_mut(&entity_id) {
                    caster.mana -= spell.mana_cost as i32;
                    let vitals = build_self_vitals(caster.hp, caster.max_hp, caster.mana, caster.max_mana);
                    self.send_to_client(sink, vitals).await?;
                }

                if heal_target == entity_id {
                    if let Some(mut player) = scene.players.get_mut(&entity_id) {
                        player.hp = (player.hp + heal_amount).min(player.max_hp);
                        let (hp, max_hp, mana, max_mana) = (player.hp, player.max_hp, player.mana, player.max_mana);
                        let fx_pos = player.pos.clone();
                        drop(player);

                        let vitals = build_self_vitals(hp, max_hp, mana, max_mana);
                        self.send_to_client(sink, vitals).await?;
                        let entity_vitals = crate::replication::build_entity_vitals_delta(entity_id, hp, max_hp, mana, max_mana);
                        scene.broadcast_in_range(entity_id, &fx_pos, entity_vitals);

                        let msg = format!("{} te cura {} HP", spell.name, heal_amount);
                        self.send_to_client(sink, build_console_message(&msg)).await?;
                        let fx = crate::replication::build_anim_fx(entity_id, spell.fx_id);
                        scene.broadcast_in_range(0, &fx_pos, fx);
                    }
                } else {
                    if let Some(mut target) = scene.players.get_mut(&heal_target) {
                        target.hp = (target.hp + heal_amount).min(target.max_hp);
                        let (hp, max_hp, mana, max_mana) = (target.hp, target.max_hp, target.mana, target.max_mana);
                        let fx_pos = target.pos.clone();
                        let tname = target.name.clone();
                        drop(target);

                        let entity_vitals = crate::replication::build_entity_vitals_delta(heal_target, hp, max_hp, mana, max_mana);
                        scene.broadcast_in_range(heal_target, &fx_pos, entity_vitals);

                        scene.send_to_player(heal_target, build_self_vitals(hp, max_hp, mana, max_mana));
                        scene.send_to_player(heal_target, build_console_message(&format!("{} te curó {} HP", spell.name, heal_amount)));

                        let msg = format!("Curas {} HP a {}", heal_amount, tname);
                        self.send_to_client(sink, build_console_message(&msg)).await?;
                        let fx = crate::replication::build_anim_fx(heal_target, spell.fx_id);
                        scene.broadcast_in_range(0, &fx_pos, fx);
                    }
                }
            }
            crate::replication::SpellType::Attack | crate::replication::SpellType::Buff => {
                let player_pos = match scene.players.get(&entity_id) {
                    Some(p) => p.pos.clone(),
                    None => return Ok(()),
                };
                let is_safe = self.world.gd().is_safe_position(map_id, player_pos.x, player_pos.y)
                    && !crate::gameplay::arenas::is_arena_map(map_id);

                let target = Self::find_target(
                    &scene,
                    &player_pos,
                    8,
                    entity_id,
                    is_safe,
                );

                if let Some(mut player) = scene.players.get_mut(&entity_id) {
                    player.mana -= spell.mana_cost as i32;
                    let vitals = build_self_vitals(
                        player.hp,
                        player.max_hp,
                        player.mana,
                        player.max_mana,
                    );
                    self.send_to_client(sink, vitals).await?;
                }

                let (caster_level, caster_class) = scene.players.get(&entity_id)
                    .map(|p| (p.level, p.id_clase))
                    .unwrap_or((1, 1));

                let (wep_magic_bonus, wep_magic_pen, ring_magic_bonus, ring_magic_pen) =
                    self.get_player_magic_item_stats(&scene, entity_id);

                let base_damage = {
                    let mut rng = rand::rng();
                    rand::Rng::random_range(&mut rng, spell.min_damage..=spell.max_damage)
                };

                let magic_calc = crate::gameplay::combat_formulas::apply_magic_bonuses(
                    base_damage, caster_level as i32, caster_class,
                    wep_magic_bonus, wep_magic_pen, ring_magic_bonus, ring_magic_pen,
                );

                match target {
                    Some(CombatTarget::Npc(npc_id)) => {
                        if !lag_validate_target(&scene, npc_id, &player_pos, 8) {
                            self.send_to_client(sink, GameError::new(GameErrorCode::TargetNotFound, "El objetivo se movió fuera de rango").to_console_packet()).await?;
                            return Ok(());
                        }
                        let (npc_magic_res, npc_magic_def) = {
                            let gd = self.world.gd();
                            scene.npcs.get(&npc_id)
                                .map(|n| {
                                    let ndata = gd.get_npc(n.npc_type);
                                    let mr = ndata.map(|nd| nd.magic_resistance).unwrap_or(0);
                                    let md = ndata.map(|nd| nd.magic_def).unwrap_or(0);
                                    (mr, md)
                                })
                                .unwrap_or((0, 0))
                        };
                        let damage = crate::gameplay::combat_formulas::apply_magic_resistance_to_npc(
                            magic_calc.damage, caster_level as i32,
                            npc_magic_res, npc_magic_def, magic_calc.magic_penetration,
                        );
                        if let Some(npc) = scene.npcs.get(&npc_id) {
                            let proj = crate::replication::build_spell_projectile(
                                player_pos.x, player_pos.y,
                                npc.pos.x, npc.pos.y,
                                spell_slot as u16,
                            );
                            scene.broadcast_in_range(0, &player_pos, proj);
                        }
                        let fx = crate::replication::build_anim_fx(npc_id, spell.fx_id);
                        scene.broadcast_in_range(0, &player_pos, fx);
                        self.attack_npc(entity_id, npc_id, damage, &scene, sink)
                            .await?;

                        {
                            let gd = self.world.gd();
                            let spell_template = gd.get_spell(spell_id as i32);
                            if let Some(st) = spell_template {
                                let current_tick = self.metrics.current_tick.load(std::sync::atomic::Ordering::Relaxed);
                                let cc_duration_ticks: u64 = 180;
                                if st.paraliza == 1 {
                                    if let Some(mut npc) = scene.npcs.get_mut(&npc_id) {
                                        npc.paralizado = true;
                                        npc.inmovilizado = false;
                                        npc.cc_expire_tick = current_tick + cc_duration_ticks;
                                    }
                                } else if st.inmoviliza == 1 {
                                    if let Some(mut npc) = scene.npcs.get_mut(&npc_id) {
                                        npc.inmovilizado = true;
                                        npc.cc_expire_tick = current_tick + cc_duration_ticks;
                                    }
                                }
                            }
                        }
                    }
                    Some(CombatTarget::Player(pid)) => {
                        if !lag_validate_target(&scene, pid, &player_pos, 8) {
                            self.send_to_client(sink, GameError::new(GameErrorCode::TargetNotFound, "El objetivo se movió fuera de rango").to_console_packet()).await?;
                            return Ok(());
                        }
                        let (target_level, target_class, target_item_mr) = scene.players.get(&pid)
                            .map(|p| {
                                let item_mr = self.get_player_item_magic_resistance(&scene, pid);
                                (p.level as i32, p.id_clase, item_mr)
                            })
                            .unwrap_or((1, 1, 0));
                        let damage = crate::gameplay::combat_formulas::apply_magic_resistance_to_user(
                            magic_calc.damage, caster_level as i32,
                            target_level, target_class, target_item_mr, magic_calc.magic_penetration,
                        );
                        if let Some(target_player) = scene.players.get(&pid) {
                            let proj = crate::replication::build_spell_projectile(
                                player_pos.x, player_pos.y,
                                target_player.pos.x, target_player.pos.y,
                                spell_slot as u16,
                            );
                            scene.broadcast_in_range(0, &player_pos, proj);
                        }
                        let fx = crate::replication::build_anim_fx(pid, spell.fx_id);
                        scene.broadcast_in_range(0, &player_pos, fx);
                        self.attack_player(entity_id, pid, damage, &scene, sink)
                            .await?;
                    }
                    None => {
                        let err = if is_safe {
                            GameError::new(GameErrorCode::SafeZoneBlocked, "No puedes atacar en zona segura.")
                        } else {
                            GameError::new(GameErrorCode::TargetNotFound, "No hay enemigo en rango de hechizo")
                        };
                        self.send_to_client(sink, err.to_console_packet()).await?;
                    }
                }
            }
        }

        Ok(())
    }

    fn get_player_weapon_info(&self, scene: &Scene, entity_id: u32) -> (i32, i32, i32, i32, bool, bool) {
        let gd = self.world.gd();
        let weapon_slot = scene.players.get(&entity_id)
            .map(|p| p.id_weapon)
            .unwrap_or(0);
        if weapon_slot <= 0 {
            return (0, 0, 0, 0, false, false);
        }
        let char_id = scene.players.get(&entity_id)
            .map(|p| p.character_id.clone())
            .unwrap_or_default();
        let inv = self.world.cache_get_inventory(&char_id);
        let weapon_item = inv.iter().find(|r| r.equipped && gd.get_object(r.item_id)
            .map(|it| it.obj_type == 1 || it.obj_type == 2 || it.obj_type == 15)
            .unwrap_or(false));
        if let Some(wi) = weapon_item {
            let item = gd.get_object(wi.item_id);
            let (w_min, w_max) = item.map(|it| (it.min_hit, it.max_hit)).unwrap_or((0, 0));
            let is_proj = item.map(|it| it.obj_type == 2).unwrap_or(false);
            let is_apu = item.map(|it| it.obj_type == 15).unwrap_or(false);
            (w_min, w_max, 0, 0, is_proj, is_apu)
        } else {
            (0, 0, 0, 0, false, false)
        }
    }

    fn get_equipped_weapon_item_id(&self, scene: &Scene, entity_id: u32) -> i32 {
        let gd = self.world.gd();
        let char_id = scene.players.get(&entity_id)
            .map(|p| p.character_id.clone())
            .unwrap_or_default();
        let inv = self.world.cache_get_inventory(&char_id);
        inv.iter()
            .find(|r| r.equipped && gd.get_object(r.item_id)
                .map(|it| it.obj_type == 1 || it.obj_type == 2 || it.obj_type == 15)
                .unwrap_or(false))
            .map(|wi| wi.item_id)
            .unwrap_or(0)
    }

    fn get_player_magic_item_stats(&self, scene: &Scene, entity_id: u32) -> (i32, i32, i32, i32) {
        let gd = self.world.gd();
        let char_id = scene.players.get(&entity_id)
            .map(|p| p.character_id.clone())
            .unwrap_or_default();
        let inv = self.world.cache_get_inventory(&char_id);
        let mut wep_bonus = 0i32;
        let mut wep_pen = 0i32;
        let mut ring_bonus = 0i32;
        let mut ring_pen = 0i32;
        for r in inv.iter() {
            if !r.equipped { continue; }
            if let Some(obj) = gd.get_object(r.item_id) {
                if obj.obj_type == 1 || obj.obj_type == 2 || obj.obj_type == 15 {
                    wep_bonus = obj.magic_damage_bonus;
                    wep_pen = obj.magic_penetration;
                } else if obj.obj_type == 12 {
                    ring_bonus = obj.magic_damage_bonus;
                    ring_pen = obj.magic_penetration;
                }
            }
        }
        (wep_bonus, wep_pen, ring_bonus, ring_pen)
    }

    fn get_player_item_magic_resistance(&self, _scene: &Scene, entity_id: u32) -> i32 {
        let gd = self.world.gd();
        let scene = self.world.get_or_create_scene(self.map_id.unwrap_or(1));
        let char_id = scene.players.get(&entity_id)
            .map(|p| p.character_id.clone())
            .unwrap_or_default();
        let inv = self.world.cache_get_inventory(&char_id);
        let mut total = 0i32;
        for r in inv.iter() {
            if !r.equipped { continue; }
            if let Some(obj) = gd.get_object(r.item_id) {
                if obj.obj_type == 3 || obj.obj_type == 4 || obj.obj_type == 5 || obj.obj_type == 12 {
                    total += obj.resistencia_magica;
                }
            }
        }
        total
    }

    fn calc_full_melee_damage(&self, scene: &Scene, entity_id: u32) -> i32 {
        use crate::gameplay::combat_formulas::*;
        let p = match scene.players.get(&entity_id) {
            Some(p) => p,
            None => return 1,
        };
        let (w_min, w_max, a_min, a_max, is_proj, _is_apu) = self.get_player_weapon_info(scene, entity_id);
        calcular_dmg(
            p.min_hit, p.max_hit, p.attr_fuerza, p.id_clase,
            w_min, w_max, a_min, a_max, is_proj,
        ).max(1)
    }

    fn do_melee_vs_npc(&self, scene: &Scene, entity_id: u32, npc_id: u32) -> (bool, i32, bool) {
        use crate::gameplay::combat_formulas::*;
        let p = match scene.players.get(&entity_id) {
            Some(p) => p,
            None => return (false, 0, false),
        };
        let npc = match scene.npcs.get(&npc_id) {
            Some(n) => n,
            None => return (false, 0, false),
        };
        let (_w_min, w_max, _a_min, _a_max, is_proj, is_apu) =
            self.get_player_weapon_info(scene, entity_id);
        let wt = if w_max <= 0 { WeaponType::Unarmed }
            else if is_proj { WeaponType::Projectile }
            else if is_apu { WeaponType::Stabbing }
            else { WeaponType::Melee };
        let atk_power = poder_ataque_arma(p.level, p.attr_agilidad, p.id_clase, wt);
        let npc_ev = npc_evasion(npc.exp_reward.min(50));
        if !roll_melee_hit(atk_power, npc_ev) {
            return (false, 0, false);
        }

        let equipped_weapon_item_id = self.get_equipped_weapon_item_id(scene, entity_id);
        let ds_hit = is_dragon_slayer_hit(equipped_weapon_item_id, npc.npc_type, NPC_TYPE_DRAGON);

        if ds_hit {
            let dmg = npc.hp.max(1);
            return (true, dmg, true);
        }

        let dmg = self.calc_full_melee_damage(scene, entity_id);
        (true, dmg, false)
    }

    fn do_melee_vs_player(&self, scene: &Scene, attacker_id: u32, target_id: u32) -> (bool, i32, bool) {
        use crate::gameplay::combat_formulas::*;
        let atk = match scene.players.get(&attacker_id) {
            Some(p) => p,
            None => return (false, 0, false),
        };
        let def = match scene.players.get(&target_id) {
            Some(p) => p,
            None => return (false, 0, false),
        };
        let (_w_min, w_max, _a_min, _a_max, is_proj, is_apu) =
            self.get_player_weapon_info(scene, attacker_id);
        let wt = if w_max <= 0 { WeaponType::Unarmed }
            else if is_proj { WeaponType::Projectile }
            else if is_apu { WeaponType::Stabbing }
            else { WeaponType::Melee };
        let atk_power = poder_ataque_arma(atk.level, atk.attr_agilidad, atk.id_clase, wt);
        let def_evasion = poder_evasion(def.level, def.attr_agilidad, def.id_clase);

        let shield_pct = self.get_shield_pct(scene, target_id);
        let total_evasion = def_evasion + poder_evasion_escudo(def.level, def.id_clase, shield_pct);
        drop(atk);
        drop(def);

        if !roll_melee_hit(atk_power, total_evasion) {
            let shield_blocked = shield_pct > 0 && {
                let def2 = scene.players.get(&target_id);
                def2.map(|d| roll_shield_block(d.level, d.id_clase, shield_pct)).unwrap_or(false)
            };
            return (false, 0, shield_blocked);
        }

        let mut dmg = self.calc_full_melee_damage(scene, attacker_id);
        let body_part = random_body_part();
        let (h_min, h_max, b_min, b_max, s_min, s_max) = self.get_target_armor_defs(scene, target_id);
        let absorb = body_part_absorption(body_part, h_min, h_max, b_min, b_max, s_min, s_max);
        dmg = (dmg - absorb).max(1);
        (true, dmg, false)
    }

    fn get_shield_pct(&self, _scene: &Scene, _target_id: u32) -> i32 {
        0
    }

    fn get_target_armor_defs(&self, scene: &Scene, target_id: u32) -> (i32, i32, i32, i32, i32, i32) {
        let gd = self.world.gd();
        let char_id = scene.players.get(&target_id)
            .map(|p| p.character_id.clone())
            .unwrap_or_default();
        let inv = self.world.cache_get_inventory(&char_id);
        let mut h_min = 0i32; let mut h_max = 0i32;
        let mut b_min = 0i32; let mut b_max = 0i32;
        let mut s_min = 0i32; let mut s_max = 0i32;
        for r in inv.iter() {
            if !r.equipped { continue; }
            if let Some(it) = gd.get_object(r.item_id) {
                match it.obj_type {
                    3 => { h_min = it.min_def; h_max = it.max_def; } // helmet
                    4 => { b_min = it.min_def; b_max = it.max_def; } // body armor
                    5 => { s_min = it.min_def; s_max = it.max_def; } // shield
                    _ => {}
                }
            }
        }
        (h_min, h_max, b_min, b_max, s_min, s_max)
    }

    pub(super) async fn grant_xp(
        &self,
        entity_id: u32,
        xp_amount: i32,
        scene: &Scene,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let double = self.world.double_exp.load(std::sync::atomic::Ordering::Relaxed);
        let base_xp = if double { xp_amount * 2 } else { xp_amount };

        let party_members = self.get_party_recipients(entity_id, scene);

        if party_members.len() > 1 {
            let share = (base_xp / party_members.len() as i32).max(1);
            let bonus_share = (share as f64 * 1.15) as i32;

            for &member_id in &party_members {
                Self::apply_xp_to_player(member_id, bonus_share, scene);
                if member_id == entity_id {
                    let msg = format!("+{} EXP (party)", bonus_share);
                    self.send_to_client(sink, build_console_message(&msg)).await?;
                } else if let Some(tx) = scene.personal_tx.get(&member_id) {
                    let msg = format!("+{} EXP (party)", bonus_share);
                    let _ = tx.send(build_console_message(&msg));
                }
                Self::check_level_up_and_notify(member_id, scene, &self.world);
                if member_id == entity_id {
                    if let Some(p) = scene.players.get(&entity_id) {
                        let exp_pkt = build_act_exp(p.exp, p.exp_next_level);
                        self.send_to_client(sink, exp_pkt).await?;
                    }
                } else if let Some(p) = scene.players.get(&member_id)
                    && let Some(tx) = scene.personal_tx.get(&member_id) {
                        let _ = tx.send(build_act_exp(p.exp, p.exp_next_level));
                }
            }
        } else {
            Self::apply_xp_to_player(entity_id, base_xp, scene);
            let msg = format!("+{} EXP", base_xp);
            self.send_to_client(sink, build_console_message(&msg)).await?;
            Self::check_level_up_and_notify(entity_id, scene, &self.world);
            if let Some(p) = scene.players.get(&entity_id) {
                let exp_pkt = build_act_exp(p.exp, p.exp_next_level);
                self.send_to_client(sink, exp_pkt).await?;
            }
        }
        Ok(())
    }

    fn get_party_recipients(&self, entity_id: u32, scene: &Scene) -> Vec<u32> {
        let party_id = scene.players.get(&entity_id).and_then(|p| p.party_id.clone());
        let party_id = match party_id {
            Some(pid) => pid,
            None => return vec![entity_id],
        };

        let attacker_pos = scene.players.get(&entity_id).map(|p| (p.pos.x, p.pos.y));
        let (ax, ay) = match attacker_pos {
            Some(pos) => pos,
            None => return vec![entity_id],
        };

        let mut recipients = Vec::new();
        for entry in scene.players.iter() {
            let p = entry.value();
            if p.party_id.as_deref() == Some(&party_id) && !p.dead {
                let dx = (p.pos.x - ax).abs();
                let dy = (p.pos.y - ay).abs();
                if dx <= 12 && dy <= 12 {
                    recipients.push(p.id);
                }
            }
        }
        if recipients.is_empty() {
            vec![entity_id]
        } else {
            recipients
        }
    }

    fn apply_xp_to_player(entity_id: u32, xp: i32, scene: &Scene) {
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.exp += xp;
        }
    }

    fn check_level_up_and_notify(entity_id: u32, scene: &Scene, world: &crate::world::GameWorld) {
        let mut leveled = false;
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            while player.exp >= player.exp_next_level && player.exp_next_level > 0 {
                player.exp -= player.exp_next_level;
                player.level += 1;
                let (new_max_hp, new_max_mana, new_min_hit, new_max_hit) =
                    crate::gameplay::balance::recalc_on_level_up(
                        player.id_clase,
                        player.level,
                        player.attr_constitucion,
                        player.attr_inteligencia,
                    );
                player.max_hp = new_max_hp;
                player.max_mana = new_max_mana;
                player.min_hit = new_min_hit;
                player.max_hit = new_max_hit;
                player.hp = player.max_hp;
                player.mana = player.max_mana;
                player.exp_next_level = crate::gameplay::balance::get_legacy_exp_next_level(player.level);
                leveled = true;
            }
        }
        if leveled {
            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                let defs = crate::gameplay::achievements::default_achievements();
                let level = p.level as u32;
                let gold = p.gold;
                let unlocked_ids = p.achievements.check_and_unlock(&defs, level, gold);
                let names: Vec<String> = unlocked_ids.iter().filter_map(|id| {
                    defs.iter().find(|d| d.id == *id).map(|d| d.name.clone())
                }).collect();
                drop(p);
                if let Some(tx) = scene.personal_tx.get(&entity_id) {
                    for name in names {
                        let _ = tx.send(build_console_message(&format!("Logro desbloqueado: {}", name)));
                    }
                }
            }
            if let Some(p) = scene.players.get(&entity_id)
                && let Some(tx) = scene.personal_tx.get(&entity_id) {
                    let _ = tx.send(build_act_level(p.level));
                    let _ = tx.send(build_self_vitals(p.hp, p.max_hp, p.mana, p.max_mana));
                    let msg = format!("Has subido al nivel {}!", p.level);
                    tracing::info!(
                        target: "activity",
                        category = "progression", action = "level_up",
                        player = %p.name, entity = entity_id,
                        level = p.level,
                        "LEVEL_UP"
                    );
                    let _ = tx.send(build_console_message(&msg));
                }

            // Advance reach_level quest objectives
            if let Some(p) = scene.players.get(&entity_id) {
                let map_id = p.pos.map;
                let level = p.level as u32;
                drop(p);
                super::GameSession::advance_quest_level(entity_id, level, world, map_id);
            }

            let should_strip_newbie = scene.players.get(&entity_id)
                .map(|p| p.level == crate::gameplay::combat_formulas::NEWBIE_MAX_LEVEL + 1)
                .unwrap_or(false);
            if should_strip_newbie {
                let char_id = scene.players.get(&entity_id).map(|p| p.character_id.clone());
                if let Some(char_id) = char_id {
                    Self::strip_newbie_items(entity_id, &char_id, scene, world);
                }
            }
        }
    }

    fn strip_newbie_items(entity_id: u32, character_id: &str, scene: &Scene, world: &crate::world::GameWorld) {
        let gd = world.gd();
        let inv = world.cache_get_inventory(character_id);
        let mut removed_any = false;
        let mut visual_changed_weapon = false;
        let mut visual_changed_body = false;
        let mut visual_changed_helmet = false;
        let mut visual_changed_shield = false;

        for row in &inv {
            let obj = match gd.get_object(row.item_id) {
                Some(o) => o,
                None => continue,
            };
            if obj.newbie == 0 { continue; }

            if row.equipped {
                match obj.obj_type {
                    2 => {
                        if let Some(mut p) = scene.players.get_mut(&entity_id) { p.id_weapon = 0; }
                        visual_changed_weapon = true;
                    }
                    3 => {
                        if let Some(mut p) = scene.players.get_mut(&entity_id) { p.id_body = 0; }
                        visual_changed_body = true;
                    }
                    4 => {
                        if let Some(mut p) = scene.players.get_mut(&entity_id) { p.id_helmet = 0; }
                        visual_changed_helmet = true;
                    }
                    8 => {
                        if let Some(mut p) = scene.players.get_mut(&entity_id) { p.id_shield = 0; }
                        visual_changed_shield = true;
                    }
                    _ => {}
                }
            }

            world.cache_delete_slot(character_id, row.slot);
            removed_any = true;
        }

        if !removed_any { return; }

        if let Some(tx) = scene.personal_tx.get(&entity_id) {
            let _ = tx.send(build_console_message(
                "Al llegar a nivel 13 dejaste de ser newbie, se eliminaron tus objetos newbie del inventario."
            ));

            let refreshed_inv = world.cache_get_inventory(character_id);
            for slot_idx in 0..20i32 {
                let row = refreshed_inv.iter().find(|r| r.slot == slot_idx);
                match row {
                    Some(r) => {
                        let item_data = crate::replication::get_item_data(&gd, r.item_id);
                        let pkt_row = crate::persistence::InventoryRow {
                            slot: r.slot, item_id: r.item_id, amount: r.amount, equipped: r.equipped,
                        };
                        let _ = tx.send(crate::replication::build_inv_item_packet(&pkt_row, &item_data));
                    }
                    None => {
                        let empty_row = crate::persistence::InventoryRow {
                            slot: slot_idx, item_id: 0, amount: 0, equipped: false,
                        };
                        let item_data = crate::replication::get_item_data(&gd, 0);
                        let _ = tx.send(crate::replication::build_inv_item_packet(&empty_row, &item_data));
                    }
                }
            }
        }

        let player_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
        if let Some(ref pos) = player_pos {
            if visual_changed_weapon {
                scene.broadcast_in_range(entity_id, pos, build_change_equipment(client_packet_id::CHANGE_WEAPON, entity_id, 0));
            }
            if visual_changed_body {
                scene.broadcast_in_range(entity_id, pos, build_change_equipment(client_packet_id::CHANGE_BODY, entity_id, 0));
            }
            if visual_changed_helmet {
                scene.broadcast_in_range(entity_id, pos, build_change_equipment(client_packet_id::CHANGE_HELMET, entity_id, 0));
            }
            if visual_changed_shield {
                scene.broadcast_in_range(entity_id, pos, build_change_equipment(client_packet_id::CHANGE_SHIELD, entity_id, 0));
            }
        }
    }

    async fn grant_npc_gold(
        &self,
        entity_id: u32,
        npc_type: i32,
        scene: &Scene,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let raw_gold = self.world.gd()
            .get_npc(npc_type)
            .map(|t| t.gold)
            .unwrap_or(0);
        let npc_gold = raw_gold * crate::gameplay::combat_formulas::NPC_GOLD_MULTIPLIER;

        let double_gold = self.world.double_gold.load(std::sync::atomic::Ordering::Relaxed);
        let mut actual_gold = if double_gold { npc_gold * 2 } else { npc_gold };

        let mid = self.map_id.unwrap_or(0);
        if let Ok(tm) = self.world.territories.try_lock()
            && let Some(t) = tm.get_territory_for_map(mid)
                && let Some(ref owner) = t.owner_clan
                    && let Some(p) = scene.players.get(&entity_id)
                        && p.clan_id.as_deref() == Some(owner.as_str()) {
                            actual_gold = (actual_gold as f64 * (1.0 + t.bonus_gold_pct as f64 / 100.0)) as i32;
                        }

        if actual_gold > 0
            && let Some(mut player) = scene.players.get_mut(&entity_id)
        {
            player.gold = crate::gameplay::balance::clamp_gold((player.gold + actual_gold) as i64) as i32;
            let msg = format!("+{} oro", actual_gold);
            self.send_to_client(sink, build_console_message(&msg))
                .await?;
        }
        Ok(())
    }

    pub(super) fn drop_npc_loot(&self, scene: &Scene, npc_type: i32, center: &crate::world::Position) {
        let loot = crate::replication::get_npc_loot(&self.world.gd(), npc_type);
        let mut rng = rand::rng();
        for (item_id, amount, grh_index) in loot {
            let drop_x =
                (center.x + rand::Rng::random_range(&mut rng, -1..=1i32)).clamp(1, 100);
            let drop_y =
                (center.y + rand::Rng::random_range(&mut rng, -1..=1i32)).clamp(1, 100);

            let ground_item = crate::world::GroundItem {
                x: drop_x,
                y: drop_y,
                item_id,
                amount,
                grh_index,
                dropped_at_ms: self.world.uptime_ms(),
            };
            scene
                .ground_items
                .insert((drop_x, drop_y), ground_item);

            let pkt = crate::replication::build_render_item(
                drop_x, drop_y, item_id, amount, grh_index,
            );
            scene.broadcast_in_range(0, center, pkt);
        }
    }

    fn find_character_id_for_entity(&self, entity_id: u32) -> Option<String> {
        for scene_ref in self.world.scenes.iter() {
            if let Some(p) = scene_ref.players.get(&entity_id) {
                return Some(p.character_id.clone());
            }
        }
        None
    }

    async fn drop_items_on_death(
        &self,
        char_id: &str,
        death_pos: &crate::world::Position,
        scene: &Scene,
    ) {
        let base_x = death_pos.x;
        let base_y = death_pos.y;
        let inv = self.world.cache_get_inventory(char_id);

        let drop_positions: Vec<(i32, i32)> = {
            let mut rng = rand::rng();
            inv.iter()
                .map(|_| {
                    let dx = rand::Rng::random_range(&mut rng, -2..=2i32);
                    let dy = rand::Rng::random_range(&mut rng, -2..=2i32);
                    ((base_x + dx).clamp(1, 100), (base_y + dy).clamp(1, 100))
                })
                .collect()
        };

        let target_entity = self.find_entity_by_char_id(char_id).unwrap_or(0);

        for (i, row) in inv.iter().enumerate() {
            if row.equipped {
                continue;
            }
            let item_data = crate::replication::get_item_data(&self.world.gd(), row.item_id);
            if item_data.newbie || item_data.no_drop {
                continue;
            }
            let (drop_x, drop_y) = drop_positions[i];

            let ground_item = crate::world::GroundItem {
                x: drop_x,
                y: drop_y,
                item_id: row.item_id,
                amount: row.amount,
                grh_index: item_data.grh_index,
                dropped_at_ms: self.world.uptime_ms(),
            };
            scene.ground_items.insert((drop_x, drop_y), ground_item);

            let pkt = crate::replication::build_render_item(
                drop_x, drop_y, row.item_id, row.amount, item_data.grh_index,
            );
            scene.broadcast_in_range(0, death_pos, pkt);

            self.world.cache_delete_slot(char_id, row.slot);

            let remove_pkt = {
                use openao_protocol::PacketWriter;
                use openao_protocol::opcodes::client_packet_id;
                let mut w = PacketWriter::with_packet_id(client_packet_id::QUITAR_USER_INV_ITEM);
                w.write_byte(row.slot as u8);
                w.into_bytes()
            };
            scene.send_to_player(target_entity, remove_pkt);
        }
    }

    fn find_entity_by_char_id(&self, char_id: &str) -> Option<u32> {
        for scene_ref in self.world.scenes.iter() {
            for entry in scene_ref.players.iter() {
                if entry.value().character_id == char_id {
                    return Some(*entry.key());
                }
            }
        }
        None
    }

    async fn handle_summon_spell(
        &mut self,
        entity_id: EntityId,
        map_id: i32,
        _spell_id: u16,
        summon_npc_index: i32,
        spell: &crate::replication::SpellData,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let gd = self.world.gd();
        let template = match gd.get_npc(summon_npc_index) {
            Some(t) => t,
            None => {
                self.send_to_client(sink, build_console_message("NPC de invocación no encontrado.")).await?;
                return Ok(());
            }
        };

        let scene = self.world.get_or_create_scene(map_id);

        let (player_pos, player_heading) = match scene.players.get(&entity_id) {
            Some(p) => (p.pos.clone(), p.heading),
            None => return Ok(()),
        };

        if let Some(mut caster) = scene.players.get_mut(&entity_id) {
            caster.mana -= spell.mana_cost as i32;
            let vitals = build_self_vitals(caster.hp, caster.max_hp, caster.mana, caster.max_mana);
            drop(caster);
            self.send_to_client(sink, vitals).await?;
        }

        let spawn_pos = {
            let mut found = None;
            for radius in 1..=5i32 {
                for dx in -radius..=radius {
                    for dy in -radius..=radius {
                        if dx.abs() != radius && dy.abs() != radius { continue; }
                        let sx = player_pos.x + dx;
                        let sy = player_pos.y + dy;
                        if sx < 1 || sy < 1 { continue; }
                        if gd.is_blocked_tile(map_id, sx, sy) { continue; }
                        let occupied = scene.players.iter().any(|p| p.pos.map == map_id && p.pos.x == sx && p.pos.y == sy && !p.dead)
                            || scene.npcs.iter().any(|n| n.pos.map == map_id && n.pos.x == sx && n.pos.y == sy && !n.dead);
                        if !occupied {
                            found = Some(Position { map: map_id, x: sx, y: sy });
                            break;
                        }
                    }
                    if found.is_some() { break; }
                }
                if found.is_some() { break; }
            }
            found
        };

        let spawn_pos = match spawn_pos {
            Some(p) => p,
            None => {
                self.send_to_client(sink, build_console_message("No hay espacio para invocar.")).await?;
                return Ok(());
            }
        };

        {
            let mut summon_ids = scene.players.get(&entity_id).map(|p| p.summons.clone()).unwrap_or_default();
            summon_ids.retain(|sid| scene.npcs.get(sid).map(|n| n.summoned_by == Some(entity_id) && !n.dead).unwrap_or(false));

            while summon_ids.len() >= MAX_SUMMONS_PER_USER {
                let oldest = summon_ids.remove(0);
                if let Some(old_npc) = scene.npcs.get(&oldest) {
                    let old_pos = old_npc.pos.clone();
                    drop(old_npc);
                    scene.npcs.remove(&oldest);
                    scene.aoi_remove(oldest);
                    let del_pkt = crate::replication::build_delete_character_packet(oldest);
                    scene.broadcast_in_range(0, &old_pos, del_pkt);
                }
            }

            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                p.summons = summon_ids;
            }
        }

        let npc_id = self.world.next_id();
        let now_ms = self.world.uptime_ms();

        let npc_spells: Vec<crate::world::NpcSpellSlot> = template.spells.iter()
            .filter(|e| e.id_spell > 0)
            .map(|e| crate::world::NpcSpellSlot { spell_id: e.id_spell })
            .collect();

        let npc_state = crate::world::NpcState {
            id: npc_id,
            npc_type: summon_npc_index,
            pos: spawn_pos.clone(),
            heading: player_heading,
            hp: template.max_hp,
            max_hp: template.max_hp,
            min_hit: template.min_hit,
            max_hit: template.max_hit,
            defense: template.def,
            exp_reward: 0,
            movement: template.movement,
            dead: false,
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
            summoned_by: Some(entity_id),
            summon_expires_at_ms: now_ms + SUMMON_DURATION_MS,
            admin_bot_owner: None,
        };

        scene.aoi_insert(npc_id, &spawn_pos);
        let pkt = crate::replication::build_npc_packet(&npc_state, &gd);
        scene.broadcast_in_range(0, &spawn_pos, pkt);
        scene.npcs.insert(npc_id, npc_state);

        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.summons.push(npc_id);
        }

        if spell.fx_id > 0 {
            let fx = crate::replication::build_anim_fx(entity_id, spell.fx_id);
            scene.broadcast_in_range(0, &player_pos, fx);
        }

        self.send_to_client(sink, build_console_message("¡Has invocado una criatura!")).await?;
        Ok(())
    }
}
