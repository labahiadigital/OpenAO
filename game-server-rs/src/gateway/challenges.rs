use openao_protocol::PacketWriter;
use openao_protocol::opcodes::client_packet_id;

use super::packets::build_console_message;
use super::GameSession;
use super::WsSink;
use crate::error::{GameError, GameErrorCode, HandlerResult};
use crate::gameplay::rooms::ChallengeParticipantData;

impl GameSession {
    pub(super) async fn handle_retos_action(
        &mut self,
        payload_str: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let entity_id = match self.entity_id {
            Some(e) => e,
            None => return Ok(()),
        };
        let char_id = match &self.character_id {
            Some(c) => c.clone(),
            None => return Ok(()),
        };
        let map_id = match self.map_id {
            Some(m) => m,
            None => return Ok(()),
        };

        if !self.command_limiter.check("retos") {
            self.send_to_client(sink, GameError::new(GameErrorCode::RateLimited, "Retos: espera un momento.").to_console_packet()).await?;
            return Ok(());
        }

        let payload: serde_json::Value = serde_json::from_str(payload_str).unwrap_or_default();
        let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("");

        match action {
            "refresh" => {
                self.send_retos_state(sink).await?;
            }
            "create" => {
                let team_size: usize = match payload.get("teamSize").and_then(|v| v.as_i64()).unwrap_or(1) {
                    2 => 2,
                    _ => 1,
                };

                let scene = self.world.get_or_create_scene(map_id);
                let (name, level, clase) = scene.players.get(&entity_id)
                    .map(|p| (p.name.clone(), p.level, p.id_clase))
                    .unwrap_or_else(|| ("???".to_string(), 1, 1));
                drop(scene);

                let class_name = crate::replication::get_class_name(clase);

                let data = ChallengeParticipantData {
                    character_id: uuid::Uuid::parse_str(&char_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                    name: name.clone(),
                    level,
                    class_name: class_name.to_string(),
                    race_name: String::new(),
                };

                let now = chrono::Utc::now().timestamp_millis();
                let result = {
                    let Ok(mut mgr) = self.world.challenges.try_lock() else {
                        self.send_to_client(sink, build_console_message("Intente de nuevo.")).await?;
                        return Ok(());
                    };
                    mgr.create(team_size, entity_id, data, now)
                };

                match result {
                    Ok(cid) => {
                        self.send_to_client(sink, build_console_message(&format!("Reto creado ({}). Esperando oponentes...", cid))).await?;
                    }
                    Err(e) => {
                        self.send_to_client(sink, build_console_message(&format!("Error al crear reto: {}", e))).await?;
                    }
                }
                self.send_retos_state(sink).await?;
            }
            "join" => {
                let challenge_id_str = payload.get("challengeId").and_then(|v| v.as_str()).unwrap_or("");
                let Ok(challenge_uuid) = uuid::Uuid::parse_str(challenge_id_str) else {
                    self.send_to_client(sink, GameError::new(GameErrorCode::InvalidSlot, "ID de reto inválido.").to_console_packet()).await?;
                    return Ok(());
                };

                let scene = self.world.get_or_create_scene(map_id);
                let (name, level, clase) = scene.players.get(&entity_id)
                    .map(|p| (p.name.clone(), p.level, p.id_clase))
                    .unwrap_or_else(|| ("???".to_string(), 1, 1));
                drop(scene);

                let class_name = crate::replication::get_class_name(clase);

                let data = ChallengeParticipantData {
                    character_id: uuid::Uuid::parse_str(&char_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
                    name: name.clone(),
                    level,
                    class_name: class_name.to_string(),
                    race_name: String::new(),
                };

                let join_result = {
                    let Ok(mut mgr) = self.world.challenges.try_lock() else {
                        self.send_to_client(sink, build_console_message("Intente de nuevo.")).await?;
                        return Ok(());
                    };
                    mgr.join(challenge_uuid, entity_id, data)
                };

                match join_result {
                    Ok(is_full) => {
                        if is_full {
                            self.send_to_client(sink, build_console_message("El reto está listo. ¡Comienza la pelea!")).await?;
                        } else {
                            self.send_to_client(sink, build_console_message("Te has unido al reto. Esperando más participantes...")).await?;
                        }
                    }
                    Err(e) => {
                        self.send_to_client(sink, build_console_message(&format!("Error: {}", e))).await?;
                    }
                }

                self.send_retos_state(sink).await?;
            }
            "cancel" => {
                let challenge_id_str = payload.get("challengeId").and_then(|v| v.as_str()).unwrap_or("");
                let Ok(challenge_uuid) = uuid::Uuid::parse_str(challenge_id_str) else {
                    self.send_to_client(sink, GameError::new(GameErrorCode::InvalidSlot, "ID de reto inválido.").to_console_packet()).await?;
                    return Ok(());
                };

                let removed = {
                    let Ok(mut mgr) = self.world.challenges.try_lock() else {
                        self.send_to_client(sink, build_console_message("Intente de nuevo.")).await?;
                        return Ok(());
                    };
                    mgr.cancel(challenge_uuid)
                };

                if removed {
                    self.send_to_client(sink, build_console_message("Reto cancelado.")).await?;
                } else {
                    self.send_to_client(sink, GameError::item_not_found("Reto").to_console_packet()).await?;
                }

                self.send_retos_state(sink).await?;
            }
            _ => {
                self.send_to_client(sink, GameError::new(GameErrorCode::NotImplemented, "Acción de reto no reconocida.").to_console_packet()).await?;
            }
        }

        Ok(())
    }

    async fn send_retos_state(
        &self,
        sink: &mut WsSink,
    ) -> HandlerResult {
        let json = {
            let Ok(mgr) = self.world.challenges.try_lock() else {
                self.send_to_client(sink, build_console_message("Intente de nuevo.")).await?;
                return Ok(());
            };
            let rooms = mgr.list_all();

            let mut entries = Vec::new();
            for room in &rooms {
                let room_id = *room.id();
                let created_at = mgr.created_at(&room_id).unwrap_or(0);
                let members_ordered = room.members_in_join_order();

                let capacity = room.config().capacity;
                let team_size = capacity / 2;

                let first_member = members_ordered.first().and_then(|eid| room.member(eid));
                let proposer_json = if let Some(member) = first_member {
                    let eid = members_ordered[0];
                    serde_json::json!({
                        "id": eid.to_string(),
                        "persistedId": member.data.character_id.to_string(),
                        "name": member.data.name,
                        "level": member.data.level,
                        "className": member.data.class_name,
                        "raceName": member.data.race_name,
                    })
                } else {
                    serde_json::json!({})
                };

                let participants: Vec<serde_json::Value> = members_ordered.iter().filter_map(|eid| {
                    let m = room.member(eid)?;
                    Some(serde_json::json!({
                        "id": eid.to_string(),
                        "persistedId": m.data.character_id.to_string(),
                        "name": m.data.name,
                        "level": m.data.level,
                        "className": m.data.class_name,
                        "raceName": m.data.race_name,
                    }))
                }).collect();

                entries.push(serde_json::json!({
                    "id": room_id.to_string(),
                    "createdAt": created_at,
                    "teamSize": team_size as i32,
                    "proposer": proposer_json,
                    "participants": participants,
                }));
            }

            let state = serde_json::json!({ "challenges": entries });
            serde_json::to_string(&state).unwrap_or_default()
        };

        let mut w = PacketWriter::with_packet_id(client_packet_id::OPEN_RETOS);
        w.write_string(&json);
        self.send_to_client(sink, w.into_bytes()).await?;

        Ok(())
    }
}
