use crate::error::{GameError, GameErrorCode, HandlerResult};

use super::packets::*;
use super::GameSession;
use crate::gateway::WsSink;

impl GameSession {
    pub(super) async fn handle_deposit_bank_gold(
        &mut self,
        amount: i32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id, char_id) = match (self.entity_id, self.map_id, &self.character_id) {
            (Some(e), Some(m), Some(c)) => (e, m, c.clone()),
            _ => return Ok(()),
        };

        if !self.command_limiter.check("bank_deposit") {
            self.send_to_client(sink, GameError::new(GameErrorCode::RateLimited, "Banco: espera un momento.").to_console_packet()).await?;
            return Ok(());
        }

        if amount < 1 {
            return Ok(());
        }

        let scene = self.world.get_or_create_scene(map_id);
        let player_gold = scene.players.get(&entity_id).map(|p| p.gold).unwrap_or(0);

        let safe_amount = amount.min(player_gold);
        if safe_amount < 1 {
            let err = GameError::new(GameErrorCode::InsufficientGold, "No tienes suficiente oro.");
            self.send_to_client(sink, err.to_console_packet()).await?;
            return Ok(());
        }

        let bank_gold = self.world.db.get_bank_gold(&char_id).await?;
        let new_bank_gold = bank_gold + safe_amount;
        let new_player_gold = player_gold - safe_amount;

        self.world.db.set_bank_gold(&char_id, new_bank_gold).await?;

        let scene = self.world.get_or_create_scene(map_id);
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.gold = crate::gameplay::balance::clamp_gold(new_player_gold as i64) as i32;
        }

        self.send_to_client(sink, build_act_gold(new_player_gold)).await?;
        tracing::info!(
            target: "activity",
            category = "economy", action = "bank_deposit",
            player = ?self.character_name, gold_delta = -(safe_amount as i64),
            "BANK_DEPOSIT"
        );
        self.send_to_client(sink, build_console_message(&format!("Depositaste {} de oro en el banco. Banco: {}", safe_amount, new_bank_gold))).await?;
        Ok(())
    }

    pub(super) async fn handle_withdraw_bank_gold(
        &mut self,
        amount: i32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let (entity_id, map_id, char_id) = match (self.entity_id, self.map_id, &self.character_id) {
            (Some(e), Some(m), Some(c)) => (e, m, c.clone()),
            _ => return Ok(()),
        };

        if !self.command_limiter.check("bank_withdraw") {
            self.send_to_client(sink, GameError::new(GameErrorCode::RateLimited, "Banco: espera un momento.").to_console_packet()).await?;
            return Ok(());
        }

        if amount < 1 {
            return Ok(());
        }

        let bank_gold = self.world.db.get_bank_gold(&char_id).await?;
        let safe_amount = amount.min(bank_gold);
        if safe_amount < 1 {
            let err = GameError::new(GameErrorCode::InsufficientGold, "No tienes suficiente oro en el banco.");
            self.send_to_client(sink, err.to_console_packet()).await?;
            return Ok(());
        }

        let new_bank_gold = bank_gold - safe_amount;
        self.world.db.set_bank_gold(&char_id, new_bank_gold).await?;

        let scene = self.world.get_or_create_scene(map_id);
        if let Some(mut player) = scene.players.get_mut(&entity_id) {
            player.gold = crate::gameplay::balance::clamp_gold((player.gold + safe_amount) as i64) as i32;
            let new_gold = player.gold;
            drop(player);
            self.send_to_client(sink, build_act_gold(new_gold)).await?;
        }

        tracing::info!(
            target: "activity",
            category = "economy", action = "bank_withdraw",
            player = ?self.character_name, gold_delta = safe_amount,
            "BANK_WITHDRAW"
        );
        self.send_to_client(sink, build_console_message(&format!("Retiraste {} de oro del banco. Banco: {}", safe_amount, new_bank_gold))).await?;
        Ok(())
    }

    pub(super) async fn handle_reorder_bank(
        &mut self,
        source: u8,
        target: u8,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let char_id = match &self.character_id {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        let bank = self.world.db.load_bank(&char_id).await?;
        let source_item = bank.iter().find(|r| r.slot == source as i32).cloned();
        let target_item = bank.iter().find(|r| r.slot == target as i32).cloned();

        match (source_item, target_item) {
            (Some(src), Some(tgt)) => {
                self.world.db.update_bank_slot(&char_id, source as i32, tgt.item_id, tgt.amount).await?;
                self.world.db.update_bank_slot(&char_id, target as i32, src.item_id, src.amount).await?;
            }
            (Some(src), None) => {
                self.world.db.delete_bank_slot(&char_id, source as i32).await?;
                self.world.db.update_bank_slot(&char_id, target as i32, src.item_id, src.amount).await?;
            }
            _ => {}
        }

        let _ = sink;
        Ok(())
    }
}
