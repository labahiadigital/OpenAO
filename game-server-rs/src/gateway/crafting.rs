use super::packets::*;
use super::GameSession;
use crate::error::{GameError, GameErrorCode, HandlerResult};
use crate::gateway::WsSink;

impl GameSession {
    pub(super) async fn handle_craft_item(
        &mut self,
        recipe_id: i32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if self.entity_id.is_none() || self.map_id.is_none() {
            return Ok(());
        }

        if !self.command_limiter.check("craft") {
            self.send_to_client(sink, GameError::new(GameErrorCode::RateLimited, "Crafteo: espera un momento.").to_console_packet()).await?;
            return Ok(());
        }

        let recipe = match self.world.gd().crafting_recipes.iter().find(|r| r.id == recipe_id) {
            Some(r) => r.clone(),
            None => {
                self.send_to_client(sink, GameError::item_not_found("Receta").to_console_packet()).await?;
                return Ok(());
            }
        };

        let char_id = match &self.character_id {
            Some(id) => id.clone(),
            None => return Ok(()),
        };

        let player_inv = self.world.cache_get_inventory(&char_id);

        for mat in &recipe.materials {
            let total: i32 = player_inv.iter()
                .filter(|inv| inv.item_id == mat.item_id)
                .map(|inv| inv.amount)
                .sum();
            if total < mat.amount {
                let gd = self.world.gd();
                let mat_name = gd.get_object(mat.item_id)
                    .map(|o| o.name.as_str())
                    .unwrap_or("material");
                self.send_to_client(sink, GameError::insufficient_items(mat.amount, mat_name).to_console_packet()).await?;
                return Ok(());
            }
        }

        for mat in &recipe.materials {
            self.world.cache_remove_items(&char_id, mat.item_id, mat.amount);
        }

        self.world.cache_add_item(&char_id, recipe.item_id, 1);

        let item_data = crate::replication::get_item_data(&self.world.gd(), recipe.item_id);
        let msg = format!("Has creado: {}", item_data.name);
        self.send_to_client(sink, build_console_message(&msg)).await?;

        self.send_full_inventory(sink).await?;

        Ok(())
    }

    pub(super) async fn send_full_inventory(
        &self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let char_id = match &self.character_id {
            Some(id) => id.clone(),
            None => return Ok(()),
        };

        let inv = self.world.cache_get_inventory(&char_id);
        for row in &inv {
            let item_data = crate::replication::get_item_data(&self.world.gd(), row.item_id);
            let pkt = crate::replication::build_inv_item_packet(row, &item_data);
            self.send_to_client(sink, pkt).await?;
        }
        Ok(())
    }
}
