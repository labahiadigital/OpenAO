use openao_protocol::PacketWriter;
use openao_protocol::opcodes::client_packet_id;

use crate::error::{GameError, GameErrorCode, HandlerResult};

use super::packets::*;
use super::GameSession;
use crate::gateway::WsSink;

impl GameSession {
    pub(super) async fn handle_click(
        &mut self,
        x: i32,
        y: i32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let entity_id = match self.entity_id { Some(e) => e, None => return Ok(()) };

        {
            let now = self.world.uptime_ms();
            let scene = self.world.get_or_create_scene(map_id);
            if let Some(p) = scene.players.get(&entity_id) {
                if !p.action_cooldowns.can_click(now) {
                    return Ok(());
                }
            }
            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                p.action_cooldowns.trigger_click(now);
            }
        }

        if self.handle_fishing_map_click(entity_id, x, y, sink).await? {
            return Ok(());
        }
        if self.handle_harvesting_map_click(entity_id, x, y, sink).await? {
            return Ok(());
        }

        let scene = self.world.get_or_create_scene(map_id);

        for entry in scene.npcs.iter() {
            let npc = entry.value();
            if npc.pos.x == x && npc.pos.y == y && !npc.dead {
                let npc_type_id = self.world.gd().get_npc(npc.npc_type)
                    .map(|t| t.npc_type)
                    .unwrap_or(0);

                const NPC_TYPE_SACERDOTE: i32 = 1;
                const NPC_TYPE_SACERDOTE_NEWBIE: i32 = 9;
                const NPC_TYPE_BANQUERO: i32 = 4;

                if npc_type_id == NPC_TYPE_SACERDOTE || npc_type_id == NPC_TYPE_SACERDOTE_NEWBIE {
                    let is_dead = scene.players.get(&entity_id).map(|p| p.dead).unwrap_or(false);
                    if is_dead {
                        if let Some(mut player) = scene.players.get_mut(&entity_id) {
                            player.dead = false;
                            player.dead_world_active = false;
                            player.dead_world_transition_at_ms = 0;
                            player.hp = player.max_hp / 2;
                            player.mana = player.max_mana / 2;
                        }
                        let (hp, max_hp, mana, max_mana, id_head, id_body) = scene.players.get(&entity_id)
                            .map(|p| (p.hp, p.max_hp, p.mana, p.max_mana, p.id_head, p.id_body))
                            .unwrap_or((0, 0, 0, 0, 0, 0));

                        let vitals_pkt = crate::replication::build_self_vitals(hp, max_hp, mana, max_mana);
                        let entity_vitals = crate::replication::build_entity_vitals_delta(entity_id, hp, max_hp, mana, max_mana);

                        let revive_pkt = build_revivir_usuario(entity_id, id_head, id_body);
                        let priest_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                        if let Some(ref pos) = priest_pos {
                            scene.broadcast_in_range(0, pos, revive_pkt.clone());
                            scene.broadcast_in_range(entity_id, pos, entity_vitals);
                        } else {
                            scene.broadcast(0, revive_pkt.clone());
                            scene.broadcast(entity_id, entity_vitals);
                        }
                        self.send_to_client(sink, revive_pkt).await?;
                        self.send_to_client(sink, vitals_pkt).await?;
                        self.send_to_client(sink, build_console_message("El sacerdote te ha resucitado.")).await?;
                    } else {
                        let needs_heal = scene.players.get(&entity_id)
                            .map(|p| p.hp < p.max_hp)
                            .unwrap_or(false);
                        if needs_heal {
                            if let Some(mut player) = scene.players.get_mut(&entity_id) {
                                player.hp = player.max_hp;
                            }
                            let (hp, max_hp, mana, max_mana) = scene.players.get(&entity_id)
                                .map(|p| (p.hp, p.max_hp, p.mana, p.max_mana))
                                .unwrap_or((0, 0, 0, 0));
                            let vitals = crate::replication::build_self_vitals(hp, max_hp, mana, max_mana);
                            self.send_to_client(sink, vitals).await?;
                            let entity_vitals = crate::replication::build_entity_vitals_delta(entity_id, hp, max_hp, mana, max_mana);
                            let heal_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                            if let Some(ref pos) = heal_pos {
                                scene.broadcast_in_range(entity_id, pos, entity_vitals);
                            }
                            self.send_to_client(sink, build_console_message("El sacerdote te cura completamente.")).await?;
                        } else {
                            self.send_to_client(sink, build_console_message("No necesitas curación.")).await?;
                        }
                    }
                    return Ok(());
                }

                if npc_type_id == NPC_TYPE_BANQUERO {
                    let char_id = match &self.character_id {
                        Some(c) => c.clone(),
                        None => return Ok(()),
                    };
                    let bank_gold = self.world.db.get_bank_gold(&char_id).await.unwrap_or(0);
                    let bank_items = self.world.db.load_bank(&char_id).await.unwrap_or_default();
                    let msg = format!(
                        "Banco - Oro: {} | Items: {} slots ocupados.",
                        bank_gold, bank_items.len()
                    );
                    self.send_to_client(sink, build_console_message(&msg)).await?;
                    return Ok(());
                }

                const NPC_TYPE_TIMBERO: i32 = 7;
                if npc_type_id == NPC_TYPE_TIMBERO {
                    let npc_name = self.world.gd().get_npc(npc.npc_type)
                        .map(|t| t.name.clone())
                        .unwrap_or_else(|| "Mercader".to_string());
                    self.market_npc_name = Some(npc_name.clone());
                    self.open_market(&npc_name, sink).await?;
                    return Ok(());
                }

                if let Some(template) = self.world.gd().get_npc(npc.npc_type)
                    && !template.objs.is_empty()
                {
                    self.trade_npc_type = Some(npc.npc_type);

                    let mut w = PacketWriter::with_packet_id(client_packet_id::OPEN_TRADE);
                    w.write_byte(template.objs.len() as u8);
                    for shop_item in &template.objs {
                        let item_data = crate::replication::get_item_data(&self.world.gd(), shop_item.item);
                        w.write_short(shop_item.item as u16);
                        w.write_string(&item_data.name);
                        w.write_int(item_data.value);
                        w.write_short(item_data.grh_index);
                    }
                    self.send_to_client(sink, w.into_bytes()).await?;
                    return Ok(());
                }

                if let Some(template) = self.world.gd().get_npc(npc.npc_type) {
                    let mut msg = format!("Ves a {} [NPC]", template.name);
                    if template.max_hp > 0 {
                        msg += &format!(" [Vida: {}/{}]", npc.hp, npc.max_hp);
                    }
                    if let Some(desc) = &template.desc {
                        msg += &format!(" - {}", desc);
                    }
                    self.send_to_client(sink, build_console_message(&msg)).await?;
                }

                return Ok(());
            }
        }

        for entry in scene.players.iter() {
            let p = entry.value();
            if p.pos.x == x && p.pos.y == y && p.id != entity_id {
                let class_name = crate::replication::get_class_name(p.id_clase);
                let mut msg = format!("Ves a {} - {}, nivel {}", p.name, class_name, p.level);
                if p.faction != "none" {
                    let faction_display = match p.faction.as_str() {
                        "armada" => "Armada",
                        "caos" => "Caos",
                        _ => "Sin faccion",
                    };
                    msg += &format!(" - {}", faction_display);
                } else if p.criminal {
                    msg += " - Criminal";
                } else {
                    msg += " - Ciudadano";
                }
                self.send_to_client(sink, build_console_message(&msg)).await?;
                return Ok(());
            }
        }

        if let Some(gi) = scene.ground_items.get(&(x, y)) {
            let item_data = crate::replication::get_item_data(&self.world.gd(), gi.item_id);
            let msg = format!("{} - {}", item_data.name, gi.amount);
            self.send_to_client(sink, build_console_message(&msg)).await?;
            return Ok(());
        }

        Ok(())
    }

    pub(super) async fn handle_buy_item(
        &mut self,
        npc_slot: i32,
        amount: i32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id) = match (self.entity_id, self.map_id) {
            (Some(e), Some(m)) => (e, m),
            _ => return Ok(()),
        };

        if !self.command_limiter.check("buy") {
            self.send_to_client(sink, GameError::new(GameErrorCode::RateLimited, "Comercio: espera un momento.").to_console_packet()).await?;
            return Ok(());
        }

        if amount <= 0 {
            return Ok(());
        }

        let trade_npc_type = match self.trade_npc_type {
            Some(t) => t,
            None => {
                self.send_to_client(sink, GameError::new(GameErrorCode::TargetNotFound, "No estás comerciando con nadie.").to_console_packet()).await?;
                return Ok(());
            }
        };

        let gd = self.world.gd();
        let npc_template = match gd.get_npc(trade_npc_type) {
            Some(t) => t,
            None => return Ok(()),
        };

        let shop_item = match npc_template.objs.get(npc_slot as usize) {
            Some(item) => item,
            None => {
                self.send_to_client(sink, GameError::invalid_slot().to_console_packet()).await?;
                return Ok(());
            }
        };

        let item_data = crate::replication::get_item_data(&self.world.gd(), shop_item.item);
        let cost = (item_data.value as i32) * amount;

        let scene = self.world.get_or_create_scene(map_id);
        let player_gold = scene.players.get(&entity_id).map(|p| p.gold).unwrap_or(0);

        if player_gold < cost {
            let err = GameError::new(GameErrorCode::InsufficientGold, format!("Necesitas {} oro (tienes {}).", cost, player_gold));
            self.send_to_client(sink, err.to_console_packet()).await?;
            return Ok(());
        }

        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.gold = crate::gameplay::balance::clamp_gold((player.gold - cost) as i64) as i32;
        }

        let char_id = match &self.character_id {
            Some(id) => id.clone(),
            None => return Ok(()),
        };

        self.world.cache_add_item(&char_id, shop_item.item, amount);

        tracing::info!(
            target: "activity",
            category = "economy", action = "buy_item",
            player = ?self.character_name, item = %item_data.name,
            amount = amount, gold_delta = -(cost as i64),
            "BUY_ITEM"
        );
        let msg = format!("Compraste {}x {} por {} oro.", amount, item_data.name, cost);
        self.send_to_client(sink, build_console_message(&msg)).await?;

        self.send_gold_update(entity_id, &scene, sink).await?;
        self.send_full_inventory(sink).await?;

        Ok(())
    }

    pub(super) async fn handle_sell_item(
        &mut self,
        inv_slot: i32,
        amount: i32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id) = match (self.entity_id, self.map_id) {
            (Some(e), Some(m)) => (e, m),
            _ => return Ok(()),
        };

        if !self.command_limiter.check("sell") {
            self.send_to_client(sink, GameError::new(GameErrorCode::RateLimited, "Comercio: espera un momento.").to_console_packet()).await?;
            return Ok(());
        }

        if amount <= 0 {
            return Ok(());
        }

        if self.trade_npc_type.is_none() {
            self.send_to_client(sink, GameError::new(GameErrorCode::TargetNotFound, "No estás comerciando con nadie.").to_console_packet()).await?;
            return Ok(());
        }

        let char_id = match &self.character_id {
            Some(id) => id.clone(),
            None => return Ok(()),
        };

        let inv = self.world.cache_get_inventory(&char_id);
        let slot_data = match inv.iter().find(|r| r.slot == inv_slot) {
            Some(row) => row.clone(),
            None => {
                let err = GameError::new(GameErrorCode::InvalidSlot, "Slot vacío.");
                self.send_to_client(sink, err.to_console_packet()).await?;
                return Ok(());
            }
        };

        if slot_data.amount < amount {
            let err = GameError::new(GameErrorCode::InsufficientItems, "No tienes esa cantidad.");
            self.send_to_client(sink, err.to_console_packet()).await?;
            return Ok(());
        }

        let item_data = crate::replication::get_item_data(&self.world.gd(), slot_data.item_id);
        let sell_price = ((item_data.value as i32) * amount) / 2;

        self.world.cache_remove_items(&char_id, slot_data.item_id, amount);

        let scene = self.world.get_or_create_scene(map_id);
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.gold = crate::gameplay::balance::clamp_gold((player.gold + sell_price) as i64) as i32;
        }

        tracing::info!(
            target: "activity",
            category = "economy", action = "sell_item",
            player = ?self.character_name, item = %item_data.name,
            amount = amount, gold_delta = sell_price,
            "SELL_ITEM"
        );
        let msg = format!("Vendiste {}x {} por {} oro.", amount, item_data.name, sell_price);
        self.send_to_client(sink, build_console_message(&msg)).await?;

        self.send_gold_update(entity_id, &scene, sink).await?;
        self.send_full_inventory(sink).await?;

        Ok(())
    }

    async fn send_gold_update(
        &self,
        entity_id: u32,
        scene: &crate::world::Scene,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if let Some(player) = scene.players.get(&entity_id) {
            let pkt = build_act_gold(player.gold);
            self.send_to_client(sink, pkt).await?;
        }
        Ok(())
    }
}
