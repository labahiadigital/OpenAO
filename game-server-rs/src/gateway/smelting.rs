use crate::error::{GameError, GameErrorCode, HandlerResult};

use super::packets::*;
use super::GameSession;
use crate::gateway::WsSink;

impl GameSession {
    pub(super) async fn handle_smelt(
        &mut self,
        recipe_id: i32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if self.entity_id.is_none() {
            return Ok(());
        }

        if !self.command_limiter.check("smelt") {
            self.send_to_client(sink, GameError::new(GameErrorCode::RateLimited, "Fundición: espera un momento.").to_console_packet()).await?;
            return Ok(());
        }

        let recipe = match self.world.gd().smelting_recipes.iter().find(|r| r.id == recipe_id) {
            Some(r) => r.clone(),
            None => {
                self.send_to_client(sink, GameError::item_not_found("Receta de fundición").to_console_packet()).await?;
                return Ok(());
            }
        };

        let char_id = match &self.character_id {
            Some(id) => id.clone(),
            None => return Ok(()),
        };

        let inv = self.world.cache_get_inventory(&char_id);
        let mineral_count: i32 = inv.iter()
            .filter(|r| r.item_id == recipe.mineral_item_id)
            .map(|r| r.amount)
            .sum();

        if mineral_count < recipe.minerals_per_ingot {
            let gd = self.world.gd();
            let mineral_name = gd.get_object(recipe.mineral_item_id)
                .map(|o| o.name.as_str())
                .unwrap_or("minerales");
            self.send_to_client(sink, GameError::insufficient_items(recipe.minerals_per_ingot, mineral_name).to_console_packet()).await?;
            return Ok(());
        }

        self.world.cache_remove_items(&char_id, recipe.mineral_item_id, recipe.minerals_per_ingot);
        self.world.cache_add_item(&char_id, recipe.ingot_item_id, 1);

        let gd = self.world.gd();
        let ingot_name = gd.get_object(recipe.ingot_item_id)
            .map(|o| o.name.as_str())
            .unwrap_or("lingote");
        let msg = format!("Has fundido: {}", ingot_name);
        self.send_to_client(sink, build_console_message(&msg)).await?;

        self.send_full_inventory(sink).await?;

        Ok(())
    }
}
