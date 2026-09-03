use super::packets::*;
use super::GameSession;
use crate::error::HandlerResult;
use crate::gateway::WsSink;

impl GameSession {
    pub(crate) async fn handle_pet_list(
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

        if player.pets.pets.is_empty() {
            self.send_to_client(sink, build_console_message("No tienes mascotas.")).await?;
            return Ok(());
        }

        self.send_to_client(sink, build_console_message("--- Tus Mascotas ---")).await?;
        for (i, pet) in player.pets.pets.iter().enumerate() {
            let status = if pet.active { " [ACTIVA]" } else if !pet.is_alive() { " [MUERTA]" } else { "" };
            self.send_to_client(sink, build_console_message(
                &format!("  {}: {} (Tipo:{}) Nv.{} HP:{}/{} Exp:{}{}", i, pet.name, pet.pet_type, pet.level, pet.hp, pet.max_hp, pet.exp, status),
            )).await?;
        }
        Ok(())
    }

    pub(crate) async fn handle_pet_summon(
        &mut self,
        index: usize,
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

        match player.pets.summon(index) {
            Ok(()) => {
                let name = player.pets.pets[index].name.clone();
                self.send_to_client(sink, build_console_message(
                    &format!("Has invocado a {}!", name),
                )).await?;
            }
            Err(e) => {
                self.send_to_client(sink, build_console_message(e)).await?;
            }
        }
        Ok(())
    }

    pub(crate) async fn handle_pet_dismiss(
        &mut self,
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

        player.pets.dismiss();
        self.send_to_client(sink, build_console_message("Mascota despachada.")).await?;
        Ok(())
    }

    pub(crate) async fn handle_pet_release(
        &mut self,
        index: usize,
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

        match player.pets.release(index) {
            Ok(name) => {
                self.send_to_client(sink, build_console_message(
                    &format!("Has liberado a {}.", name),
                )).await?;
            }
            Err(e) => {
                self.send_to_client(sink, build_console_message(e)).await?;
            }
        }
        Ok(())
    }
}
