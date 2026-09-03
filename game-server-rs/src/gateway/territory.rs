use super::packets::*;
use super::GameSession;
use crate::error::HandlerResult;
use crate::gateway::WsSink;

impl GameSession {
    pub(crate) async fn handle_territory_list(
        &mut self,
        _entity_id: u32,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let lines: Vec<String> = {
            let Ok(mgr) = self.world.territories.try_lock() else {
                self.send_to_client(sink, super::packets::build_console_message("Intente de nuevo.")).await?;
                return Ok(());
            };
            let mut ids: Vec<_> = mgr.territories.keys().copied().collect();
            ids.sort();
            ids.iter()
                .filter_map(|tid| mgr.territory_info(*tid))
                .map(|info| format!("  {}", info))
                .collect()
        };

        self.send_to_client(sink, build_console_message("--- Territorios ---")).await?;
        for line in lines {
            self.send_to_client(sink, build_console_message(&line)).await?;
        }
        Ok(())
    }
}
