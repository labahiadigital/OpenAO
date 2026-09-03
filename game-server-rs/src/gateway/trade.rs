use crate::error::HandlerResult;
use crate::gateway::packets::build_console_message;
use crate::gateway::WsSink;
use crate::world::{TradeOffer, TradeSession};

use super::GameSession;

impl GameSession {
    #[allow(dead_code)]
    pub(super) async fn handle_trade_request(
        &mut self,
        target_entity_id: u32,
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

        if entity_id == target_entity_id {
            self.send_to_client(sink, build_console_message("No puedes comerciar contigo mismo.")).await?;
            return Ok(());
        }

        if self.world.entity_trade.contains_key(&entity_id) {
            self.send_to_client(sink, build_console_message("Ya estás en una transacción.")).await?;
            return Ok(());
        }

        let scene = self.world.get_or_create_scene(map_id);
        let target_name = scene.players.get(&target_entity_id)
            .map(|p| p.name.clone());
        let my_name = scene.players.get(&entity_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "???".into());
        drop(scene);

        let Some(target_name) = target_name else {
            self.send_to_client(sink, build_console_message("Jugador no encontrado.")).await?;
            return Ok(());
        };

        if self.world.trade_requests.contains_key(&target_entity_id)
            && self.world.trade_requests.get(&target_entity_id).map(|r| *r.value()) == Some(entity_id)
        {
            let trade_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
            self.world.trade_requests.remove(&target_entity_id);
            self.world.trade_requests.remove(&entity_id);

            let session = TradeSession {
                player_a: entity_id,
                player_b: target_entity_id,
                offer_a: TradeOffer::default(),
                offer_b: TradeOffer::default(),
            };
            self.world.active_trades.insert(trade_id.clone(), session);
            self.world.entity_trade.insert(entity_id, trade_id.clone());
            self.world.entity_trade.insert(target_entity_id, trade_id);

            let scene = self.world.get_or_create_scene(map_id);
            scene.send_to_player(entity_id, build_console_message(&format!("Transacción iniciada con {}.", target_name)));
            scene.send_to_player(target_entity_id, build_console_message(&format!("Transacción iniciada con {}.", my_name)));
        } else {
            self.world.trade_requests.insert(entity_id, target_entity_id);

            self.send_to_client(sink, build_console_message(&format!("Solicitud de transacción enviada a {}.", target_name))).await?;

            let scene = self.world.get_or_create_scene(map_id);
            scene.send_to_player(target_entity_id, build_console_message(&format!("{} quiere comerciar contigo. Haz click en él/ella para aceptar.", my_name)));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub(super) async fn handle_trade_offer_gold(
        &mut self,
        gold: i32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let trade_id = match self.world.entity_trade.get(&entity_id) {
            Some(r) => r.value().clone(),
            None => {
                self.send_to_client(sink, build_console_message("No estás en una transacción.")).await?;
                return Ok(());
            }
        };

        if let Some(mut trade) = self.world.active_trades.get_mut(&trade_id) {
            let offer = if trade.player_a == entity_id {
                &mut trade.offer_a
            } else {
                &mut trade.offer_b
            };
            offer.gold = gold.max(0);
            offer.confirmed = false;

            let other_id = if trade.player_a == entity_id { trade.player_b } else { trade.player_a };
            let map_id = self.map_id.unwrap_or(1);
            let scene = self.world.get_or_create_scene(map_id);
            scene.send_to_player(other_id, build_console_message(&format!("La otra parte ofrece {} oro.", gold)));
        }

        Ok(())
    }

    pub(super) async fn handle_trade_confirm(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let trade_id = match self.world.entity_trade.get(&entity_id) {
            Some(r) => r.value().clone(),
            None => {
                self.send_to_client(sink, build_console_message("No estás en una transacción.")).await?;
                return Ok(());
            }
        };

        let should_execute = {
            if let Some(mut trade) = self.world.active_trades.get_mut(&trade_id) {
                if trade.player_a == entity_id {
                    trade.offer_a.confirmed = true;
                } else {
                    trade.offer_b.confirmed = true;
                }
                trade.offer_a.confirmed && trade.offer_b.confirmed
            } else {
                false
            }
        };

        if should_execute {
            self.execute_trade(&trade_id).await;
        } else {
            self.send_to_client(sink, build_console_message("Has confirmado la transacción. Esperando confirmación de la otra parte.")).await?;

            if let Some(trade) = self.world.active_trades.get(&trade_id) {
                let other_id = if trade.player_a == entity_id { trade.player_b } else { trade.player_a };
                let map_id = self.map_id.unwrap_or(1);
                let scene = self.world.get_or_create_scene(map_id);
                scene.send_to_player(other_id, build_console_message("La otra parte ha confirmado. Confirma tú también para completar."));
            }
        }

        Ok(())
    }

    pub(super) async fn handle_trade_cancel(
        &mut self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(id) => id,
            None => return Ok(()),
        };

        let trade_id = match self.world.entity_trade.get(&entity_id) {
            Some(r) => r.value().clone(),
            None => {
                self.world.trade_requests.remove(&entity_id);
                self.send_to_client(sink, build_console_message("Transacción cancelada.")).await?;
                return Ok(());
            }
        };

        if let Some((_, trade)) = self.world.active_trades.remove(&trade_id) {
            self.world.entity_trade.remove(&trade.player_a);
            self.world.entity_trade.remove(&trade.player_b);

            let other_id = if trade.player_a == entity_id { trade.player_b } else { trade.player_a };
            let map_id = self.map_id.unwrap_or(1);
            let scene = self.world.get_or_create_scene(map_id);
            scene.send_to_player(entity_id, build_console_message("Transacción cancelada."));
            scene.send_to_player(other_id, build_console_message("La otra parte canceló la transacción."));
        }

        Ok(())
    }

    async fn execute_trade(&self, trade_id: &str) {
        let Some((_, trade)) = self.world.active_trades.remove(trade_id) else { return };
        self.world.entity_trade.remove(&trade.player_a);
        self.world.entity_trade.remove(&trade.player_b);

        let map_id = self.map_id.unwrap_or(1);
        let scene = self.world.get_or_create_scene(map_id);

        let gold_a = trade.offer_a.gold;
        let gold_b = trade.offer_b.gold;

        if let Some(mut pa) = scene.players.get_mut(&trade.player_a) {
            pa.gold = crate::gameplay::balance::clamp_gold((pa.gold as i64) - (gold_a as i64) + (gold_b as i64)) as i32;
        }
        if let Some(mut pb) = scene.players.get_mut(&trade.player_b) {
            pb.gold = crate::gameplay::balance::clamp_gold((pb.gold as i64) - (gold_b as i64) + (gold_a as i64)) as i32;
        }

        scene.send_to_player(trade.player_a, build_console_message("Transacción completada con éxito."));
        scene.send_to_player(trade.player_b, build_console_message("Transacción completada con éxito."));

        if let Some(pa) = scene.players.get(&trade.player_a) {
            use crate::gateway::packets::build_act_gold;
            scene.send_to_player(trade.player_a, build_act_gold(pa.gold));
        }
        if let Some(pb) = scene.players.get(&trade.player_b) {
            use crate::gateway::packets::build_act_gold;
            scene.send_to_player(trade.player_b, build_act_gold(pb.gold));
        }
    }

    /// Called on disconnect to clean up trades
    pub(super) fn cleanup_trade(&self, entity_id: u32) {
        self.world.trade_requests.remove(&entity_id);

        if let Some((_, trade_id)) = self.world.entity_trade.remove(&entity_id)
            && let Some((_, trade)) = self.world.active_trades.remove(&trade_id) {
                let other_id = if trade.player_a == entity_id { trade.player_b } else { trade.player_a };
                self.world.entity_trade.remove(&other_id);

                if let Some(map_id) = self.map_id {
                    let scene = self.world.get_or_create_scene(map_id);
                    scene.send_to_player(other_id, build_console_message("La transacción fue cancelada porque el otro jugador se desconectó."));
                }
            }
    }
}
