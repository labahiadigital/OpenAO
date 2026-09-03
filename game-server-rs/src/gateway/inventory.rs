use openao_protocol::opcodes::client_packet_id;
use openao_protocol::PacketWriter;

use super::packets::*;
use super::GameSession;
use crate::error::{GameError, GameErrorCode, HandlerResult};
use crate::gateway::WsSink;

impl GameSession {
    pub(super) async fn handle_agarrar_item(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id, char_id) = match (self.entity_id, self.map_id, &self.character_id) {
            (Some(e), Some(m), Some(c)) => (e, m, c.clone()),
            _ => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let player_pos = match scene.players.get(&entity_id) {
            Some(p) => p.pos.clone(),
            None => return Ok(()),
        };

        let mut found_key: Option<(i32, i32)> = None;
        for entry in scene.ground_items.iter() {
            let (gx, gy) = *entry.key();
            if (gx - player_pos.x).abs() <= 1 && (gy - player_pos.y).abs() <= 1 {
                found_key = Some((gx, gy));
                break;
            }
        }

        let key = match found_key {
            Some(k) => k,
            None => {
                let err = GameError::new(GameErrorCode::ItemNotFound, "No hay items en el suelo cercanos");
                self.send_to_client(sink, err.to_console_packet()).await?;
                return Ok(());
            }
        };

        if let Some((_, ground_item)) = scene.ground_items.remove(&key) {
            let inv = self.world.cache_get_inventory(&char_id);

            let target_slot = inv.iter()
                .find(|r| r.item_id == ground_item.item_id)
                .map(|r| r.slot)
                .or_else(|| self.world.cache_find_empty_slot(&char_id));

            let slot = match target_slot {
                Some(s) => s,
                None => {
                    scene.ground_items.insert(key, ground_item);
                    self.send_to_client(sink, GameError::inventory_full().to_console_packet()).await?;
                    return Ok(());
                }
            };

            let existing = inv.iter().find(|r| r.slot == slot);
            let new_amount = existing.map(|ex| ex.amount + ground_item.amount).unwrap_or(ground_item.amount);

            self.world.cache_update_slot(&char_id, slot, ground_item.item_id, new_amount, false);

            let item_data = crate::replication::get_item_data(&self.world.gd(), ground_item.item_id);
            let row = crate::persistence::InventoryRow {
                slot, item_id: ground_item.item_id, amount: new_amount, equipped: false,
            };
            let inv_pkt = crate::replication::build_inv_item_packet(&row, &item_data);
            self.send_to_client(sink, inv_pkt).await?;

            let del_pkt = crate::replication::build_delete_ground_item(key.0, key.1);
            scene.broadcast_in_range(0, &player_pos, del_pkt);

            let msg = format!("Recogiste {} x{}", item_data.name, ground_item.amount);
            let pkt = build_console_message(&msg);
            self.send_to_client(sink, pkt).await?;

            self.advance_quest_collect(entity_id, ground_item.item_id, ground_item.amount as u32);
        }

        Ok(())
    }

    pub(super) async fn handle_use_item(
        &mut self,
        slot: u8,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id, char_id) = match (self.entity_id, self.map_id, &self.character_id) {
            (Some(e), Some(m), Some(c)) => (e, m, c.clone()),
            _ => return Ok(()),
        };

        {
            let now = self.world.uptime_ms();
            let scene = self.world.get_or_create_scene(map_id);
            if let Some(p) = scene.players.get(&entity_id) {
                if !p.action_cooldowns.can_use_item(now) {
                    return Ok(());
                }
            }
            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                p.action_cooldowns.trigger_use_item(now);
            }
        }

        let inv = self.world.cache_get_inventory(&char_id);
        let item = match inv.iter().find(|r| r.slot == slot as i32) {
            Some(i) => i.clone(),
            None => {
                let err = GameError::new(GameErrorCode::InvalidSlot, "Slot vacío");
                self.send_to_client(sink, err.to_console_packet()).await?;
                return Ok(());
            }
        };

        let obj = match self.world.gd().get_object(item.item_id) {
            Some(o) => o.clone(),
            None => {
                self.send_to_client(sink, build_console_message("Item desconocido")).await?;
                return Ok(());
            }
        };

        let item_info = crate::replication::get_item_data(&self.world.gd(), item.item_id);
        let scene = self.world.get_or_create_scene(map_id);

        if obj.newbie != 0 {
            let player_level = scene.players.get(&entity_id).map(|p| p.level).unwrap_or(0);
            if !crate::gameplay::combat_formulas::is_newbie_character(player_level) {
                self.send_to_client(sink, build_console_message("Solo los personajes newbie pueden usar este item.")).await?;
                return Ok(());
            }
        }

        if let Some(p) = scene.players.get(&entity_id)
            && p.dead && obj.obj_type != 31
        {
            self.send_to_client(sink, build_console_message("Los muertos no pueden usar items.")).await?;
            return Ok(());
        }

        if crate::gateway::fishing::is_fishing_rod(item.item_id)
            && self.handle_fishing_rod_use(entity_id, slot, item.item_id, sink).await? {
                return Ok(());
            }

        if crate::gateway::harvesting::is_harvesting_tool(item.item_id, &self.world.gd())
            && self.handle_harvesting_tool_use(entity_id, slot, item.item_id, sink).await? {
                return Ok(());
            }

        const OBJ_TYPE_BOAT: i32 = 14;
        if obj.obj_type == OBJ_TYPE_BOAT {
            let is_nav = scene.players.get(&entity_id).map(|p| p.navegando).unwrap_or(false);
            let new_nav = !is_nav;

            if new_nav {
                let px = scene.players.get(&entity_id).map(|p| p.pos.x).unwrap_or(0);
                let py = scene.players.get(&entity_id).map(|p| p.pos.y).unwrap_or(0);
                if !self.world.gd().is_water_tile(map_id, px, py)
                    && !self.world.gd().is_adjacent_to_water(map_id, px, py) {
                    self.send_to_client(sink, build_console_message("Necesitas estar cerca del agua para navegar.")).await?;
                    return Ok(());
                }
            }

            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                p.navegando = new_nav;
            }
            self.send_to_client(sink, build_navegando(new_nav)).await?;
            let msg = if new_nav { "Comienzas a navegar." } else { "Dejas de navegar." };
            self.send_to_client(sink, build_console_message(msg)).await?;
            return Ok(());
        }

        #[allow(unused_assignments)]
        let mut consumed = false;

        let name_lower = obj.name.to_lowercase();
        let is_crafting_tool = name_lower.contains("serrucho")
            || name_lower.contains("costurero")
            || (name_lower.contains("martillo") && name_lower.contains("herrero"));

        if is_crafting_tool {
            let profession = if name_lower.contains("serrucho") {
                "carpentry"
            } else if name_lower.contains("costurero") {
                "tailoring"
            } else {
                "blacksmith"
            };

            let title = match profession {
                "carpentry" => "Carpintería",
                "tailoring" => "Sastrería",
                _ => "Herrería",
            };

            let player_level = scene.players.get(&entity_id)
                .map(|p| p.level)
                .unwrap_or(1);
            let skill = (player_level * 3).min(100);

            let recipes: Vec<serde_json::Value> = self.world.gd().crafting_recipes.iter()
                .filter(|r| r.profession == profession && r.skill <= skill)
                .map(|r| {
                    let item_data = crate::replication::get_item_data(&self.world.gd(), r.item_id);
                    let materials: Vec<serde_json::Value> = r.materials.iter().map(|m| {
                        let mat_data = crate::replication::get_item_data(&self.world.gd(), m.item_id);
                        serde_json::json!({
                            "itemId": m.item_id,
                            "name": mat_data.name,
                            "amount": m.amount,
                            "owned": 0
                        })
                    }).collect();
                    serde_json::json!({
                        "itemId": r.item_id,
                        "name": item_data.name,
                        "grhIndex": item_data.grh_index,
                        "amount": 1,
                        "materials": materials
                    })
                })
                .collect();

            let payload = serde_json::json!({
                "profession": profession,
                "title": title,
                "recipes": recipes
            });

            let pkt = crate::gateway::packets::build_open_crafting(&payload.to_string());
            self.send_to_client(sink, pkt).await?;
            return Ok(());
        }

        match obj.obj_type {
            11 => {
                // Pociones
                match obj.tipo_pocion {
                    3 => {
                        let result = {
                            if let Some(mut player) = scene.players.get_mut(&entity_id) {
                                if player.hp < player.max_hp {
                                    let base = {
                                        let mut rng = rand::rng();
                                        rand::Rng::random_range(&mut rng, obj.min_modificador.max(1)..=obj.max_modificador.max(1))
                                    };
                                    let pct_bonus = if obj.porcentaje > 0 { player.max_hp * obj.porcentaje / 100 } else { 0 };
                                    let heal = (base + pct_bonus).max(1);
                                    player.hp = (player.hp + heal).min(player.max_hp);
                                    let vals = (player.hp, player.max_hp, player.mana, player.max_mana);
                                    Some((vals, heal))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        };
                        if let Some(((hp, max_hp, mana, max_mana), heal)) = result {
                            self.send_to_client(sink, build_self_vitals(hp, max_hp, mana, max_mana)).await?;
                            let entity_vitals = crate::replication::build_entity_vitals_delta(entity_id, hp, max_hp, mana, max_mana);
                            let potion_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                            if let Some(ref pos) = potion_pos {
                                scene.broadcast_in_range(entity_id, pos, entity_vitals);
                            }
                            let m = format!("Usas {} (+{} HP)", obj.name, heal);
                            self.send_to_client(sink, build_console_message(&m)).await?;
                        }
                        consumed = true;
                    }
                    4 => {
                        let result = {
                            if let Some(mut player) = scene.players.get_mut(&entity_id) {
                                if player.mana < player.max_mana {
                                    let base = (player.max_mana as f32 * 0.04 + player.level as f32 / 2.0 + 40.0 / player.level.max(1) as f32) as i32;
                                    let pct_bonus = if obj.porcentaje > 0 { player.max_mana * obj.porcentaje / 100 } else { 0 };
                                    let restore = (base + pct_bonus).max(1);
                                    player.mana = (player.mana + restore).min(player.max_mana);
                                    let vals = (player.hp, player.max_hp, player.mana, player.max_mana);
                                    Some((vals, restore))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        };
                        if let Some(((hp, max_hp, mana, max_mana), restore)) = result {
                            self.send_to_client(sink, build_self_vitals(hp, max_hp, mana, max_mana)).await?;
                            let entity_vitals = crate::replication::build_entity_vitals_delta(entity_id, hp, max_hp, mana, max_mana);
                            let potion_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                            if let Some(ref pos) = potion_pos {
                                scene.broadcast_in_range(entity_id, pos, entity_vitals);
                            }
                            let m = format!("Usas {} (+{} Mana)", obj.name, restore);
                            self.send_to_client(sink, build_console_message(&m)).await?;
                        }
                        consumed = true;
                    }
                    1 => {
                        let boost = {
                            let mut rng = rand::rng();
                            rand::Rng::random_range(&mut rng, obj.min_modificador.max(1)..=obj.max_modificador.max(1))
                        };
                        if let Some(mut p) = scene.players.get_mut(&entity_id) {
                            p.buffs.apply(crate::gameplay::buffs::BuffType::Agility, boost, 60 * 60);
                        }
                        let msg = format!("Usas {} (+{} agilidad, 60s)", obj.name, boost);
                        self.send_to_client(sink, build_console_message(&msg)).await?;

                        let mut w = PacketWriter::with_packet_id(client_packet_id::UPDATE_AGILIDAD);
                        w.write_short(boost as u16);
                        self.send_to_client(sink, w.into_bytes()).await?;
                        consumed = true;
                    }
                    2 => {
                        let boost = {
                            let mut rng = rand::rng();
                            rand::Rng::random_range(&mut rng, obj.min_modificador.max(1)..=obj.max_modificador.max(1))
                        };
                        if let Some(mut p) = scene.players.get_mut(&entity_id) {
                            p.buffs.apply(crate::gameplay::buffs::BuffType::Strength, boost, 60 * 60);
                        }
                        let msg = format!("Usas {} (+{} fuerza, 60s)", obj.name, boost);
                        self.send_to_client(sink, build_console_message(&msg)).await?;

                        let mut w = PacketWriter::with_packet_id(client_packet_id::UPDATE_FUERZA);
                        w.write_short(boost as u16);
                        self.send_to_client(sink, w.into_bytes()).await?;
                        consumed = true;
                    }
                    _ => {
                        self.send_to_client(sink, build_console_message("Esta poción aún no tiene efecto implementado.")).await?;
                        return Ok(());
                    }
                }
            }
            1 | 13 => {
                let msg = format!("Consumes {}", obj.name);
                self.send_to_client(sink, build_console_message(&msg)).await?;
                consumed = true;
            }
            24 => {
                if obj.spell_index <= 0 {
                    self.send_to_client(sink, build_console_message("Pergamino sin hechizo.")).await?;
                    return Ok(());
                }
                let no_mana_classes = [3, 5, 9, 10, 11];
                let id_clase = scene.players.get(&entity_id).map(|p| p.id_clase).unwrap_or(1);
                if no_mana_classes.contains(&id_clase) {
                    self.send_to_client(sink, build_console_message("Tu clase no puede aprender hechizos.")).await?;
                    return Ok(());
                }

                let gd = self.world.gd();
                let spell_data = gd.spells.get(&obj.spell_index);
                let spell_name = spell_data.map(|s| s.name.clone()).unwrap_or_else(|| "Desconocido".to_string());
                let mana_cost = spell_data.map(|s| s.mana_required).unwrap_or(0) as u16;

                let already_known = crate::replication::DEFAULT_SPELLS
                    .iter()
                    .any(|(_, id)| *id == obj.spell_index as u16);
                if already_known {
                    self.send_to_client(sink, build_console_message("Ya conoces este hechizo.")).await?;
                    return Ok(());
                }

                let next_slot = {
                    let max_existing = crate::replication::DEFAULT_SPELLS.iter()
                        .map(|(s, _)| *s)
                        .max()
                        .unwrap_or(0);
                    max_existing + 1
                };

                if next_slot > 35 {
                    self.send_to_client(sink, build_console_message("No tienes espacio para más hechizos.")).await?;
                    return Ok(());
                }

                let learn_pkt = crate::replication::build_learn_spell(next_slot, obj.spell_index as u16, &spell_name, mana_cost);
                self.send_to_client(sink, learn_pkt).await?;
                let msg = format!("Has aprendido: {}", spell_name);
                self.send_to_client(sink, build_console_message(&msg)).await?;
                consumed = true;
            }
            19 => {
                // Travel ticket (teleport item)
                let pvp_blocked = scene.players.get(&entity_id).map(|p| {
                    if p.pvp_block_until_ms > 0 {
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        now_ms < p.pvp_block_until_ms
                    } else { false }
                }).unwrap_or(false);

                if pvp_blocked {
                    self.send_to_client(sink, build_console_message("No puedes teleportarte durante 5 segundos después de combate PvP.")).await?;
                    return Ok(());
                }

                let dest = match &obj.travel_ticket_destination {
                    Some(d) if d.map > 0 && d.x > 0 && d.y > 0 => d.clone(),
                    _ => {
                        self.send_to_client(sink, build_console_message("Este item todavía no tiene un destino configurado.")).await?;
                        return Ok(());
                    }
                };

                self.send_to_client(sink, build_console_message(&format!("Usas {} — teleportando...", obj.name))).await?;

                // Consume item before teleport
                let new_amount = item.amount - 1;
                if new_amount <= 0 {
                    self.world.db.delete_inventory_slot(&char_id, slot as i32).await.ok();
                    self.world.cache_delete_slot(&char_id, slot as i32);
                    let row = crate::persistence::InventoryRow { slot: slot as i32, item_id: 0, amount: 0, equipped: false };
                    let empty_data = crate::replication::get_item_data(&self.world.gd(), 0);
                    let empty_pkt = crate::replication::build_inv_item_packet(&row, &empty_data);
                    self.send_to_client(sink, empty_pkt).await?;
                } else {
                    self.world.db.update_inventory_slot(&char_id, slot as i32, item.item_id, new_amount, item.equipped).await.ok();
                    self.world.cache_update_slot(&char_id, slot as i32, item.item_id, new_amount, item.equipped);
                    let row = crate::persistence::InventoryRow { slot: slot as i32, item_id: item.item_id, amount: new_amount, equipped: item.equipped };
                    let updated_pkt = crate::replication::build_inv_item_packet(&row, &item_info);
                    self.send_to_client(sink, updated_pkt).await?;
                }

                let old_map = map_id;
                drop(scene);

                self.do_teleport(entity_id, old_map, dest.map, dest.x, dest.y, sink).await?;
                return Ok(());
            }
            _ => {
                self.send_to_client(sink, build_console_message("No se puede usar este item.")).await?;
                return Ok(());
            }
        }

        if consumed {
            if matches!(obj.obj_type, 11 | 1 | 13) {
                let sound = crate::replication::build_play_sound(46);
                let sound_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                if let Some(ref pos) = sound_pos {
                    scene.broadcast_in_range(0, pos, sound);
                } else {
                    scene.broadcast(0, sound);
                }
            }

            let new_amount = item.amount - 1;
            if new_amount <= 0 {
                self.world.cache_delete_slot(&char_id, slot as i32);
                let mut w = PacketWriter::with_packet_id(client_packet_id::QUITAR_USER_INV_ITEM);
                w.write_byte(slot);
                self.send_to_client(sink, w.into_bytes()).await?;
            } else {
                self.world.cache_update_slot(&char_id, slot as i32, item.item_id, new_amount, item.equipped);
                let row = crate::persistence::InventoryRow { slot: slot as i32, item_id: item.item_id, amount: new_amount, equipped: item.equipped };
                let pkt = crate::replication::build_inv_item_packet(&row, &item_info);
                self.send_to_client(sink, pkt).await?;
            }
        }

        Ok(())
    }

    pub(super) async fn handle_equip_item(
        &mut self,
        slot: u8,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id, char_id) = match (self.entity_id, self.map_id, &self.character_id) {
            (Some(e), Some(m), Some(c)) => (e, m, c.clone()),
            _ => return Ok(()),
        };

        {
            let now = self.world.uptime_ms();
            let scene = self.world.get_or_create_scene(map_id);
            if let Some(p) = scene.players.get(&entity_id) {
                if !p.action_cooldowns.can_equip_toggle(now) {
                    return Ok(());
                }
            }
            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                p.action_cooldowns.trigger_equip_toggle(now);
            }
        }

        let inv = self.world.cache_get_inventory(&char_id);
        let item = match inv.iter().find(|r| r.slot == slot as i32) {
            Some(i) => i.clone(),
            None => {
                let err = GameError::new(GameErrorCode::InvalidSlot, "Slot vacío");
                self.send_to_client(sink, err.to_console_packet()).await?;
                return Ok(());
            }
        };

        let new_equipped = !item.equipped;

        if new_equipped {
            let gd = self.world.gd();
            if let Some(obj) = gd.get_object(item.item_id) {
                let player_raza = {
                    let scene = self.world.get_or_create_scene(map_id);
                    scene.players.get(&entity_id).map(|p| p.id_raza).unwrap_or(1)
                };
                let is_dwarf_race = player_raza == 4 || player_raza == 5;
                if obj.raza_enana == 1 && !is_dwarf_race {
                    self.send_to_client(sink, build_console_message("Solo razas enanas pueden equipar este objeto.")).await?;
                    return Ok(());
                }
                if obj.raza_enana == 0 && is_dwarf_race && obj.obj_type == 3 {
                    self.send_to_client(sink, build_console_message("Tu raza no puede equipar esta armadura.")).await?;
                    return Ok(());
                }
                if !obj.clases_no_permitidas.is_empty() {
                    let scene = self.world.get_or_create_scene(map_id);
                    let player_class = scene.players.get(&entity_id).map(|p| p.id_clase).unwrap_or(1);
                    if obj.clases_no_permitidas.contains(&player_class) {
                        self.send_to_client(sink, build_console_message("Tu clase no puede equipar este objeto.")).await?;
                        return Ok(());
                    }
                }
            }
        }

        self.world.cache_update_slot(&char_id, slot as i32, item.item_id, item.amount, new_equipped);

        let item_data = crate::replication::get_item_data(&self.world.gd(), item.item_id);
        let row = crate::persistence::InventoryRow { slot: slot as i32, item_id: item.item_id, amount: item.amount, equipped: new_equipped };
        let pkt = crate::replication::build_inv_item_packet(&row, &item_data);
        self.send_to_client(sink, pkt).await?;

        let gd = self.world.gd();
        let obj = gd.get_object(item.item_id);
        if let Some(obj) = obj {
            let anim = obj.anim as i32;
            let visual_id = if new_equipped { anim } else { 0 };
            let change_pkt = match obj.obj_type {
                2 => {
                    let scene = self.world.get_or_create_scene(map_id);
                    if let Some(mut p) = scene.players.get_mut(&entity_id) {
                        p.id_weapon = if new_equipped { item.item_id } else { 0 };
                    }
                    Some(build_change_equipment(client_packet_id::CHANGE_WEAPON, entity_id, visual_id))
                }
                3 => {
                    let scene = self.world.get_or_create_scene(map_id);
                    if let Some(mut p) = scene.players.get_mut(&entity_id) {
                        p.id_body = if new_equipped { anim } else { 0 };
                    }
                    Some(build_change_equipment(client_packet_id::CHANGE_BODY, entity_id, visual_id))
                }
                4 => {
                    let scene = self.world.get_or_create_scene(map_id);
                    if let Some(mut p) = scene.players.get_mut(&entity_id) {
                        p.id_helmet = if new_equipped { anim } else { 0 };
                    }
                    Some(build_change_equipment(client_packet_id::CHANGE_HELMET, entity_id, visual_id))
                }
                8 => {
                    let scene = self.world.get_or_create_scene(map_id);
                    if let Some(mut p) = scene.players.get_mut(&entity_id) {
                        p.id_shield = if new_equipped { anim } else { 0 };
                    }
                    Some(build_change_equipment(client_packet_id::CHANGE_SHIELD, entity_id, visual_id))
                }
                _ => None,
            };

            if let Some(change_pkt) = change_pkt {
                self.send_to_client(sink, change_pkt.clone()).await?;
                let scene = self.world.get_or_create_scene(map_id);
                let equip_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
                if let Some(ref pos) = equip_pos {
                    scene.broadcast_in_range(entity_id, pos, change_pkt);
                } else {
                    scene.broadcast(entity_id, change_pkt);
                }
            }
        }

        let msg = if new_equipped { format!("Equipas {}", item_data.name) } else { format!("Desequipas {}", item_data.name) };
        self.send_to_client(sink, build_console_message(&msg)).await?;

        Ok(())
    }

    pub(super) async fn handle_drop_item(
        &mut self,
        slot: u8,
        qty: u16,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id, char_id) = match (self.entity_id, self.map_id, &self.character_id) {
            (Some(e), Some(m), Some(c)) => (e, m, c.clone()),
            _ => return Ok(()),
        };

        {
            let now = self.world.uptime_ms();
            let scene = self.world.get_or_create_scene(map_id);
            if let Some(p) = scene.players.get(&entity_id) {
                if !p.action_cooldowns.can_drop_item(now) {
                    return Ok(());
                }
            }
            if let Some(mut p) = scene.players.get_mut(&entity_id) {
                p.action_cooldowns.trigger_drop_item(now);
            }
        }

        let inv = self.world.cache_get_inventory(&char_id);
        let item = match inv.iter().find(|r| r.slot == slot as i32) {
            Some(i) => i.clone(),
            None => {
                let err = GameError::new(GameErrorCode::InvalidSlot, "Slot vacío");
                self.send_to_client(sink, err.to_console_packet()).await?;
                return Ok(());
            }
        };

        let item_data = crate::replication::get_item_data(&self.world.gd(), item.item_id);
        let drop_qty = (qty as i32).min(item.amount);
        let remaining = item.amount - drop_qty;

        if remaining <= 0 {
            self.world.cache_delete_slot(&char_id, slot as i32);
            let mut w = PacketWriter::with_packet_id(client_packet_id::QUITAR_USER_INV_ITEM);
            w.write_byte(slot);
            self.send_to_client(sink, w.into_bytes()).await?;
        } else {
            self.world.cache_update_slot(&char_id, slot as i32, item.item_id, remaining, item.equipped);
            let row = crate::persistence::InventoryRow { slot: slot as i32, item_id: item.item_id, amount: remaining, equipped: item.equipped };
            let pkt = crate::replication::build_inv_item_packet(&row, &item_data);
            self.send_to_client(sink, pkt).await?;
        }

        let scene = self.world.get_or_create_scene(map_id);
        let player_pos = scene.players.get(&entity_id).map(|p| p.pos.clone());
        if let Some(ref pos) = player_pos {
            if let Some((drop_x, drop_y)) = find_nearest_drop_position(map_id, pos.x, pos.y, &self.world, &scene) {
                let ground_item = crate::world::GroundItem {
                    x: drop_x,
                    y: drop_y,
                    item_id: item.item_id,
                    amount: drop_qty,
                    grh_index: item_data.grh_index,
                    dropped_at_ms: self.world.uptime_ms(),
                };
                scene.ground_items.insert((drop_x, drop_y), ground_item);

                let render_pkt = crate::replication::build_render_item(
                    drop_x, drop_y, item.item_id, drop_qty, item_data.grh_index,
                );
                scene.broadcast_in_range(0, pos, render_pkt);
            } else {
                self.send_to_client(sink, build_console_message("No hay espacio para tirar el item.")).await?;
            }
        }

        let msg = format!("Tiras {} x{}", item_data.name, drop_qty);
        self.send_to_client(sink, build_console_message(&msg)).await?;

        Ok(())
    }

    pub(super) async fn handle_reorder_inventory(
        &mut self,
        source: u8,
        target: u8,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let char_id = match &self.character_id {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        if source == target {
            return Ok(());
        }

        let inv = self.world.cache_get_inventory(&char_id);
        let src_item = inv.iter().find(|r| r.slot == source as i32).cloned();
        let tgt_item = inv.iter().find(|r| r.slot == target as i32).cloned();

        match (src_item, tgt_item) {
            (Some(src), Some(tgt)) => {
                self.world.cache_update_slot(&char_id, source as i32, tgt.item_id, tgt.amount, tgt.equipped);
                self.world.cache_update_slot(&char_id, target as i32, src.item_id, src.amount, src.equipped);
            }
            (Some(src), None) => {
                self.world.cache_delete_slot(&char_id, source as i32);
                self.world.cache_update_slot(&char_id, target as i32, src.item_id, src.amount, src.equipped);
            }
            _ => return Ok(()),
        }

        self.send_full_inventory(sink).await?;

        Ok(())
    }

    pub(super) async fn handle_reorder_spell(
        &mut self,
        _source: u8,
        _target: u8,
        _sink: &mut WsSink,
    ) -> HandlerResult {
        Ok(())
    }

    pub(super) async fn handle_toggle_hidden(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(e) => e,
            None => return Ok(()),
        };
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let Some(player) = scene.players.get(&entity_id) else { return Ok(()); };

        if player.dead {
            self.send_to_client(sink, build_console_message("Los muertos no pueden ocultarse.")).await?;
            return Ok(());
        }

        if player.hidden_skill {
            self.send_to_client(sink, build_console_message("Ya estás oculto.")).await?;
            return Ok(());
        }

        let current_tick = self.metrics.current_tick.load(std::sync::atomic::Ordering::Relaxed);
        if player.hidden_skill_cooldown_tick > current_tick {
            self.send_to_client(sink, build_console_message("Todavía no puedes volver a ocultarte.")).await?;
            return Ok(());
        }

        let level = player.level as f64;
        let skill = (level * 3.0).min(100.0);
        drop(player);

        let chance = hidden_skill_chance(skill);
        let cooldown_ticks: u64 = 9;
        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.hidden_skill_cooldown_tick = current_tick + cooldown_ticks;
        }

        let roll = rand::random_range(1..=100) as f64;
        if roll > chance {
            self.send_to_client(sink, build_console_message("Fallaste al intentar ocultarte.")).await?;
            return Ok(());
        }

        let duration_ticks = hidden_skill_duration_ticks(skill);
        if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.hidden_skill = true;
            p.hidden_skill_expire_tick = current_tick + duration_ticks;
        }

        let delete_pkt = crate::replication::build_delete_character_packet(entity_id);
        if let Some(player) = scene.players.get(&entity_id) {
            scene.broadcast_in_range(entity_id, &player.pos, delete_pkt);
        }

        self.send_to_client(sink, build_console_message("Te ocultas entre las sombras.")).await?;
        Ok(())
    }

    pub(super) async fn handle_toggle_clan_safe(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(e) => e,
            None => return Ok(()),
        };
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let scene = self.world.get_or_create_scene(map_id);
        let (new_state, pk) = if let Some(mut p) = scene.players.get_mut(&entity_id) {
            p.seguro_clan_activado = !p.seguro_clan_activado;
            let zona = self.world.gd().maps_meta.get(&map_id).map(|m| m.pk).unwrap_or(0) as u8;
            (p.seguro_clan_activado, zona)
        } else {
            return Ok(());
        };

        let personal_safe = scene.players.get(&entity_id).map(|p| p.seguro_activado).unwrap_or(false);
        self.send_to_client(sink, build_self_flags(pk, personal_safe, new_state)).await?;

        let msg = if new_state { "Seguro de clan activado" } else { "Seguro de clan desactivado" };
        self.send_to_client(sink, build_console_message(msg)).await?;

        Ok(())
    }
}

fn hidden_skill_chance(skill: f64) -> f64 {
    let raw = (((0.000002 * skill - 0.0002) * skill + 0.0064) * skill + 0.1124) * 100.0;
    raw.clamp(1.0, 99.0)
}

fn hidden_skill_duration_ticks(skill: f64) -> u64 {
    let missing = 100.0 - skill;
    let mut counter: f64 = 0.0;
    let mut remaining = missing;
    while remaining > 0.0 {
        let step = remaining.min(10.0);
        counter += step * (1.0 + (100.0 - remaining) / 100.0);
        remaining -= step;
    }
    let duration_ms = (counter * 40.0).max(1000.0);
    (duration_ms / 16.67) as u64
}

pub fn stop_hidden_skill(entity_id: u32, scene: &crate::world::Scene, current_tick: u64, rehide_delay_ticks: u64) {
    if let Some(mut p) = scene.players.get_mut(&entity_id) {
        if !p.hidden_skill { return; }
        p.hidden_skill = false;
        p.hidden_skill_expire_tick = 0;
        if rehide_delay_ticks > 0 {
            p.hidden_skill_cooldown_tick = p.hidden_skill_cooldown_tick.max(current_tick + rehide_delay_ticks);
        }
    }
    if let Some(player) = scene.players.get(&entity_id) {
        let char_pkt = crate::replication::build_character_packet(&player);
        let pos = player.pos.clone();
        drop(player);
        scene.broadcast_in_range(entity_id, &pos, char_pkt);
    }
    if let Some(tx) = scene.personal_tx.get(&entity_id) {
        let _ = tx.send(super::packets::build_console_message("Has vuelto a ser visible."));
    }
}

pub fn can_keep_hidden_while_acting(player: &crate::world::PlayerState) -> bool {
    player.hidden_skill && player.id_clase == 8
}

fn can_drop_at(map_id: i32, x: i32, y: i32, world: &crate::world::GameWorld, scene: &crate::world::Scene) -> bool {
    if world.gd().is_blocked_tile(map_id, x, y) {
        return false;
    }
    if world.gd().get_tile_exit(map_id, x, y).is_some() {
        return false;
    }
    if scene.ground_items.contains_key(&(x, y)) {
        return false;
    }
    true
}

pub fn find_nearest_drop_position(
    map_id: i32,
    origin_x: i32,
    origin_y: i32,
    world: &crate::world::GameWorld,
    scene: &crate::world::Scene,
) -> Option<(i32, i32)> {
    let (map_w, map_h) = world.gd().get_map_bounds(map_id);
    for radius in 0..=10i32 {
        let min_x = (origin_x - radius).max(1);
        let max_x = (origin_x + radius).min(map_w);
        let min_y = (origin_y - radius).max(1);
        let max_y = (origin_y + radius).min(map_h);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if radius > 0 && x > min_x && x < max_x && y > min_y && y < max_y {
                    continue;
                }
                if can_drop_at(map_id, x, y, world, scene) {
                    return Some((x, y));
                }
            }
        }
    }
    None
}
