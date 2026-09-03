use openao_protocol::PacketWriter;
use openao_protocol::opcodes::client_packet_id;

use crate::error::{GameError, HandlerResult};

use super::packets::*;
use super::GameSession;
use crate::gateway::WsSink;
use crate::world::{Clan, EntityId};

impl GameSession {
    pub(super) async fn handle_clan_command(
        &mut self,
        arg: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let parts: Vec<&str> = arg.splitn(2, ' ').collect();
        let subcmd = parts.first().map(|s| s.to_lowercase()).unwrap_or_default();

        match subcmd.as_str() {
            "crear" => {
                let clan_name = parts.get(1).unwrap_or(&"").trim();
                self.handle_clan_create(clan_name, entity_id, sink).await
            }
            "salir" => self.handle_clan_leave(entity_id, sink).await,
            "info" => self.handle_clan_info(entity_id, sink).await,
            "expulsar" => {
                let target_name = parts.get(1).unwrap_or(&"").trim();
                self.handle_clan_kick(target_name, entity_id, sink).await
            }
            "lider" => {
                let target_name = parts.get(1).unwrap_or(&"").trim();
                self.handle_clan_transfer_leader(target_name, entity_id, sink).await
            }
            "eliminar" => {
                let confirm = parts.get(1).unwrap_or(&"").trim();
                self.handle_clan_delete(confirm, entity_id, sink).await
            }
            "postular" => {
                let target_clan = parts.get(1).unwrap_or(&"").trim();
                self.handle_clan_apply(target_clan, entity_id, sink).await
            }
            "aceptar" => {
                let request_id = parts.get(1).unwrap_or(&"").trim();
                self.handle_clan_accept_request(request_id, entity_id, sink).await
            }
            "rechazar" => {
                let request_id = parts.get(1).unwrap_or(&"").trim();
                self.handle_clan_reject_request(request_id, entity_id, sink).await
            }
            "colider" => {
                let target_name = parts.get(1).unwrap_or(&"").trim();
                self.handle_clan_colider(target_name, entity_id, sink).await
            }
            _ => {
                self.send_to_client(sink, build_console_message(
                    "Uso: /clan [crear|salir|info|expulsar|lider|eliminar|postular|aceptar|rechazar|colider] arg"
                )).await?;
                Ok(())
            }
        }
    }

    async fn handle_clan_create(
        &mut self,
        name: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if name.is_empty() || name.len() < 3 {
            self.send_to_client(sink, build_console_message("El nombre del clan debe tener al menos 3 caracteres.")).await?;
            return Ok(());
        }

        if name.len() > 20 {
            self.send_to_client(sink, build_console_message("El nombre del clan no puede tener más de 20 caracteres.")).await?;
            return Ok(());
        }

        let my_clan = self.get_player_clan_id(entity_id);
        if my_clan.is_some() {
            self.send_to_client(sink, build_console_message("Ya estás en un clan. Sal primero con /clan salir.")).await?;
            return Ok(());
        }

        for clan_ref in self.world.clans.iter() {
            if clan_ref.name.eq_ignore_ascii_case(name) {
                self.send_to_client(sink, build_console_message("Ya existe un clan con ese nombre.")).await?;
                return Ok(());
            }
        }

        let my_name = self.character_name.clone().unwrap_or_else(|| "???".into());

        let clan_id = format!("clan-{}-{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(), entity_id);

        let clan = Clan {
            id: clan_id.clone(),
            name: name.to_string(),
            leader_id: entity_id,
            leader_name: my_name.clone(),
            member_ids: vec![entity_id],
            co_leader_ids: vec![],
        };
        self.world.clans.insert(clan_id.clone(), clan);

        self.set_player_clan_id(entity_id, Some(clan_id.clone()));

        self.send_to_client(sink, build_console_message(&format!("Clan '{}' creado exitosamente. Eres el líder.", name))).await?;
        self.send_clan_state_for_clan(&clan_id);
        Ok(())
    }

    async fn handle_clan_leave(
        &mut self,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let my_clan_id = match self.get_player_clan_id(entity_id) {
            Some(cid) => cid,
            None => {
                self.send_to_client(sink, GameError::not_in_clan().to_console_packet()).await?;
                return Ok(());
            }
        };

        self.set_player_clan_id(entity_id, None);

        let (is_leader, should_disband, _remaining, leader_id) = {
            if let Some(mut clan) = self.world.clans.get_mut(&my_clan_id) {
                let is_leader = clan.leader_id == entity_id;
                clan.member_ids.retain(|&id| id != entity_id);
                let should_disband = is_leader || clan.member_ids.is_empty();
                (is_leader, should_disband, clan.member_ids.clone(), clan.leader_id)
            } else {
                (false, true, vec![], 0)
            }
        };

        if should_disband {
            if let Some((_, clan)) = self.world.clans.remove(&my_clan_id) {
                for &mid in &clan.member_ids {
                    self.set_player_clan_id(mid, None);
                    self.send_empty_clan_state(mid);
                }
            }
            self.send_empty_clan_state(entity_id);
            let msg = if is_leader {
                "Disolviste el clan."
            } else {
                "Saliste del clan."
            };
            self.send_to_client(sink, build_console_message(msg)).await?;
        } else {
            self.send_empty_clan_state(entity_id);
            self.send_to_client(sink, build_console_message("Saliste del clan.")).await?;

            let _ = leader_id;
            self.send_clan_state_for_clan(&my_clan_id);
        }

        Ok(())
    }

    async fn handle_clan_info(
        &mut self,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let my_clan_id = match self.get_player_clan_id(entity_id) {
            Some(cid) => cid,
            None => {
                self.send_to_client(sink, GameError::not_in_clan().to_console_packet()).await?;
                return Ok(());
            }
        };

        if let Some(clan) = self.world.clans.get(&my_clan_id) {
            let member_names: Vec<String> = clan.member_ids.iter().filter_map(|&mid| {
                for scene_ref in self.world.scenes.iter() {
                    if let Some(p) = scene_ref.players.get(&mid) {
                        return Some(p.name.clone());
                    }
                }
                None
            }).collect();

            let info = format!(
                "Clan: {} | Líder: {} | Miembros ({}/10): {}",
                clan.name,
                clan.leader_name,
                member_names.len(),
                member_names.join(", "),
            );
            self.send_to_client(sink, build_console_message(&info)).await?;
        } else {
            self.send_to_client(sink, build_console_message("Tu clan ya no existe.")).await?;
            self.set_player_clan_id(entity_id, None);
        }

        Ok(())
    }

    fn get_player_clan_id(&self, entity_id: EntityId) -> Option<String> {
        for scene_ref in self.world.scenes.iter() {
            if let Some(p) = scene_ref.players.get(&entity_id) {
                return p.clan_id.clone();
            }
        }
        None
    }

    fn set_player_clan_id(&self, entity_id: EntityId, clan_id: Option<String>) {
        for scene_ref in self.world.scenes.iter() {
            if let Some(mut p) = scene_ref.players.get_mut(&entity_id) {
                p.clan_id = clan_id;
                return;
            }
        }
    }

    fn send_empty_clan_state(&self, entity_id: EntityId) {
        let delta = serde_json::json!({ "upsert": [], "remove": [] });
        let mut pw = PacketWriter::new();
        pw.write_byte(client_packet_id::CLAN_STATE);
        pw.write_string(&delta.to_string());
        let data = pw.into_bytes();

        for scene_ref in self.world.scenes.iter() {
            if let Some(tx) = scene_ref.personal_tx.get(&entity_id) {
                let _ = tx.send(data);
                return;
            }
        }
    }

    async fn handle_clan_kick(
        &mut self,
        target_name: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if target_name.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /clan expulsar nombre")).await?;
            return Ok(());
        }
        let my_clan_id = match self.get_player_clan_id(entity_id) {
            Some(cid) => cid,
            None => {
                self.send_to_client(sink, GameError::not_in_clan().to_console_packet()).await?;
                return Ok(());
            }
        };
        let is_leader = self.world.clans.get(&my_clan_id)
            .map(|c| c.leader_id == entity_id)
            .unwrap_or(false);
        if !is_leader {
            self.send_to_client(sink, GameError::not_clan_leader().to_console_packet()).await?;
            return Ok(());
        }
        let target_id = self.find_player_by_name(target_name);
        let target_id = match target_id {
            Some(id) => id,
            None => {
                self.send_to_client(sink, GameError::player_not_found(target_name).to_console_packet()).await?;
                return Ok(());
            }
        };
        if target_id == entity_id {
            self.send_to_client(sink, build_console_message("No puedes expulsarte a ti mismo. Usa /clan salir.")).await?;
            return Ok(());
        }
        if let Some(mut clan) = self.world.clans.get_mut(&my_clan_id) {
            clan.member_ids.retain(|&id| id != target_id);
        }
        self.set_player_clan_id(target_id, None);
        self.send_empty_clan_state(target_id);
        for scene_ref in self.world.scenes.iter() {
            if let Some(tx) = scene_ref.personal_tx.get(&target_id) {
                let _ = tx.send(build_console_message("Fuiste expulsado del clan."));
                break;
            }
        }
        self.send_to_client(sink, build_console_message("Miembro expulsado del clan.")).await?;
        self.send_clan_state_for_clan(&my_clan_id);
        Ok(())
    }

    async fn handle_clan_transfer_leader(
        &mut self,
        target_name: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if target_name.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /clan lider nombre")).await?;
            return Ok(());
        }
        let my_clan_id = match self.get_player_clan_id(entity_id) {
            Some(cid) => cid,
            None => {
                self.send_to_client(sink, GameError::not_in_clan().to_console_packet()).await?;
                return Ok(());
            }
        };
        let is_leader = self.world.clans.get(&my_clan_id)
            .map(|c| c.leader_id == entity_id)
            .unwrap_or(false);
        if !is_leader {
            self.send_to_client(sink, GameError::not_clan_leader().to_console_packet()).await?;
            return Ok(());
        }
        let target_id = match self.find_player_by_name(target_name) {
            Some(id) => id,
            None => {
                self.send_to_client(sink, GameError::player_not_found(target_name).to_console_packet()).await?;
                return Ok(());
            }
        };
        let target_in_clan = self.get_player_clan_id(target_id)
            .map(|cid| cid == my_clan_id)
            .unwrap_or(false);
        if !target_in_clan {
            self.send_to_client(sink, build_console_message("Ese jugador no está en tu clan.")).await?;
            return Ok(());
        }
        if let Some(mut clan) = self.world.clans.get_mut(&my_clan_id) {
            clan.leader_id = target_id;
            clan.leader_name = target_name.to_string();
        }
        self.send_to_client(sink, build_console_message(&format!("Liderazgo transferido a {}.", target_name))).await?;
        for scene_ref in self.world.scenes.iter() {
            if let Some(tx) = scene_ref.personal_tx.get(&target_id) {
                let _ = tx.send(build_console_message("Ahora eres el líder del clan."));
                break;
            }
        }
        self.send_clan_state_for_clan(&my_clan_id);
        Ok(())
    }

    async fn handle_clan_delete(
        &mut self,
        confirm: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if !confirm.eq_ignore_ascii_case("confirmar") {
            self.send_to_client(sink, build_console_message("Uso: /clan eliminar confirmar")).await?;
            return Ok(());
        }
        let my_clan_id = match self.get_player_clan_id(entity_id) {
            Some(cid) => cid,
            None => {
                self.send_to_client(sink, GameError::not_in_clan().to_console_packet()).await?;
                return Ok(());
            }
        };
        let is_leader = self.world.clans.get(&my_clan_id)
            .map(|c| c.leader_id == entity_id)
            .unwrap_or(false);
        if !is_leader {
            self.send_to_client(sink, GameError::not_clan_leader().to_console_packet()).await?;
            return Ok(());
        }
        if let Some((_, clan)) = self.world.clans.remove(&my_clan_id) {
            for &mid in &clan.member_ids {
                self.set_player_clan_id(mid, None);
                self.send_empty_clan_state(mid);
                if mid != entity_id {
                    for scene_ref in self.world.scenes.iter() {
                        if let Some(tx) = scene_ref.personal_tx.get(&mid) {
                            let _ = tx.send(build_console_message("Tu clan fue eliminado por su líder."));
                            break;
                        }
                    }
                }
            }
        }
        self.send_to_client(sink, build_console_message("Clan eliminado correctamente.")).await?;
        Ok(())
    }

    fn find_player_by_name(&self, name: &str) -> Option<EntityId> {
        let name_lower = name.to_lowercase();
        for scene_ref in self.world.scenes.iter() {
            for entry in scene_ref.players.iter() {
                if entry.value().name.to_lowercase() == name_lower {
                    return Some(entry.value().id);
                }
            }
        }
        None
    }

    async fn handle_clan_apply(
        &mut self,
        target_clan_name: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if target_clan_name.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /clan postular nombreClan")).await?;
            return Ok(());
        }
        if self.get_player_clan_id(entity_id).is_some() {
            self.send_to_client(sink, GameError::new(crate::error::GameErrorCode::AlreadyInClan, "Ya estás en un clan. Sal primero con /clan salir.").to_console_packet()).await?;
            return Ok(());
        }
        let clan_id = {
            let mut found = None;
            for entry in self.world.clans.iter() {
                if entry.value().name.eq_ignore_ascii_case(target_clan_name) {
                    found = Some(entry.key().clone());
                    break;
                }
            }
            found
        };
        let clan_id = match clan_id {
            Some(cid) => cid,
            None => {
                self.send_to_client(sink, build_console_message("No se encontró un clan con ese nombre.")).await?;
                return Ok(());
            }
        };
        for entry in self.world.clan_requests.iter() {
            if entry.value().applicant_id == entity_id && entry.value().clan_id == clan_id {
                self.send_to_client(sink, build_console_message("Ya tienes una solicitud pendiente en ese clan.")).await?;
                return Ok(());
            }
        }
        let my_name = self.character_name.clone().unwrap_or_else(|| "???".into());
        let req_id = format!("cr-{}-{}", entity_id, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
        let request = crate::world::ClanRequest {
            id: req_id.clone(),
            applicant_id: entity_id,
            applicant_name: my_name.clone(),
            clan_id: clan_id.clone(),
            message: String::new(),
        };
        self.world.clan_requests.insert(req_id.clone(), request);
        self.send_to_client(sink, build_console_message("Solicitud enviada al clan.")).await?;
        if let Some(clan) = self.world.clans.get(&clan_id) {
            let leader_id = clan.leader_id;
            let msg = build_console_message(&format!("{} solicita unirse al clan. Usa /clan aceptar {} o /clan rechazar {}", my_name, req_id, req_id));
            for scene_ref in self.world.scenes.iter() {
                if let Some(tx) = scene_ref.personal_tx.get(&leader_id) {
                    let _ = tx.send(msg.clone());
                    break;
                }
            }
            for &co_id in &clan.co_leader_ids {
                for scene_ref in self.world.scenes.iter() {
                    if let Some(tx) = scene_ref.personal_tx.get(&co_id) {
                        let _ = tx.send(msg.clone());
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_clan_accept_request(
        &mut self,
        request_id: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if request_id.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /clan aceptar requestId")).await?;
            return Ok(());
        }
        let my_clan_id = match self.get_player_clan_id(entity_id) {
            Some(cid) => cid,
            None => {
                self.send_to_client(sink, GameError::not_in_clan().to_console_packet()).await?;
                return Ok(());
            }
        };
        let is_leader_or_co = self.world.clans.get(&my_clan_id)
            .map(|c| c.leader_id == entity_id || c.co_leader_ids.contains(&entity_id))
            .unwrap_or(false);
        if !is_leader_or_co {
            self.send_to_client(sink, GameError::not_clan_leader().to_console_packet()).await?;
            return Ok(());
        }
        let request = match self.world.clan_requests.remove(request_id) {
            Some((_, req)) => req,
            None => {
                self.send_to_client(sink, GameError::item_not_found("Solicitud").to_console_packet()).await?;
                return Ok(());
            }
        };
        if request.clan_id != my_clan_id {
            self.send_to_client(sink, build_console_message("Esa solicitud no es para tu clan.")).await?;
            return Ok(());
        }
        if let Some(mut clan) = self.world.clans.get_mut(&my_clan_id) {
            clan.member_ids.push(request.applicant_id);
        }
        self.set_player_clan_id(request.applicant_id, Some(my_clan_id.clone()));
        let msg = build_console_message("Fuiste aceptado en el clan.");
        for scene_ref in self.world.scenes.iter() {
            if let Some(tx) = scene_ref.personal_tx.get(&request.applicant_id) {
                let _ = tx.send(msg);
                break;
            }
        }
        self.send_to_client(sink, build_console_message("Solicitud aceptada.")).await?;
        self.send_clan_state_for_clan(&my_clan_id);
        Ok(())
    }

    async fn handle_clan_reject_request(
        &mut self,
        request_id: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if request_id.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /clan rechazar requestId")).await?;
            return Ok(());
        }
        let my_clan_id = match self.get_player_clan_id(entity_id) {
            Some(cid) => cid,
            None => {
                self.send_to_client(sink, GameError::not_in_clan().to_console_packet()).await?;
                return Ok(());
            }
        };
        let is_leader_or_co = self.world.clans.get(&my_clan_id)
            .map(|c| c.leader_id == entity_id || c.co_leader_ids.contains(&entity_id))
            .unwrap_or(false);
        if !is_leader_or_co {
            self.send_to_client(sink, GameError::not_clan_leader().to_console_packet()).await?;
            return Ok(());
        }
        let request = match self.world.clan_requests.remove(request_id) {
            Some((_, req)) => req,
            None => {
                self.send_to_client(sink, GameError::item_not_found("Solicitud").to_console_packet()).await?;
                return Ok(());
            }
        };
        let msg = build_console_message("Tu solicitud de clan fue rechazada.");
        for scene_ref in self.world.scenes.iter() {
            if let Some(tx) = scene_ref.personal_tx.get(&request.applicant_id) {
                let _ = tx.send(msg);
                break;
            }
        }
        self.send_to_client(sink, build_console_message("Solicitud rechazada.")).await?;
        Ok(())
    }

    async fn handle_clan_colider(
        &mut self,
        target_name: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        if target_name.is_empty() {
            self.send_to_client(sink, build_console_message("Uso: /clan colider nombre")).await?;
            return Ok(());
        }
        let my_clan_id = match self.get_player_clan_id(entity_id) {
            Some(cid) => cid,
            None => {
                self.send_to_client(sink, GameError::not_in_clan().to_console_packet()).await?;
                return Ok(());
            }
        };
        let is_leader = self.world.clans.get(&my_clan_id)
            .map(|c| c.leader_id == entity_id)
            .unwrap_or(false);
        if !is_leader {
            self.send_to_client(sink, GameError::not_clan_leader().to_console_packet()).await?;
            return Ok(());
        }
        let target_id = match self.find_player_by_name(target_name) {
            Some(id) => id,
            None => {
                self.send_to_client(sink, GameError::player_not_found(target_name).to_console_packet()).await?;
                return Ok(());
            }
        };
        let target_in_clan = self.get_player_clan_id(target_id)
            .map(|cid| cid == my_clan_id)
            .unwrap_or(false);
        if !target_in_clan {
            self.send_to_client(sink, build_console_message("Ese jugador no está en tu clan.")).await?;
            return Ok(());
        }
        if let Some(mut clan) = self.world.clans.get_mut(&my_clan_id) {
            if clan.co_leader_ids.contains(&target_id) {
                clan.co_leader_ids.retain(|&id| id != target_id);
                drop(clan);
                self.send_to_client(sink, build_console_message(&format!("{} ya no es co-líder.", target_name))).await?;
                let msg = build_console_message("Ya no eres co-líder del clan.");
                for scene_ref in self.world.scenes.iter() {
                    if let Some(tx) = scene_ref.personal_tx.get(&target_id) {
                        let _ = tx.send(msg);
                        break;
                    }
                }
            } else {
                clan.co_leader_ids.push(target_id);
                drop(clan);
                self.send_to_client(sink, build_console_message(&format!("{} es ahora co-líder.", target_name))).await?;
                let msg = build_console_message("Ahora eres co-líder del clan.");
                for scene_ref in self.world.scenes.iter() {
                    if let Some(tx) = scene_ref.personal_tx.get(&target_id) {
                        let _ = tx.send(msg);
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn send_clan_state_for_clan(&self, clan_id: &str) {
        let clan = match self.world.clans.get(clan_id) {
            Some(c) => c,
            None => return,
        };

        let mut upsert_items = Vec::new();
        for &mid in &clan.member_ids {
            for scene_ref in self.world.scenes.iter() {
                if let Some(p) = scene_ref.players.get(&mid) {
                    upsert_items.push(serde_json::json!({
                        "id": mid,
                        "nameCharacter": p.name,
                        "map": p.pos.map,
                        "pos": { "x": p.pos.x, "y": p.pos.y },
                        "online": true,
                    }));
                    break;
                }
            }
        }

        let delta = serde_json::json!({
            "upsert": upsert_items,
            "remove": [],
        });

        let mut pw = PacketWriter::new();
        pw.write_byte(client_packet_id::CLAN_STATE);
        pw.write_string(&delta.to_string());
        let data = pw.into_bytes();

        for &mid in &clan.member_ids {
            for scene_ref in self.world.scenes.iter() {
                if let Some(tx) = scene_ref.personal_tx.get(&mid) {
                    let _ = tx.send(data.clone());
                    break;
                }
            }
        }
    }
}
