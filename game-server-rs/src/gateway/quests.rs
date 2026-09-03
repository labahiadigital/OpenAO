use super::packets::*;
use super::GameSession;
use crate::error::HandlerResult;
use crate::gateway::WsSink;

impl GameSession {
    pub(crate) async fn handle_quest_list(
        &mut self,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let mid = self.map_id.unwrap_or(0);
        let scene = match self.world.scenes.get(&mid) {
            Some(s) => s,
            None => return Ok(()),
        };
        let player = match scene.players.get(&entity_id) {
            Some(p) => p,
            None => return Ok(()),
        };

        if player.quest_log.active.is_empty() {
            self.send_to_client(sink, build_console_message("No tienes misiones activas. Habla con NPCs para obtener misiones.")).await?;
            return Ok(());
        }

        let gd = self.world.gd();
        self.send_to_client(sink, build_console_message("--- Misiones Activas ---")).await?;
        for aq in &player.quest_log.active {
            let name = gd.quests.get(aq.quest_id)
                .map(|d| d.name.as_str())
                .unwrap_or("???");
            let progress: Vec<String> = aq.objectives.iter().map(|o| {
                format!("{}/{}", o.current, o.required)
            }).collect();
            let status = if aq.is_complete() { " [COMPLETAR]" } else { "" };
            self.send_to_client(sink, build_console_message(
                &format!("  #{}: {} - Progreso: {}{}", aq.quest_id, name, progress.join(", "), status),
            )).await?;
        }

        let completed_count = player.quest_log.completed.len();
        if completed_count > 0 {
            self.send_to_client(sink, build_console_message(
                &format!("Misiones completadas: {}", completed_count),
            )).await?;
        }

        Ok(())
    }

    pub(crate) async fn handle_quest_accept(
        &mut self,
        quest_id: u32,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let gd = self.world.gd();
        let def = match gd.quests.get(quest_id) {
            Some(d) => d,
            None => {
                self.send_to_client(sink, build_console_message("Mision no encontrada.")).await?;
                return Ok(());
            }
        };

        let mid = self.map_id.unwrap_or(0);
        let scene = match self.world.scenes.get(&mid) {
            Some(s) => s,
            None => return Ok(()),
        };
        let mut player = match scene.players.get_mut(&entity_id) {
            Some(p) => p,
            None => return Ok(()),
        };

        match player.quest_log.can_accept(def, player.level as u32) {
            Ok(()) => {
                player.quest_log.accept(def);
                self.send_to_client(sink, build_console_message(
                    &format!("Mision aceptada: {} - {}", def.name, def.description),
                )).await?;
            }
            Err(reason) => {
                self.send_to_client(sink, build_console_message(reason)).await?;
            }
        }
        Ok(())
    }

    pub(crate) async fn handle_quest_abandon(
        &mut self,
        quest_id: u32,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let mid = self.map_id.unwrap_or(0);
        let scene = match self.world.scenes.get(&mid) {
            Some(s) => s,
            None => return Ok(()),
        };
        let mut player = match scene.players.get_mut(&entity_id) {
            Some(p) => p,
            None => return Ok(()),
        };

        if player.quest_log.abandon(quest_id) {
            self.send_to_client(sink, build_console_message("Mision abandonada.")).await?;
        } else {
            self.send_to_client(sink, build_console_message("No tienes esa mision activa.")).await?;
        }
        Ok(())
    }

    pub(crate) async fn handle_quest_complete(
        &mut self,
        quest_id: u32,
        entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let gd = self.world.gd();
        let def = match gd.quests.get(quest_id) {
            Some(d) => d.clone(),
            None => {
                self.send_to_client(sink, build_console_message("Mision no encontrada.")).await?;
                return Ok(());
            }
        };

        let mid = self.map_id.unwrap_or(0);
        let scene = match self.world.scenes.get(&mid) {
            Some(s) => s,
            None => return Ok(()),
        };
        let mut player = match scene.players.get_mut(&entity_id) {
            Some(p) => p,
            None => return Ok(()),
        };

        let is_complete = player.quest_log.get_active(quest_id)
            .map(|a| a.is_complete())
            .unwrap_or(false);

        if !is_complete {
            self.send_to_client(sink, build_console_message("La mision no esta completa aun.")).await?;
            return Ok(());
        }

        if player.quest_log.complete(quest_id) {
            player.gold = crate::gameplay::balance::clamp_gold((player.gold + def.reward.gold) as i64) as i32;
            player.exp += def.reward.exp as i32;

            let gold = player.gold;
            let exp = player.exp;
            let exp_next = player.exp_next_level;
            let reward_items = def.reward.items.clone();
            drop(player);

            self.send_to_client(sink, build_console_message(
                &format!("Mision completada: {}! Recompensa: {} oro, {} exp",
                    def.name, def.reward.gold, def.reward.exp),
            )).await?;

            if !reward_items.is_empty() {
                for &(item_id, amount) in &reward_items {
                    if let Some(ref char_id) = self.character_id {
                        let _ = self.world.cache_add_item(char_id, item_id, amount.into());
                    }
                }
                self.send_to_client(sink, build_console_message("Items de recompensa agregados al inventario.")).await?;
            }

            self.send_to_client(sink, build_act_gold(gold)).await?;
            self.send_to_client(sink, build_act_exp(exp, exp_next)).await?;
        }
        Ok(())
    }

    /// Called when an NPC is killed, to advance kill objectives in all active quests.
    pub fn advance_quest_kills(&self, entity_id: u32, npc_type: i32) {
        let mid = self.map_id.unwrap_or(0);
        let scene = match self.world.scenes.get(&mid) {
            Some(s) => s,
            None => return,
        };
        let mut player = match scene.players.get_mut(&entity_id) {
            Some(p) => p,
            None => return,
        };

        let gd = self.world.gd();
        let quest_ids: Vec<u32> = player.quest_log.active.iter().map(|a| a.quest_id).collect();
        for qid in quest_ids {
            if let Some(def) = gd.quests.get(qid)
                && let Some(aq) = player.quest_log.get_active_mut(qid) {
                    aq.advance_kill(npc_type, def);
                }
        }
    }

    /// Called when a player picks up / collects an item, to advance collect objectives.
    pub fn advance_quest_collect(&self, entity_id: u32, item_id: i32, amount: u32) {
        let mid = self.map_id.unwrap_or(0);
        let scene = match self.world.scenes.get(&mid) {
            Some(s) => s,
            None => return,
        };
        let mut player = match scene.players.get_mut(&entity_id) {
            Some(p) => p,
            None => return,
        };

        let gd = self.world.gd();
        let quest_ids: Vec<u32> = player.quest_log.active.iter().map(|a| a.quest_id).collect();
        for qid in quest_ids {
            if let Some(def) = gd.quests.get(qid)
                && let Some(aq) = player.quest_log.get_active_mut(qid) {
                    aq.advance_collect(item_id, amount, def);
                }
        }
    }

    /// Called when a player enters a new map (teleport/connect).
    pub fn advance_quest_visit_map(&self, entity_id: u32, map_id: i32) {
        let mid = map_id;
        let scene = match self.world.scenes.get(&mid) {
            Some(s) => s,
            None => return,
        };
        let mut player = match scene.players.get_mut(&entity_id) {
            Some(p) => p,
            None => return,
        };

        let gd = self.world.gd();
        let quest_ids: Vec<u32> = player.quest_log.active.iter().map(|a| a.quest_id).collect();
        for qid in quest_ids {
            if let Some(def) = gd.quests.get(qid)
                && let Some(aq) = player.quest_log.get_active_mut(qid) {
                    aq.advance_visit_map(map_id, def);
                }
        }
    }

    /// Called when a player levels up, to advance reach_level objectives.
    pub fn advance_quest_level(entity_id: u32, level: u32, world: &crate::world::GameWorld, map_id: i32) {
        let scene = match world.scenes.get(&map_id) {
            Some(s) => s,
            None => return,
        };
        let mut player = match scene.players.get_mut(&entity_id) {
            Some(p) => p,
            None => return,
        };

        let gd = world.gd();
        let quest_ids: Vec<u32> = player.quest_log.active.iter().map(|a| a.quest_id).collect();
        for qid in quest_ids {
            if let Some(def) = gd.quests.get(qid)
                && let Some(aq) = player.quest_log.get_active_mut(qid) {
                    aq.advance_level(level, def);
                }
        }
    }
}
