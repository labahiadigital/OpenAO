use futures_util::SinkExt;
use openao_protocol::opcodes::client_packet_id;
use openao_protocol::PacketReader;
use tracing::{debug, info, warn};

use super::packets::*;
use super::GameSession;
use crate::error::HandlerResult;
use crate::gateway::WsSink;
use crate::world::{PlayerState, Position};

impl GameSession {
    pub(super) async fn handle_connect_character(
        &mut self,
        reader: &mut PacketReader<'_>,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let ticket = reader.get_string()?;
        let _type_game = reader.get_byte()?;
        let _id_char = reader.get_short()?;
        debug!("ConnectCharacter from {}: ticket={}", self.addr, ticket);

        let character_id = if self.authenticated {
            match self.auth_character_id.clone() {
                Some(cid) => cid,
                None => {
                    warn!("ELR2 authenticated but no character_id stored for {}", self.addr);
                    return Ok(());
                }
            }
        } else {
            match self.world.db.consume_game_ticket(&ticket).await {
                Ok(Some((account_id, character_id))) => {
                    self.auth_account_id = Some(account_id);
                    character_id
                }
                Ok(None) => {
                    warn!("Invalid or expired ticket from {}", self.addr);
                    return Ok(());
                }
                Err(e) => {
                    warn!("DB error consuming ticket: {e}");
                    return Ok(());
                }
            }
        };

        let char_data = match self.world.db.load_character(&character_id).await {
            Ok(Some(data)) => data,
            Ok(None) => {
                warn!("Character {} not found", character_id);
                return Ok(());
            }
            Err(e) => {
                warn!("DB error loading character: {e}");
                return Ok(());
            }
        };

        let entity_id = self.world.next_id();
        let player = PlayerState {
            id: entity_id,
            account_id: self.auth_account_id.clone().unwrap_or_default(),
            character_id: char_data.id.clone(),
            name: char_data.name.clone(),
            client_ip: self.addr.ip().to_string(),
            pos: Position {
                map: char_data.map_id,
                x: char_data.pos_x,
                y: char_data.pos_y,
            },
            heading: 3,
            hp: char_data.hp,
            max_hp: char_data.max_hp,
            mana: char_data.mana,
            max_mana: char_data.max_mana,
            level: char_data.level,
            exp: char_data.exp,
            exp_next_level: char_data.exp_next_level,
            gold: char_data.gold,
            dead: char_data.dead,
            criminal: char_data.criminal,
            faction: char_data.faction.clone(),
            faction_rank: char_data.faction_rank,
            faction_score: char_data.faction_score,
            faction_rank_armada: 0,
            faction_score_armada: char_data.faction_score_armada,
            faction_rank_caos: 0,
            faction_score_caos: char_data.faction_score_caos,
            min_hit: char_data.min_hit,
            max_hit: char_data.max_hit,
            id_clase: char_data.id_clase,
            id_raza: char_data.id_raza,
            attr_fuerza: char_data.attr_fuerza,
            attr_agilidad: char_data.attr_agilidad,
            attr_inteligencia: char_data.attr_inteligencia,
            attr_constitucion: char_data.attr_constitucion,
            id_head: char_data.id_head,
            id_body: char_data.id_body,
            id_helmet: char_data.id_helmet,
            id_weapon: char_data.id_weapon,
            id_shield: char_data.id_shield,
            id_arrow_slot: char_data.id_arrow_slot,
            id_ring_slot: char_data.id_ring_slot,
            navegando: char_data.navegando,
            party_id: None,
            clan_id: None,
            home_map: char_data.home_map,
            home_x: char_data.home_x,
            home_y: char_data.home_y,
            pvp_block_until_ms: 0,
            revive_at_ms: 0,
            fishing: None,
            harvesting: None,
            buffs: crate::gameplay::buffs::BuffManager::new(),
            invisible: false,
            hidden_skill: false,
            hidden_skill_expire_tick: 0,
            hidden_skill_cooldown_tick: 0,
            jail_until_ms: 0,
            quest_log: self.world.db.load_quest_log(&char_data.id).await.unwrap_or_default(),
            pets: self.world.db.load_pets(&char_data.id).await.unwrap_or_default(),
            spell_cooldowns: crate::gameplay::cooldowns::CooldownManager::new(),
            achievements: self.world.db.load_achievements(&char_data.id).await.unwrap_or_default(),
            paralizado: false,
            paralizado_until_ms: 0,
            inmovilizado: false,
            inmovilizado_until_ms: 0,
            invisible_spell: false,
            invisible_spell_until_ms: 0,
            seguro_activado: false,
            seguro_clan_activado: false,
            dead_world_active: char_data.dead,
            dead_world_transition_at_ms: 0,
            logout_expires_at_ms: 0,
            logout_origin_x: 0,
            logout_origin_y: 0,
            criminales_matados: char_data.criminales_matados,
            ciudadanos_matados: char_data.ciudadanos_matados,
            meditar: false,
            action_cooldowns: crate::world::ActionCooldowns::default(),
            summons: Vec::new(),
        };

        // Evict any previous session for this character
        if let Some((_, (old_eid, old_map, old_evict_flag))) = self.world.active_characters.remove(&char_data.id) {
            warn!("Evicting previous session for character '{}' (entity {} on map {})", char_data.name, old_eid, old_map);
            old_evict_flag.store(true, std::sync::atomic::Ordering::Relaxed);
            if let Some(old_scene) = self.world.scenes.get(&old_map) {
                old_scene.players.remove(&old_eid);
                old_scene.personal_tx.remove(&old_eid);
                old_scene.outbound_pressure.remove(&old_eid);
                old_scene.replicators.remove(&old_eid);
                old_scene.input_receivers.remove(&old_eid);
                old_scene.aoi_remove(old_eid);
                let del_pkt = crate::replication::build_delete_character_packet(old_eid);
                let old_pos = crate::world::Position { map: old_map, x: 0, y: 0 };
                old_scene.broadcast_in_range(old_eid, &old_pos, del_pkt);
            }
        }

        {
            let scene = self.world.get_or_create_scene(char_data.map_id);
            scene.aoi_insert(entity_id, &player.pos);
            scene.players.insert(entity_id, player);
            scene.add_replicator(entity_id);
            scene.add_input_receiver(entity_id);
            if let Some(ref ptx) = self.personal_tx {
                scene.personal_tx.insert(entity_id, ptx.clone());
                scene.outbound_pressure.insert(entity_id, std::sync::atomic::AtomicU32::new(0));
            }
        }

        self.world.active_characters.insert(char_data.id.clone(), (entity_id, char_data.map_id, self.evicted.clone()));
        self.entity_id = Some(entity_id);
        self.map_id = Some(char_data.map_id);
        self.character_name = Some(char_data.name.clone());
        self.character_id = Some(char_data.id.clone());

        if self.world.db.is_muted(&char_data.id).await.unwrap_or(false) {
            self.world.muted_players.insert(entity_id, true);
        }

        self.send_initial_data(entity_id, &char_data, sink).await?;

        {
            let scene = self.world.get_or_create_scene(char_data.map_id);
            let Some(player_ref) = scene.players.get(&entity_id) else {
                warn!("Player {} disappeared from scene after insert (race condition)", entity_id);
                return Ok(());
            };
            let announce_pkt = crate::replication::build_character_packet(&player_ref);
            let color = get_name_color(player_ref.criminal, &player_ref.faction, false);
            let equip_body = player_ref.id_body;
            let equip_weapon = player_ref.id_weapon;
            let equip_helmet = player_ref.id_helmet;
            let equip_shield = player_ref.id_shield;
            let connect_pos = player_ref.pos.clone();
            drop(player_ref);
            scene.broadcast_in_range(entity_id, &connect_pos, announce_pkt);
            scene.broadcast_in_range(entity_id, &connect_pos, build_act_color_name(entity_id, color));
            if equip_body > 0 {
                scene.broadcast_in_range(entity_id, &connect_pos, build_change_equipment(client_packet_id::CHANGE_BODY, entity_id, equip_body));
            }
            if equip_weapon > 0 {
                scene.broadcast_in_range(entity_id, &connect_pos, build_change_equipment(client_packet_id::CHANGE_WEAPON, entity_id, equip_weapon));
            }
            if equip_helmet > 0 {
                scene.broadcast_in_range(entity_id, &connect_pos, build_change_equipment(client_packet_id::CHANGE_HELMET, entity_id, equip_helmet));
            }
            if equip_shield > 0 {
                scene.broadcast_in_range(entity_id, &connect_pos, build_change_equipment(client_packet_id::CHANGE_SHIELD, entity_id, equip_shield));
            }
            scene.mark_entity_broadcast_announced(entity_id, &connect_pos);
        }

        if self.use_elr2 {
            let token = self.reconnect_mgr.issue_token(
                crate::reconnect::ReconnectState {
                    account_id: self.auth_account_id.clone().unwrap_or_default(),
                    character_id: char_data.id.clone(),
                    character_name: char_data.name.clone(),
                    entity_id,
                    map_id: char_data.map_id,
                },
            );
            let payload = serde_json::json!({ "reconnect_token": token });
            let push = crate::elr2::Frame::push(
                crate::elr2::ROUTE_AUTHENTICATE,
                bytes::Bytes::from(serde_json::to_vec(&payload).unwrap_or_default()),
            );
            sink.send(tokio_tungstenite::tungstenite::Message::Binary(push.encode())).await?;
        }

        info!(
            "Player '{}' (entity {}) connected to map {}",
            char_data.name, entity_id, char_data.map_id
        );

        Ok(())
    }

    async fn send_initial_data(
        &self,
        entity_id: u32,
        char_data: &crate::persistence::CharacterData,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let scene = self.world.get_or_create_scene(char_data.map_id);

        self.world.cache_load_inventory(&char_data.id).await.ok();
        let inv_rows = self.world.cache_get_inventory(&char_data.id);

        // Build all initial state packets into a batch for a single flush
        let mut batch: Vec<Vec<u8>> = Vec::with_capacity(32);

        let Some(my_player_ref) = scene.players.get(&entity_id) else {
            warn!("Player {} disappeared from scene during initial data send", entity_id);
            return Ok(());
        };
        batch.push(crate::replication::build_my_character_packet(&my_player_ref));
        drop(my_player_ref);

        if let Some(player) = scene.players.get(&entity_id) {
            batch.push(build_self_vitals(player.hp, player.max_hp, player.mana, player.max_mana));
            batch.push(build_act_gold(player.gold));
            batch.push(build_act_exp(player.exp, player.exp_next_level));
            batch.push(build_self_attributes(
                player.attr_fuerza, player.attr_agilidad,
                player.attr_inteligencia, player.attr_constitucion,
                player.min_hit, player.max_hit,
            ));

            let pk = self.world.gd().maps_meta.get(&char_data.map_id)
                .map(|m| m.pk).unwrap_or(0);
            let (seg, seg_clan) = scene.players.get(&entity_id)
                .map(|p| (p.seguro_activado, p.seguro_clan_activado))
                .unwrap_or((false, false));
            batch.push(build_self_flags(pk as u8, seg, seg_clan));

            let color = get_name_color(player.criminal, &player.faction, false);
            batch.push(build_act_color_name(entity_id, color));

            batch.push(build_change_equipment(client_packet_id::CHANGE_ROPA, entity_id, player.id_head));
            batch.push(build_change_equipment(client_packet_id::CHANGE_BODY, entity_id, player.id_body));
            batch.push(build_change_equipment(client_packet_id::CHANGE_HELMET, entity_id, player.id_helmet));
            batch.push(build_change_equipment(client_packet_id::CHANGE_WEAPON, entity_id, player.id_weapon));
            batch.push(build_change_equipment(client_packet_id::CHANGE_SHIELD, entity_id, player.id_shield));
        }

        for row in &inv_rows {
            let item_data = crate::replication::get_item_data(&self.world.gd(), row.item_id);
            batch.push(crate::replication::build_inv_item_packet(row, &item_data));
        }

        if char_data.max_mana > 0 {
            for &(slot, spell_id) in &crate::replication::DEFAULT_SPELLS {
                let spell = crate::replication::get_spell_data(&self.world.gd(), spell_id);
                batch.push(crate::replication::build_learn_spell(slot, spell_id, &spell.name, spell.mana_cost));
            }
        }

        let gd = self.world.gd();
        let map_name = gd.maps_meta.get(&char_data.map_id)
            .map(|m| m.name.as_str())
            .unwrap_or("Desconocido");
        batch.push(build_name_map(map_name));
        batch.push(build_self_map_meta_delta(map_name));

        tracing::info!(
            target: "activity",
            category = "session", action = "character_connect",
            player = %char_data.name, map = char_data.map_id,
            level = char_data.level, ip = %self.addr,
            "CHARACTER_CONNECT"
        );
        let welcome = format!(
            "Bienvenido a OpenAO, {}! (Servidor Rust, mapa {})",
            char_data.name, char_data.map_id
        );
        batch.push(build_console_message(&welcome));

        // Flush the initial state batch in one go
        self.send_batch_to_client(sink, batch).await?;

        // Nearby entities — sent as a second batch
        let my_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
        let nearby_eids = my_pos.as_ref()
            .map(|pos| scene.entities_in_range(pos))
            .unwrap_or_default();

        let (my_dead_world, my_party, my_clan) = scene.players.get(&entity_id)
            .map(|p| (p.dead_world_active, p.party_id.clone(), p.clan_id.clone()))
            .unwrap_or((false, None, None));

        let mut entity_batch: Vec<Vec<u8>> = Vec::with_capacity(nearby_eids.len());
        for &eid in &nearby_eids {
            if eid == entity_id { continue; }
            if let Some(other) = scene.players.get(&eid) {
                if !crate::gateway::movement::can_render_character(my_dead_world, &my_party, &my_clan, &other) { continue; }
                entity_batch.push(crate::replication::build_character_packet(&other));
            } else if let Some(npc) = scene.npcs.get(&eid) {
                if my_dead_world { continue; }
                entity_batch.push(crate::replication::build_npc_packet(&npc, &self.world.gd()));
            }
        }

        if let Some(ref pos) = my_pos {
            let vr_x = openao_protocol::constants::CLIENT_VIEW_RANGE_X;
            let vr_y = openao_protocol::constants::CLIENT_VIEW_RANGE_Y;
            for entry in scene.ground_items.iter() {
                let gi = entry.value();
                if (gi.x - pos.x).abs() <= vr_x && (gi.y - pos.y).abs() <= vr_y {
                    entity_batch.push(crate::replication::build_render_item(gi.x, gi.y, gi.item_id, gi.amount, gi.grh_index));
                }
            }
        }

        // Drop the scene Ref before iterating all scenes to avoid DashMap shard contention
        drop(scene);

        let online_count: u16 = self.world.scenes.iter()
            .map(|s| s.players.len() as u16)
            .sum();
        entity_batch.push(build_act_online(online_count));

        self.send_batch_to_client(sink, entity_batch).await?;

        Ok(())
    }
}
