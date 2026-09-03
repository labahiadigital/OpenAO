use super::packets::*;
use super::GameSession;
use crate::error::HandlerResult;
use crate::gateway::WsSink;
use crate::gameplay::achievements::default_achievements;

impl GameSession {
    pub(crate) async fn handle_achievements_list(
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

        let defs = default_achievements();
        let unlocked_count = player.achievements.unlocked.len();

        drop(player);
        drop(scene);

        self.send_to_client(sink, build_console_message(
            &format!("--- Logros ({}/{}) ---", unlocked_count, defs.len()),
        )).await?;

        let scene = match self.world.scenes.get(&mid) {
            Some(s) => s,
            None => return Ok(()),
        };
        let player = match scene.players.get(&entity_id) {
            Some(p) => p,
            None => return Ok(()),
        };

        let mut lines = Vec::new();
        for def in &defs {
            let status = if player.achievements.is_unlocked(def.id) { " [DESBLOQUEADO]" } else { "" };
            lines.push(format!("  #{}: {} - {}{}", def.id, def.name, def.description, status));
        }
        drop(player);
        drop(scene);

        for line in lines {
            self.send_to_client(sink, build_console_message(&line)).await?;
        }
        Ok(())
    }
}
