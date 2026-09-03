use std::time::Duration;

use crate::error::{GameError, HandlerResult};

use super::packets::*;
use super::GameSession;
use crate::gateway::WsSink;
use crate::world::{EntityId, Party, PartyInvite};

#[allow(unused_imports)]
use openao_protocol::opcodes::client_packet_id;

const PARTY_INVITATION_TIMEOUT: Duration = Duration::from_secs(30);

impl GameSession {
    pub(super) async fn handle_party_invite(
        &mut self,
        target_name: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        let my_faction = {
            let scene = self.world.get_or_create_scene(map_id);
            match scene.players.get(&entity_id) {
                Some(p) => p.faction.clone(),
                None => return Ok(()),
            }
        };

        let mut target_eid: Option<EntityId> = None;
        let mut rejection_msg: Option<String> = None;

        for scene_ref in self.world.scenes.iter() {
            for player_ref in scene_ref.players.iter() {
                if player_ref.name.eq_ignore_ascii_case(target_name) {
                    if player_ref.id == entity_id {
                        rejection_msg = Some("No puedes invitarte a ti mismo.".into());
                    } else if player_ref.faction != my_faction {
                        rejection_msg = Some("Solo puedes hacer party con personas de tu misma facción.".into());
                    } else if player_ref.party_id.is_some() {
                        rejection_msg = Some(format!("{} ya está en una party.", player_ref.name));
                    } else {
                        target_eid = Some(player_ref.id);
                    }
                    break;
                }
            }
            if target_eid.is_some() || rejection_msg.is_some() {
                break;
            }
        }

        if let Some(msg) = rejection_msg {
            self.send_to_client(sink, build_console_message(&msg)).await?;
            return Ok(());
        }

        let target_eid = match target_eid {
            Some(id) => id,
            None => {
                self.send_to_client(sink, GameError::player_not_found(target_name).to_console_packet()).await?;
                return Ok(());
            }
        };

        let my_party_id = {
            let scene = self.world.get_or_create_scene(map_id);
            scene.players.get(&entity_id).and_then(|p| p.party_id.clone())
        };

        if let Some(ref pid) = my_party_id
            && let Some(party) = self.world.parties.get(pid)
        {
            if party.leader_id != entity_id {
                self.send_to_client(sink, GameError::not_party_leader().to_console_packet()).await?;
                return Ok(());
            }
            if party.is_full() {
                self.send_to_client(sink, GameError::party_full().to_console_packet()).await?;
                return Ok(());
            }
        }

        self.world.party_invites.insert(target_eid, PartyInvite {
            inviter_id: entity_id,
            expires_at: std::time::Instant::now() + PARTY_INVITATION_TIMEOUT,
        });

        let invite_msg = build_console_message(&format!(
            "{} te invitó a su party. Escribe /aceptar para unirte.",
            self.character_name.as_deref().unwrap_or("???")
        ));
        for scene_ref in self.world.scenes.iter() {
            if let Some(tx) = scene_ref.personal_tx.get(&target_eid) {
                let _ = tx.send(invite_msg);
                break;
            }
        }

        self.send_to_client(sink, build_console_message(&format!("{} recibió la invitación a tu party.", target_name))).await?;
        Ok(())
    }

    pub(super) async fn handle_party_accept(
        &mut self,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let invite = self.world.party_invites.remove(&entity_id);

        let (_, invite) = match invite {
            Some(pair) => pair,
            None => {
                self.send_to_client(sink, build_console_message("No tienes ninguna invitación de party pendiente.")).await?;
                return Ok(());
            }
        };

        if std::time::Instant::now() > invite.expires_at {
            self.send_to_client(sink, build_console_message("La invitación expiró.")).await?;
            return Ok(());
        }

        let inviter_eid = invite.inviter_id;

        let my_party = {
            let mut found = None;
            for scene_ref in self.world.scenes.iter() {
                if let Some(p) = scene_ref.players.get(&entity_id) {
                    found = p.party_id.clone();
                    break;
                }
            }
            found
        };

        if my_party.is_some() {
            self.send_to_client(sink, build_console_message("Ya estás en una party.")).await?;
            return Ok(());
        }

        let inviter_party_id: Option<String> = {
            let mut found = None;
            for scene_ref in self.world.scenes.iter() {
                if let Some(p) = scene_ref.players.get(&inviter_eid) {
                    found = p.party_id.clone();
                    break;
                }
            }
            found
        };

        let inviter_name: String = {
            let mut name = String::from("???");
            for scene_ref in self.world.scenes.iter() {
                if let Some(p) = scene_ref.players.get(&inviter_eid) {
                    name = p.name.clone();
                    break;
                }
            }
            name
        };

        let party_id = if let Some(pid) = inviter_party_id {
            if let Some(mut party) = self.world.parties.get_mut(&pid) {
                if party.is_full() {
                    self.send_to_client(sink, GameError::party_full().to_console_packet()).await?;
                    return Ok(());
                }
                if party.leader_id != inviter_eid {
                    self.send_to_client(sink, build_console_message("La invitación ya no es válida.")).await?;
                    return Ok(());
                }
                party.member_ids.push(entity_id);
            }
            pid
        } else {
            let pid = format!("party-{}-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(), inviter_eid);

            let party = Party {
                id: pid.clone(),
                leader_id: inviter_eid,
                member_ids: vec![inviter_eid, entity_id],
            };
            self.world.parties.insert(pid.clone(), party);

            for scene_ref in self.world.scenes.iter() {
                if let Some(mut p) = scene_ref.players.get_mut(&inviter_eid) {
                    p.party_id = Some(pid.clone());
                    break;
                }
            }
            pid
        };

        for scene_ref in self.world.scenes.iter() {
            if let Some(mut p) = scene_ref.players.get_mut(&entity_id) {
                p.party_id = Some(party_id.clone());
                break;
            }
        }

        self.send_to_client(sink, build_console_message(&format!("Te uniste a la party de {}.", inviter_name))).await?;

        if let Some(party) = self.world.parties.get(&party_id) {
            let member_ids = party.member_ids.clone();
            let leader = party.leader_id;
            drop(party);
            self.sync_party_state(&party_id, &member_ids, leader);
        }

        Ok(())
    }

    pub(super) async fn handle_party_leave(
        &mut self,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let my_party_id = {
            let mut found = None;
            for scene_ref in self.world.scenes.iter() {
                if let Some(p) = scene_ref.players.get(&entity_id) {
                    found = p.party_id.clone();
                    break;
                }
            }
            found
        };

        let party_id = match my_party_id {
            Some(pid) => pid,
            None => {
                self.send_to_client(sink, build_console_message("No estás en una party.")).await?;
                return Ok(());
            }
        };

        self.clear_player_party(entity_id);

        let (is_leader, should_disband, remaining_members, leader_id) = {
            if let Some(mut party) = self.world.parties.get_mut(&party_id) {
                let is_leader = party.leader_id == entity_id;
                party.member_ids.retain(|&id| id != entity_id);
                let should_disband = is_leader || party.member_ids.len() <= 1;
                (is_leader, should_disband, party.member_ids.clone(), party.leader_id)
            } else {
                (false, true, vec![], 0)
            }
        };

        if should_disband {
            if let Some((_, party)) = self.world.parties.remove(&party_id) {
                for &mid in &party.member_ids {
                    self.clear_player_party(mid);
                    self.send_empty_party_state(mid);
                }
            }
            self.send_empty_party_state(entity_id);
            let msg = if is_leader {
                "Disolviste la party."
            } else {
                "Saliste de la party."
            };
            self.send_to_client(sink, build_console_message(msg)).await?;
        } else {
            self.send_empty_party_state(entity_id);
            self.send_to_client(sink, build_console_message("Saliste de la party.")).await?;
            self.sync_party_state(&party_id, &remaining_members, leader_id);
        }

        Ok(())
    }

    pub(super) async fn handle_party_kick(
        &mut self,
        target_name: &str,
        entity_id: EntityId,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let my_party_id = {
            let mut found = None;
            for scene_ref in self.world.scenes.iter() {
                if let Some(p) = scene_ref.players.get(&entity_id) {
                    found = p.party_id.clone();
                    break;
                }
            }
            found
        };

        let party_id = match my_party_id {
            Some(pid) => pid,
            None => {
                self.send_to_client(sink, build_console_message("No estás en una party.")).await?;
                return Ok(());
            }
        };

        if let Some(party) = self.world.parties.get(&party_id)
            && party.leader_id != entity_id
        {
            self.send_to_client(sink, GameError::not_party_leader().to_console_packet()).await?;
            return Ok(());
        }

        let target_eid: Option<EntityId> = {
            let mut found = None;
            if let Some(party) = self.world.parties.get(&party_id) {
                for &mid in &party.member_ids {
                    for scene_ref in self.world.scenes.iter() {
                        if let Some(p) = scene_ref.players.get(&mid)
                            && p.name.eq_ignore_ascii_case(target_name)
                        {
                            found = Some(mid);
                            break;
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }
            }
            found
        };

        let target_eid = match target_eid {
            Some(id) => id,
            None => {
                self.send_to_client(sink, build_console_message(&format!("{} no está en tu party.", target_name))).await?;
                return Ok(());
            }
        };

        if target_eid == entity_id {
            self.send_to_client(sink, build_console_message("No puedes echarte a ti mismo. Usa /salirparty para cerrar la party.")).await?;
            return Ok(());
        }

        self.clear_player_party(target_eid);

        let (should_disband, remaining_members, leader_id) = {
            if let Some(mut party) = self.world.parties.get_mut(&party_id) {
                party.member_ids.retain(|&id| id != target_eid);
                let should_disband = party.member_ids.len() <= 1;
                (should_disband, party.member_ids.clone(), party.leader_id)
            } else {
                (true, vec![], 0)
            }
        };

        self.send_empty_party_state(target_eid);
        let kick_msg = build_console_message("Fuiste expulsado de la party.");
        for scene_ref in self.world.scenes.iter() {
            if let Some(tx) = scene_ref.personal_tx.get(&target_eid) {
                let _ = tx.send(kick_msg);
                break;
            }
        }

        if should_disband {
            if let Some((_, party)) = self.world.parties.remove(&party_id) {
                for &mid in &party.member_ids {
                    self.clear_player_party(mid);
                    self.send_empty_party_state(mid);
                }
            }
        } else {
            self.sync_party_state(&party_id, &remaining_members, leader_id);
        }

        self.send_to_client(sink, build_console_message(&format!("{} fue expulsado de la party.", target_name))).await?;
        Ok(())
    }
}
