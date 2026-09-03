use openao_protocol::opcodes::client_packet_id;
use openao_protocol::PacketWriter;

use crate::error::{GameError, GameErrorCode, HandlerResult};
use crate::gameplay::input_queue::GameInput;

use super::packets::*;
use super::GameSession;
use crate::gateway::WsSink;

impl GameSession {
    pub(super) async fn handle_movement(
        &mut self,
        heading: u8,
        move_id: u16,
        sink: &mut WsSink,
    ) -> HandlerResult {
        tracing::debug!("handle_movement START heading={} move_id={}", heading, move_id);
        let entity_id = match self.entity_id {
            Some(id) => id,
            None => return Ok(()),
        };
        let map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };

        {
            tracing::debug!("handle_movement: getting scene for input_receivers");
            let scene = self.world.get_or_create_scene(map_id);
            tracing::debug!("handle_movement: got scene, checking input_receivers");
            if let Some(receiver_entry) = scene.input_receivers.get(&entity_id)
                && let Ok(mut receiver) = receiver_entry.try_lock() {
                    let packet = elura::gameplay::netcode::InputPacket {
                        client_tick: move_id as u64,
                        acknowledged_server_tick: 0,
                        inputs: vec![elura::gameplay::netcode::InputFrame {
                            sequence: move_id as u64,
                            target_tick: receiver.current_tick(),
                            input: GameInput::Move { heading },
                        }],
                    };
                    match receiver.receive(packet) {
                        Ok(report) => {
                            if report.accepted.is_empty() {
                                tracing::debug!("Movement input deduplicated for entity {}", entity_id);
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            tracing::debug!("Movement input rejected for entity {}: {}", entity_id, e);
                        }
                    }
                }
        }

        let mut tile_exit_target: Option<(i32, i32, i32)> = None;

        {
            let scene = self.world.get_or_create_scene(map_id);

            // Phase 1: Read player data and compute movement target (with write lock)
            let move_data = if let Some(player) = scene.players.get(&entity_id) {
                if player.paralizado || player.inmovilizado {
                    drop(player);
                    None
                } else {
                    let (dx, dy) = heading_to_delta(heading);
                    let new_x = player.pos.x + dx;
                    let new_y = player.pos.y + dy;
                    let navegando = player.navegando;
                    let logout_pending = player.logout_expires_at_ms > 0;
                    let hp = player.hp;
                    let max_hp = player.max_hp;
                    let mana = player.mana;
                    let max_mana = player.max_mana;
                    // Drop write lock BEFORE any iteration on scene.players/npcs
                    drop(player);
                    Some((new_x, new_y, navegando, logout_pending, hp, max_hp, mana, max_mana))
                }
            } else {
                None
            };

            // Phase 2: Validate movement (NO write lock held)
            if let Some((new_x, new_y, navegando, logout_pending, hp, max_hp, mana, max_mana)) = move_data {
                let (map_w, map_h) = self.world.gd().get_map_bounds(map_id);
                let in_bounds = (1..=map_w).contains(&new_x) && (1..=map_h).contains(&new_y);
                let blocked = self.world.gd().is_blocked_tile(map_id, new_x, new_y);
                let is_water_dest = self.world.gd().is_water_tile(map_id, new_x, new_y);
                let occupied = is_tile_occupied(&scene, new_x, new_y, entity_id);
                let movement_ok = in_bounds && !blocked && !occupied && if navegando {
                    is_water_dest
                } else {
                    !is_water_dest
                };

                if !movement_ok {
                    tracing::debug!(
                        entity_id, map_id, new_x, new_y,
                        in_bounds, blocked, occupied, is_water_dest, navegando,
                        "Movement rejected"
                    );
                }

                // Phase 3: Apply movement (re-acquire write lock briefly)
                if movement_ok {
                    if let Some(mut player) = scene.players.get_mut(&entity_id) {
                        if logout_pending {
                            player.logout_expires_at_ms = 0;
                        }
                        player.pos.x = new_x;
                        player.pos.y = new_y;
                        player.heading = heading;
                        drop(player);
                    }

                    if logout_pending {
                        if let Some(tx) = scene.personal_tx.get(&entity_id) {
                            let _ = tx.send(super::packets::build_console_message(
                                "[Servidor] La salida se canceló porque te moviste."
                            ));
                        }
                    }

                    scene.aoi_move(entity_id, &crate::world::Position { map: map_id, x: new_x, y: new_y });

                    crate::gateway::fishing::cancel_fishing_on_move(entity_id, &scene, &self.world);
                    crate::gateway::harvesting::cancel_harvesting_on_move(entity_id, &scene, &self.world);

                    {
                        let keep = scene.players.get(&entity_id).map(|p| super::inventory::can_keep_hidden_while_acting(&p)).unwrap_or(false);
                        if !keep {
                            let tick = self.metrics.current_tick.load(std::sync::atomic::Ordering::Relaxed);
                            super::inventory::stop_hidden_skill(entity_id, &scene, tick, 0);
                        }
                    }

                    let server_tick = self.metrics.current_tick.load(std::sync::atomic::Ordering::Relaxed) as u16;
                    let bcast_pkt = crate::replication::build_move_entity_packet_with_tick(entity_id, new_x, new_y, heading, server_tick);
                    let new_pos = crate::world::Position { map: map_id, x: new_x, y: new_y };
                    scene.broadcast_in_range(entity_id, &new_pos, bcast_pkt);

                    let self_pkt = build_act_position_with_move_id(entity_id, new_x, new_y, move_id);
                    self.send_to_client(sink, self_pkt).await?;

                    let vitals_pkt = build_self_vitals(hp, max_hp, mana, max_mana);
                    self.send_to_client(sink, vitals_pkt).await?;

                    if let Some(exit) = self.world.gd().get_tile_exit(map_id, new_x, new_y) {
                        tracing::info!(entity_id, map_id, new_x, new_y, target_map=exit.target_map, target_x=exit.target_x, target_y=exit.target_y, "Tile exit detected");
                        tile_exit_target = Some((exit.target_map, exit.target_x, exit.target_y));
                    }
                }
            }
        }

        if self.trade_npc_type.is_some() {
            self.trade_npc_type = None;
            let w = PacketWriter::with_packet_id(client_packet_id::CLOSE_FORCE);
            self.send_to_client(sink, w.into_bytes()).await?;
        }

        if let Some((target_map, target_x, target_y)) = tile_exit_target {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let blocked = {
                let scene = self.world.get_or_create_scene(map_id);
                scene.players.get(&entity_id)
                    .map(|p| p.pvp_block_until_ms > now_ms)
                    .unwrap_or(false)
            };
            if blocked {
                let err = GameError::new(GameErrorCode::PvpMapChangeBlocked, "No puedes cambiar de mapa durante combate PvP.");
                self.send_to_client(sink, err.to_console_packet()).await?;
            } else {
                self.do_teleport(entity_id, map_id, target_map, target_x, target_y, sink).await?;
            }
        }

        Ok(())
    }

    pub(super) async fn handle_change_heading(
        &mut self,
        heading: u8,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(id) => id,
            None => return Ok(()),
        };
        let map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.heading = heading;
            drop(player);

            let mut w = PacketWriter::with_packet_id(client_packet_id::CHANGE_HEADING);
            w.write_short(entity_id as u16);
            w.write_byte(heading);
            let pkt = w.into_bytes();
            self.send_to_client(sink, pkt.clone()).await?;
            let heading_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
            if let Some(ref pos) = heading_pos {
                scene.broadcast_in_range(entity_id, pos, pkt);
            } else {
                scene.broadcast(entity_id, pkt);
            }
        }

        Ok(())
    }

    pub(super) async fn handle_teleport(
        &mut self,
        cmd: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(id) => id,
            None => return Ok(()),
        };
        let old_map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };

        {
            let scene = self.world.get_or_create_scene(old_map_id);
            let is_jailed = scene.players.get(&entity_id)
                .map(|p| p.jail_until_ms > 0)
                .unwrap_or(false);
            if is_jailed {
                self.send_to_client(sink, build_console_message("No puedes teletransportarte desde la cárcel.")).await?;
                return Ok(());
            }
        }

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() < 4 {
            self.send_to_client(sink, build_console_message("Uso: /tp mapa x y")).await?;
            return Ok(());
        }

        let new_map: i32 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                self.send_to_client(sink, build_console_message("Mapa inválido")).await?;
                return Ok(());
            }
        };
        let new_x: i32 = parts[2].parse().unwrap_or(50);
        let new_y: i32 = parts[3].parse().unwrap_or(50);

        self.do_teleport(entity_id, old_map_id, new_map, new_x, new_y, sink).await?;
        self.send_to_client(sink, build_console_message(&format!("Teletransportado al mapa {} ({},{})", new_map, new_x, new_y))).await?;

        Ok(())
    }

    fn check_map_entry_denied(&self, entity_id: u32, current_map: i32, target_map: i32) -> Option<String> {
        use crate::gameplay::combat_formulas::{DRAGON_SLAYER_SWORD_ITEM_ID, CLAN_RING_MAP_ID};

        let scene = self.world.get_or_create_scene(current_map);
        let (char_id, player_level, is_admin, player_faction) = match scene.players.get(&entity_id) {
            Some(p) => (p.character_id.clone(), p.level, p.invisible, p.faction.clone()),
            None => return None,
        };

        if is_admin {
            return None;
        }

        if target_map == CLAN_RING_MAP_ID {
            let inv = self.world.cache_get_inventory(&char_id);
            if inv.iter().any(|r| r.item_id == DRAGON_SLAYER_SWORD_ITEM_ID) {
                return Some("No puedes entrar a la arena con una Espada Mata Dragones en el inventario.".to_string());
            }
        }

        if let Some(meta) = self.world.gd().get_map_meta(target_map) {
            let min_lv = meta.min_level;
            let max_lv = meta.max_level;
            let map_name = if meta.name.is_empty() { "este lugar".to_string() } else { meta.name.clone() };

            if min_lv > 0 && player_level < min_lv {
                return if max_lv > 0 {
                    Some(format!("Solo los personajes desde nivel {} hasta nivel {} pueden entrar a {}", min_lv, max_lv, map_name))
                } else {
                    Some(format!("Solo los personajes desde nivel {} pueden entrar a {}", min_lv, map_name))
                };
            }
            if max_lv > 0 && player_level > max_lv {
                return if min_lv > 0 {
                    Some(format!("Solo los personajes desde nivel {} hasta nivel {} pueden entrar a {}", min_lv, max_lv, map_name))
                } else {
                    Some(format!("Solo los personajes hasta nivel {} pueden entrar a {}", max_lv, map_name))
                };
            }
        }

        if let Some(denied) = check_faction_portal_denied(target_map, &player_faction) {
            return Some(denied);
        }

        None
    }

    pub(super) async fn do_teleport(
        &mut self,
        entity_id: u32,
        old_map_id: i32,
        new_map: i32,
        new_x: i32,
        new_y: i32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if let Some(denied) = self.check_map_entry_denied(entity_id, old_map_id, new_map) {
            self.send_to_client(sink, super::packets::build_console_message(&denied)).await?;
            return Ok(());
        }

        let removed_player = {
            let old_scene = self.world.get_or_create_scene(old_map_id);
            let old_pos = old_scene.players.get(&entity_id).map(|p| p.pos.clone());
            if let Some((_, player_state)) = old_scene.players.remove(&entity_id) {
                let del_pkt = crate::replication::build_delete_character_packet(entity_id);
                if let Some(ref pos) = old_pos {
                    old_scene.broadcast_in_range(entity_id, pos, del_pkt);
                } else {
                    old_scene.broadcast(entity_id, del_pkt);
                }
                old_scene.aoi_remove(entity_id);
                old_scene.remove_replicator(entity_id);
                old_scene.remove_input_receiver(entity_id);
                if self.personal_tx.is_some() {
                    old_scene.personal_tx.remove(&entity_id);
                    old_scene.outbound_pressure.remove(&entity_id);
                }
                Some(player_state)
            } else {
                None
            }
        };

        if let Some(player_state) = removed_player {
            let new_scene = self.world.get_or_create_scene(new_map);
            let mut moved = player_state;
            moved.pos.map = new_map;
            moved.pos.x = new_x;
            moved.pos.y = new_y;

            let announce = crate::replication::build_character_packet(&moved);
            let color = get_name_color(moved.criminal, &moved.faction, false);
            let equip_body = moved.id_body;
            let equip_weapon = moved.id_weapon;
            let equip_helmet = moved.id_helmet;
            let equip_shield = moved.id_shield;
            let tp_pos = moved.pos.clone();
            new_scene.broadcast_in_range(entity_id, &tp_pos, announce);
            new_scene.broadcast_in_range(entity_id, &tp_pos, build_act_color_name(entity_id, color));
            if equip_body > 0 {
                new_scene.broadcast_in_range(entity_id, &tp_pos, build_change_equipment(client_packet_id::CHANGE_BODY, entity_id, equip_body));
            }
            if equip_weapon > 0 {
                new_scene.broadcast_in_range(entity_id, &tp_pos, build_change_equipment(client_packet_id::CHANGE_WEAPON, entity_id, equip_weapon));
            }
            if equip_helmet > 0 {
                new_scene.broadcast_in_range(entity_id, &tp_pos, build_change_equipment(client_packet_id::CHANGE_HELMET, entity_id, equip_helmet));
            }
            if equip_shield > 0 {
                new_scene.broadcast_in_range(entity_id, &tp_pos, build_change_equipment(client_packet_id::CHANGE_SHIELD, entity_id, equip_shield));
            }
            new_scene.mark_entity_broadcast_announced(entity_id, &tp_pos);

            new_scene.aoi_insert(entity_id, &moved.pos);
            new_scene.players.insert(entity_id, moved);
            new_scene.add_replicator(entity_id);
            new_scene.add_input_receiver(entity_id);
            if let Some(ref ptx) = self.personal_tx {
                new_scene.personal_tx.insert(entity_id, ptx.clone());
                new_scene.outbound_pressure.insert(entity_id, std::sync::atomic::AtomicU32::new(0));
            }

            self.broadcast_rx_needs_refresh = true;
        }

        if let Some(ref cid) = self.character_id {
            self.world.active_characters.insert(cid.clone(), (entity_id, new_map, self.evicted.clone()));
        }
        self.map_id = Some(new_map);

        let mut w = PacketWriter::with_packet_id(client_packet_id::TELEP_ME);
        w.write_short(new_map as u16);
        w.write_short(new_x as u16);
        w.write_short(new_y as u16);
        w.write_byte(3);
        self.send_to_client(sink, w.into_bytes()).await?;

        let (map_name_str, zone_safe) = {
            let gd = self.world.gd();
            let meta = gd.get_map_meta(new_map);
            (
                meta.map(|m| m.name.clone()).unwrap_or_else(|| "Desconocido".to_string()),
                meta.map(|m| m.pk).unwrap_or(0),
            )
        };
        let name_pkt = build_name_map(&map_name_str);
        self.send_to_client(sink, name_pkt).await?;
        self.send_to_client(sink, build_self_map_meta_delta(&map_name_str)).await?;

        let flags_pkt = build_self_flags(zone_safe as u8, false, false);
        self.send_to_client(sink, flags_pkt).await?;

        let new_scene = self.world.get_or_create_scene(new_map);
        let tp_dest = crate::world::Position { map: new_map, x: new_x, y: new_y };
        let my_dead_world = new_scene.players.get(&entity_id)
            .map(|p| p.dead_world_active)
            .unwrap_or(false);
        let my_party = new_scene.players.get(&entity_id).and_then(|p| p.party_id.clone());
        let my_clan = new_scene.players.get(&entity_id).and_then(|p| p.clan_id.clone());
        let nearby_eids = new_scene.entities_in_range(&tp_dest);
        for &eid in &nearby_eids {
            if eid == entity_id { continue; }
            if let Some(other) = new_scene.players.get(&eid) {
                if !can_render_character(my_dead_world, &my_party, &my_clan, &other) { continue; }
                let pkt = crate::replication::build_character_packet(&other);
                self.send_to_client(sink, pkt).await?;
                let c = get_name_color(other.criminal, &other.faction, false);
                self.send_to_client(sink, build_act_color_name(other.id, c)).await?;
                if other.id_body > 0 {
                    self.send_to_client(sink, build_change_equipment(client_packet_id::CHANGE_BODY, other.id, other.id_body)).await?;
                }
                if other.id_weapon > 0 {
                    self.send_to_client(sink, build_change_equipment(client_packet_id::CHANGE_WEAPON, other.id, other.id_weapon)).await?;
                }
                if other.id_helmet > 0 {
                    self.send_to_client(sink, build_change_equipment(client_packet_id::CHANGE_HELMET, other.id, other.id_helmet)).await?;
                }
                if other.id_shield > 0 {
                    self.send_to_client(sink, build_change_equipment(client_packet_id::CHANGE_SHIELD, other.id, other.id_shield)).await?;
                }
            } else if let Some(npc) = new_scene.npcs.get(&eid) {
                if my_dead_world { continue; }
                let pkt = crate::replication::build_npc_packet(&npc, &self.world.gd());
                self.send_to_client(sink, pkt).await?;
            }
        }

        let vr_x = openao_protocol::constants::CLIENT_VIEW_RANGE_X;
        let vr_y = openao_protocol::constants::CLIENT_VIEW_RANGE_Y;
        for entry in new_scene.ground_items.iter() {
            let gi = entry.value();
            if (gi.x - new_x).abs() <= vr_x && (gi.y - new_y).abs() <= vr_y {
                let pkt = crate::replication::build_render_item(gi.x, gi.y, gi.item_id, gi.amount, gi.grh_index);
                self.send_to_client(sink, pkt).await?;
            }
        }

        self.advance_quest_visit_map(entity_id, new_map);

        if let Some(mut p) = new_scene.players.get_mut(&entity_id) {
            p.achievements.stats.total_maps_visited += 1;
        }

        Ok(())
    }

    pub(super) async fn handle_resync_position(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(id) => id,
            None => return Ok(()),
        };
        let map_id = match self.map_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        if let Some(player) = scene.players.get(&entity_id) {
            let pkt = build_act_position_with_move_id(entity_id, player.pos.x, player.pos.y, 0);
            self.send_to_client(sink, pkt).await?;
        }

        Ok(())
    }
}

pub(crate) fn can_render_character(
    viewer_dead_world: bool,
    viewer_party: &Option<String>,
    viewer_clan: &Option<String>,
    target: &crate::world::PlayerState,
) -> bool {
    let same_party = viewer_party.is_some() && viewer_party == &target.party_id;
    let same_clan = viewer_clan.is_some() && viewer_clan == &target.clan_id;

    if same_party || same_clan {
        return true;
    }

    if viewer_dead_world {
        return target.dead;
    }

    if target.invisible || target.hidden_skill || target.invisible_spell {
        return false;
    }

    true
}

fn is_tile_occupied(scene: &crate::world::Scene, x: i32, y: i32, self_id: u32) -> bool {
    for entry in scene.players.iter() {
        let p = entry.value();
        if p.id != self_id && p.pos.x == x && p.pos.y == y && !p.dead {
            return true;
        }
    }
    for entry in scene.npcs.iter() {
        let n = entry.value();
        if n.pos.x == x && n.pos.y == y && !n.dead {
            return true;
        }
    }
    false
}

struct FactionPortalRestriction {
    map_id: i32,
    faction: &'static str,
}

const FACTION_PORTAL_RESTRICTIONS: &[FactionPortalRestriction] = &[
    FactionPortalRestriction { map_id: 151, faction: "caos" },
    FactionPortalRestriction { map_id: 60, faction: "armada" },
];

fn check_faction_portal_denied(target_map: i32, player_faction: &str) -> Option<String> {
    for restriction in FACTION_PORTAL_RESTRICTIONS {
        if restriction.map_id == target_map {
            if player_faction == restriction.faction {
                return None;
            }
            return Some(if restriction.faction == "caos" {
                "Solo los miembros del Caos pueden usar este portal.".to_string()
            } else {
                "Solo los miembros de la Armada pueden usar este portal.".to_string()
            });
        }
    }
    None
}
