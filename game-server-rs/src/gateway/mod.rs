mod bank;
mod challenges;
mod clan;
mod combat;
mod commerce;
mod connect;
mod crafting;
mod dialog;
pub mod fishing;
mod harvesting;
pub(crate) mod inventory;
mod market;
mod movement;
pub mod packets;
mod party;
mod smelting;
mod trade;
mod quests;
mod pets;
mod territory;
mod achievements;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::time::{interval, Instant};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tracing::{debug, info, warn, Instrument};

const AUTH_DEADLINE: Duration = Duration::from_secs(15);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(90);

use openao_protocol::opcodes::{client_packet_id, server_packet_id};
use openao_protocol::{PacketReader, PacketWriter};

use crate::elr2::{self, Frame, ROUTE_AUTHENTICATE, ROUTE_HEARTBEAT, ROUTE_GAME};
use crate::error::HandlerResult;
use crate::rate_limit::{CommandRateLimiter, RateLimiter};
use crate::reconnect::ReconnectManager;
use crate::routes::PacketRouter;
use crate::ServerMetrics;
use crate::world::{BroadcastPacket, EntityId, GameWorld};

pub type WsSink = SplitSink<WebSocketStream<TcpStream>, Message>;
type WsStream = futures_util::stream::SplitStream<WebSocketStream<TcpStream>>;

fn wrap_elr2_push(game_payload: Vec<u8>) -> Bytes {
    Frame::push(ROUTE_GAME, Bytes::from(game_payload)).encode()
}

pub struct GameSession {
    session_id: String,
    addr: SocketAddr,
    world: Arc<GameWorld>,
    entity_id: Option<EntityId>,
    map_id: Option<i32>,
    character_name: Option<String>,
    character_id: Option<String>,
    broadcast_rx_needs_refresh: bool,
    personal_tx: Option<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,
    authenticated: bool,
    auth_account_id: Option<String>,
    auth_character_id: Option<String>,
    use_elr2: bool,
    trade_npc_type: Option<i32>,
    market_npc_name: Option<String>,
    rate_limiter: RateLimiter,
    command_limiter: CommandRateLimiter,
    reconnect_mgr: Arc<ReconnectManager>,
    router: Arc<PacketRouter>,
    metrics: Arc<ServerMetrics>,
    /// Set to true by a newer session for the same character, signaling this session to close.
    pub(crate) evicted: Arc<std::sync::atomic::AtomicBool>,
}

impl GameSession {
    pub fn new(
        addr: SocketAddr,
        world: Arc<GameWorld>,
        use_elr2: bool,
        reconnect_mgr: Arc<ReconnectManager>,
        metrics: Arc<ServerMetrics>,
        router: Arc<PacketRouter>,
    ) -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
            addr,
            world,
            entity_id: None,
            map_id: None,
            character_name: None,
            character_id: None,
            broadcast_rx_needs_refresh: false,
            personal_tx: None,
            authenticated: false,
            auth_account_id: None,
            auth_character_id: None,
            use_elr2,
            trade_npc_type: None,
            market_npc_name: None,
            rate_limiter: RateLimiter::new(60),
            command_limiter: CommandRateLimiter::new(500),
            reconnect_mgr,
            router,
            metrics,
            evicted: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub async fn run(
        self,
        sink: WsSink,
        stream: WsStream,
    ) -> HandlerResult {
        let span = tracing::info_span!("session", sid = %self.session_id, addr = %self.addr);
        self.run_inner(sink, stream).instrument(span).await
    }

    async fn run_inner(
        mut self,
        mut sink: WsSink,
        mut stream: WsStream,
    ) -> HandlerResult {
        // Dummy broadcast channel — never sends, used before player joins a scene
        let (dummy_tx, _) = tokio::sync::broadcast::channel::<BroadcastPacket>(1);
        let mut broadcast_rx = dummy_tx.subscribe();
        let mut broadcast_active = false;

        let (personal_tx, mut personal_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        self.personal_tx = Some(personal_tx);
        let personal_queue_warn_threshold: usize = 512;

        let mut last_activity = Instant::now();
        let mut hb_interval = interval(HEARTBEAT_INTERVAL);
        hb_interval.tick().await;
        let mut hb_sequence: u32 = 0;

        let auth_deadline = if self.use_elr2 {
            Some(Instant::now() + AUTH_DEADLINE)
        } else {
            None
        };

        loop {
            if self.broadcast_rx_needs_refresh {
                self.broadcast_rx_needs_refresh = false;
                if let Some(mid) = self.map_id {
                    let scene = self.world.get_or_create_scene(mid);
                    broadcast_rx = scene.broadcast_tx.subscribe();
                    broadcast_active = true;
                }
            }

            if !broadcast_active
                && self.entity_id.is_some()
                && let Some(mid) = self.map_id
            {
                let scene = self.world.get_or_create_scene(mid);
                broadcast_rx = scene.broadcast_tx.subscribe();
                broadcast_active = true;
            }

            if let Some(deadline) = auth_deadline
                && !self.authenticated && self.entity_id.is_none() && Instant::now() > deadline
            {
                warn!("Auth deadline exceeded for {}, disconnecting", self.addr);
                break;
            }

            if last_activity.elapsed() > CLIENT_TIMEOUT {
                warn!("Client {} timed out (no activity for {:?})", self.addr, CLIENT_TIMEOUT);
                break;
            }

            if self.evicted.load(std::sync::atomic::Ordering::Relaxed) {
                warn!("Session {} evicted by newer connection, closing", self.addr);
                break;
            }

            tokio::select! {
                msg = stream.next() => {
                    match msg {
                        Some(Ok(Message::Binary(data))) => {
                            last_activity = Instant::now();
                            self.handle_message(&data, &mut sink).await?;
                            if let Some(eid) = self.entity_id {
                                let _ = self.tick_fishing(eid, &mut sink).await;
                                let _ = self.tick_harvesting(eid, &mut sink).await;
                                let _ = self.tick_revive(&mut sink).await;
                                if self.check_pending_logout(eid) {
                                    break;
                                }
                            }
                            if self.entity_id.is_some() && self.broadcast_rx_needs_refresh {
                                self.broadcast_rx_needs_refresh = false;
                                if let Some(mid) = self.map_id {
                                    let scene = self.world.get_or_create_scene(mid);
                                    broadcast_rx = scene.broadcast_tx.subscribe();
                                    broadcast_active = true;
                                }
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            last_activity = Instant::now();
                            match tokio::time::timeout(std::time::Duration::from_secs(5), sink.send(Message::Pong(data))).await {
                                Ok(result) => result?,
                                Err(_) => { warn!("Pong send timed out for {}, disconnecting", self.addr); break; }
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            last_activity = Instant::now();
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        Some(Err(e)) => {
                            warn!("WebSocket error from {}: {e}", self.addr);
                            break;
                        }
                        _ => {}
                    }
                }
                bcast = broadcast_rx.recv() => {
                    if let Ok(pkt) = bcast
                        && let Some(eid) = self.entity_id
                    {
                        let send_bcast = async {
                            let should_send = pkt.sender_entity_id == 0
                                || pkt.sender_entity_id == u32::MAX
                                || pkt.sender_entity_id != eid;
                            if should_send {
                                self.metrics.total_packets_out.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                if self.use_elr2 {
                                    let frame_bytes = wrap_elr2_push(pkt.data);
                                    sink.feed(Message::Binary(frame_bytes)).await?;
                                } else {
                                    sink.feed(Message::Binary(pkt.data.into())).await?;
                                }
                            }
                            while let Ok(extra_pkt) = broadcast_rx.try_recv() {
                                if let Some(eid2) = self.entity_id {
                                    let send2 = extra_pkt.sender_entity_id == 0
                                        || extra_pkt.sender_entity_id == u32::MAX
                                        || extra_pkt.sender_entity_id != eid2;
                                    if send2 {
                                        self.metrics.total_packets_out.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                        if self.use_elr2 {
                                            let fb = wrap_elr2_push(extra_pkt.data);
                                            sink.feed(Message::Binary(fb)).await?;
                                        } else {
                                            sink.feed(Message::Binary(extra_pkt.data.into())).await?;
                                        }
                                    }
                                }
                            }
                            sink.flush().await?;
                            Ok::<(), tokio_tungstenite::tungstenite::Error>(())
                        };
                        if tokio::time::timeout(std::time::Duration::from_secs(5), send_bcast).await.is_err() {
                            warn!("Broadcast send timed out for {}, disconnecting", self.addr);
                            break;
                        }
                    }
                }
                personal = personal_rx.recv() => {
                    if let Some(data) = personal {
                        let send_personal = async {
                            self.metrics.total_packets_out.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if self.use_elr2 {
                                let frame_bytes = wrap_elr2_push(data);
                                sink.feed(Message::Binary(frame_bytes)).await?;
                            } else {
                                sink.feed(Message::Binary(data.into())).await?;
                            }
                            let mut drained: usize = 0;
                            while let Ok(extra) = personal_rx.try_recv() {
                                self.metrics.total_packets_out.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                drained += 1;
                                if self.use_elr2 {
                                    let frame_bytes = wrap_elr2_push(extra);
                                    sink.feed(Message::Binary(frame_bytes)).await?;
                                } else {
                                    sink.feed(Message::Binary(extra.into())).await?;
                                }
                            }
                            if drained >= personal_queue_warn_threshold {
                                warn!("[{}] Personal queue had {} pending packets — possible backpressure", self.addr, drained);
                            }
                            sink.flush().await?;
                            Ok::<(), tokio_tungstenite::tungstenite::Error>(())
                        };
                        if tokio::time::timeout(std::time::Duration::from_secs(5), send_personal).await.is_err() {
                            warn!("Personal send timed out for {}, disconnecting", self.addr);
                            break;
                        }
                    }
                }
                _ = hb_interval.tick() => {
                    let hb_send = async {
                        if self.use_elr2 {
                            hb_sequence += 1;
                            let hb = Frame {
                                kind: elr2::FrameKind::Request,
                                flags: 0,
                                route: ROUTE_HEARTBEAT,
                                request_id: 0,
                                sequence: hb_sequence,
                                payload: Bytes::new(),
                            };
                            sink.send(Message::Binary(hb.encode())).await
                        } else {
                            sink.send(Message::Ping(Bytes::from_static(b"hb"))).await
                        }
                    };
                    if tokio::time::timeout(std::time::Duration::from_secs(5), hb_send).await.is_err() {
                        warn!("Heartbeat send timed out for {}, disconnecting", self.addr);
                        break;
                    }
                }
            }
        }

        self.handle_disconnect().await;
        debug!("Connection closed for {}", self.addr);
        Ok(())
    }

    async fn handle_disconnect(&self) {
        if let Some(eid) = self.entity_id
            && let Some(mid) = self.map_id
        {
            let scene = self.world.get_or_create_scene(mid);
            let player_name = self.character_name.as_deref().unwrap_or("?");

            if let Some(player) = scene.players.get(&eid)
                && let Some(ref char_id) = self.character_id
            {
                let bank_gold = self.world.db.get_bank_gold(char_id).await.unwrap_or(0);
                let _ = self.world.db.save_character_state(
                    char_id,
                    player.pos.map, player.pos.x, player.pos.y,
                    player.hp, player.max_hp, player.mana, player.max_mana,
                    player.gold, player.level, player.exp, player.exp_next_level,
                    player.dead, &player.faction, player.criminal,
                    player.min_hit, player.max_hit,
                    player.attr_fuerza, player.attr_agilidad,
                    player.attr_inteligencia, player.attr_constitucion,
                    player.home_map, player.home_x, player.home_y,
                    player.id_head, player.id_body, player.id_helmet,
                    player.id_weapon, player.id_shield,
                    player.id_arrow_slot, player.id_ring_slot,
                    player.navegando, bank_gold,
                    player.id_clase, player.faction_rank, player.faction_score,
                    player.faction_score_armada, player.faction_score_caos,
                    player.criminales_matados, player.ciudadanos_matados,
                ).await;
                self.world.cache_flush_inventory(char_id).await;
                self.world.cache_remove(char_id);
                let _ = self.world.db.save_quest_log(char_id, &player.quest_log).await;
                let _ = self.world.db.save_pets(char_id, &player.pets).await;
                let _ = self.world.db.save_achievements(char_id, &player.achievements).await;

                info!("Saved state for '{}' at map={}, pos=({},{})",
                    player_name, player.pos.map, player.pos.x, player.pos.y);

                let reconnect_token = self.reconnect_mgr.issue_token(
                    crate::reconnect::ReconnectState {
                        account_id: self.auth_account_id.clone().unwrap_or_default(),
                        character_id: char_id.clone(),
                        character_name: player_name.to_string(),
                        entity_id: eid,
                        map_id: mid,
                    },
                );
                debug!("Issued reconnect token for '{}': {}", player_name, reconnect_token);
            }

            let delete_pkt = crate::replication::build_delete_character_packet(eid);
            let disconnect_pos = scene.players.get(&eid).map(|p| p.pos.clone());
            if let Some(ref pos) = disconnect_pos {
                scene.broadcast_in_range(eid, pos, delete_pkt);
            } else {
                scene.broadcast(eid, delete_pkt);
            }

            scene.aoi_remove(eid);
            scene.players.remove(&eid);
            scene.personal_tx.remove(&eid);
            scene.outbound_pressure.remove(&eid);
            scene.remove_replicator(eid);
            scene.remove_input_receiver(eid);

            if let Some(ref cid) = self.character_id {
                self.world.active_characters.remove(cid);
            }

            self.cleanup_summons_on_disconnect(eid, &scene);
            self.cleanup_party_on_disconnect(eid);
            self.cleanup_trade(eid);

            tracing::info!(
                target: "activity",
                category = "session", action = "character_disconnect",
                player = player_name, entity = eid, map = mid,
                "CHARACTER_DISCONNECT"
            );
            info!(
                "Player '{}' (entity {}) disconnected from map {}",
                player_name, eid, mid
            );
        }
    }

    fn cleanup_summons_on_disconnect(&self, eid: EntityId, scene: &crate::world::Scene) {
        let summon_ids: Vec<EntityId> = scene.npcs.iter()
            .filter(|e| e.summoned_by == Some(eid))
            .map(|e| *e.key())
            .collect();
        for sid in summon_ids {
            if let Some((_, npc)) = scene.npcs.remove(&sid) {
                scene.aoi_remove(sid);
                let del = crate::replication::build_delete_character_packet(sid);
                scene.broadcast_in_range(0, &npc.pos, del);
            }
        }
    }

    fn cleanup_party_on_disconnect(&self, eid: EntityId) {
        let party_id_to_check: Option<String> = {
            let mut found = None;
            for party_ref in self.world.parties.iter() {
                if party_ref.member_ids.contains(&eid) {
                    found = Some(party_ref.key().clone());
                    break;
                }
            }
            found
        };

        if let Some(party_id) = party_id_to_check {
            let should_disband = {
                if let Some(mut party) = self.world.parties.get_mut(&party_id) {
                    party.member_ids.retain(|&id| id != eid);
                    if party.member_ids.len() <= 1 {
                        true
                    } else if party.leader_id == eid {
                        party.leader_id = party.member_ids[0];
                        false
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if should_disband {
                if let Some((_, party)) = self.world.parties.remove(&party_id) {
                    for &member_id in &party.member_ids {
                        self.clear_player_party(member_id);
                        self.send_empty_party_state(member_id);
                    }
                }
            } else if let Some(party) = self.world.parties.get(&party_id) {
                let member_ids = party.member_ids.clone();
                let leader = party.leader_id;
                drop(party);
                self.sync_party_state(&party_id, &member_ids, leader);
            }
        }
    }

    fn clear_player_party(&self, entity_id: EntityId) {
        for scene_ref in self.world.scenes.iter() {
            if let Some(mut player) = scene_ref.players.get_mut(&entity_id) {
                player.party_id = None;
                return;
            }
        }
    }

    fn build_party_state_packet(&self, member_ids: &[EntityId], leader_id: EntityId) -> Vec<u8> {
        let mut upsert_items = Vec::new();
        for &mid in member_ids {
            for scene_ref in self.world.scenes.iter() {
                if let Some(p) = scene_ref.players.get(&mid) {
                    upsert_items.push(serde_json::json!({
                        "id": mid,
                        "nameCharacter": p.name,
                        "map": p.pos.map,
                        "pos": { "x": p.pos.x, "y": p.pos.y },
                        "online": true,
                        "isLeader": mid == leader_id,
                    }));
                    break;
                }
            }
        }

        let delta = serde_json::json!({
            "upsert": upsert_items,
            "remove": [],
        });

        packets::build_party_state(&delta.to_string())
    }

    #[allow(dead_code)]
    fn send_party_state_to_player(&self, entity_id: EntityId, members: &[EntityId], leader_id: EntityId) {
        let data = self.build_party_state_packet(members, leader_id);
        for scene_ref in self.world.scenes.iter() {
            if let Some(tx) = scene_ref.personal_tx.get(&entity_id) {
                let _ = tx.send(data);
                return;
            }
        }
    }

    fn send_empty_party_state(&self, entity_id: EntityId) {
        let delta = serde_json::json!({ "upsert": [], "remove": [] });
        let data = packets::build_party_state(&delta.to_string());

        for scene_ref in self.world.scenes.iter() {
            if let Some(tx) = scene_ref.personal_tx.get(&entity_id) {
                let _ = tx.send(data);
                return;
            }
        }
    }

    fn sync_party_state(&self, _party_id: &str, member_ids: &[EntityId], leader_id: EntityId) {
        let data = self.build_party_state_packet(member_ids, leader_id);
        for &mid in member_ids {
            for scene_ref in self.world.scenes.iter() {
                if let Some(tx) = scene_ref.personal_tx.get(&mid) {
                    let _ = tx.send(data.clone());
                    break;
                }
            }
        }
    }

    const MAX_PACKET_SIZE: usize = 8192;

    /// Top-level message handler: detects ELR2 framing vs legacy protocol.
    /// Applies rate limiting and size validation before processing.
    async fn handle_message(
        &mut self,
        data: &[u8],
        sink: &mut WsSink,
    ) -> HandlerResult {
        if data.is_empty() {
            return Ok(());
        }

        if data.len() > Self::MAX_PACKET_SIZE {
            debug!("Oversized packet ({} bytes) from {}", data.len(), self.addr);
            self.metrics.packets_rejected.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Ok(());
        }

        if !self.rate_limiter.check() {
            debug!("Rate limited packet from {}", self.addr);
            self.metrics.packets_rejected.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Ok(());
        }

        self.metrics.total_packets_in.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let is_elr2 = data.len() >= elr2::ELR2_HEADER_LEN
            && u32::from_be_bytes([data[0], data[1], data[2], data[3]]) == elr2::ELR2_MAGIC;

        if is_elr2 {
            self.use_elr2 = true;
            self.handle_elr2_frame(data, sink).await
        } else {
            self.handle_legacy_binary(data, sink).await
        }
    }

    /// Handle an ELR2-framed message
    async fn handle_elr2_frame(
        &mut self,
        data: &[u8],
        sink: &mut WsSink,
    ) -> HandlerResult {
        let received_at = std::time::Instant::now();
        let frame = match Frame::decode(Bytes::copy_from_slice(data)) {
            Ok(f) => f,
            Err(e) => {
                warn!("Invalid ELR2 frame from {}: {e}", self.addr);
                return Ok(());
            }
        };

        match frame.route {
            ROUTE_AUTHENTICATE => {
                self.handle_elr2_auth(&frame, sink).await?;
            }
            ROUTE_HEARTBEAT => {
                let server_tick = self.metrics.current_tick.load(std::sync::atomic::Ordering::Relaxed);
                let server_received_ms = received_at.duration_since(self.metrics.uptime_start).as_millis() as u64;
                let server_sent_ms = std::time::Instant::now().duration_since(self.metrics.uptime_start).as_millis() as u64;
                let payload = serde_json::json!({
                    "server_tick": server_tick,
                    "server_received_at": server_received_ms,
                    "server_sent_at": server_sent_ms,
                });
                let response = Frame::response(
                    &frame,
                    Bytes::from(serde_json::to_vec(&payload).unwrap_or_default()),
                );
                match tokio::time::timeout(std::time::Duration::from_secs(5), sink.send(Message::Binary(response.encode()))).await {
                    Ok(result) => result?,
                    Err(_) => return Err(Box::<dyn std::error::Error + Send + Sync>::from("heartbeat response timeout").into()),
                }
            }
            ROUTE_GAME => {
                if !self.authenticated {
                    let err = Frame::error_response(&frame, Bytes::from_static(b"not authenticated"));
                    match tokio::time::timeout(std::time::Duration::from_secs(5), sink.send(Message::Binary(err.encode()))).await {
                        Ok(result) => result?,
                        Err(_) => return Err(Box::<dyn std::error::Error + Send + Sync>::from("send timeout").into()),
                    }
                    return Ok(());
                }
                self.handle_game_payload(&frame.payload, sink).await?;
            }
            _ => {
                debug!("Unknown ELR2 route {} from {}", frame.route, self.addr);
                let err = Frame::error_response(&frame, Bytes::from_static(b"unknown route"));
                sink.send(Message::Binary(err.encode())).await?;
            }
        }
        Ok(())
    }

    /// Send a message with a 5-second timeout to prevent blocking.
    async fn timed_sink_send(sink: &mut WsSink, msg: Message) -> HandlerResult {
        match tokio::time::timeout(std::time::Duration::from_secs(5), sink.send(msg)).await {
            Ok(result) => result?,
            Err(_) => return Err(Box::<dyn std::error::Error + Send + Sync>::from("WebSocket send timeout").into()),
        }
        Ok(())
    }

    /// ELR2 authentication: payload is JSON `{"ticket": "..."}` for fresh login
    /// or `{"reconnect_token": "..."}` for session resume.
    async fn handle_elr2_auth(
        &mut self,
        frame: &Frame,
        sink: &mut WsSink,
    ) -> HandlerResult {
        #[derive(serde::Deserialize)]
        struct AuthPayload {
            ticket: Option<String>,
            reconnect_token: Option<String>,
        }

        let auth: AuthPayload = match serde_json::from_slice(&frame.payload) {
            Ok(a) => a,
            Err(_) => {
                let err = Frame::error_response(frame, Bytes::from_static(b"invalid auth payload"));
                Self::timed_sink_send(sink, Message::Binary(err.encode())).await?;
                return Ok(());
            }
        };

        if let Some(token) = auth.reconnect_token {
            if let Some(state) = self.reconnect_mgr.consume_token(&token) {
                info!("ELR2 reconnect success for {} (char={})", self.addr, state.character_name);
                self.authenticated = true;
                self.auth_account_id = Some(state.account_id.clone());
                self.auth_character_id = Some(state.character_id.clone());

                let new_token = self.reconnect_mgr.issue_token(
                    crate::reconnect::ReconnectState {
                        account_id: state.account_id.clone(),
                        character_id: state.character_id.clone(),
                        character_name: state.character_name.clone(),
                        entity_id: state.entity_id,
                        map_id: state.map_id,
                    },
                );

                let resp_json = serde_json::json!({
                    "status": "ok",
                    "reconnected": true,
                    "account_id": state.account_id,
                    "character_id": state.character_id,
                    "reconnect_token": new_token,
                });
                let resp_bytes = serde_json::to_vec(&resp_json).unwrap_or_default();
                let response = Frame::response(frame, Bytes::from(resp_bytes));
                Self::timed_sink_send(sink, Message::Binary(response.encode())).await?;
            } else {
                let err = Frame::error_response(frame, Bytes::from_static(b"invalid or expired reconnect token"));
                Self::timed_sink_send(sink, Message::Binary(err.encode())).await?;
            }
            return Ok(());
        }

        let ticket = match auth.ticket {
            Some(t) => t,
            None => {
                let err = Frame::error_response(frame, Bytes::from_static(b"ticket or reconnect_token required"));
                Self::timed_sink_send(sink, Message::Binary(err.encode())).await?;
                return Ok(());
            }
        };

        let ticket_data = self.world.db.consume_game_ticket(&ticket).await;
        match ticket_data {
            Ok(Some((account_id, character_id))) => {
                info!("ELR2 auth success for {} (account={}, char={})", self.addr, account_id, character_id);
                self.authenticated = true;
                self.auth_account_id = Some(account_id.clone());
                self.auth_character_id = Some(character_id.clone());

                let resp_json = serde_json::json!({
                    "status": "ok",
                    "account_id": account_id,
                    "character_id": character_id,
                });
                let resp_bytes = serde_json::to_vec(&resp_json).unwrap_or_default();
                let response = Frame::response(frame, Bytes::from(resp_bytes));
                Self::timed_sink_send(sink, Message::Binary(response.encode())).await?;
            }
            Ok(None) => {
                let err = Frame::error_response(frame, Bytes::from_static(b"invalid or expired ticket"));
                Self::timed_sink_send(sink, Message::Binary(err.encode())).await?;
            }
            Err(e) => {
                warn!("Ticket validation error: {e}");
                let err = Frame::error_response(frame, Bytes::from_static(b"internal error"));
                Self::timed_sink_send(sink, Message::Binary(err.encode())).await?;
            }
        }
        Ok(())
    }

    /// Process game payload (same as legacy binary, but payload doesn't include ELR2 header)
    async fn handle_game_payload(
        &mut self,
        payload: &[u8],
        sink: &mut WsSink,
    ) -> HandlerResult {
        self.handle_legacy_binary(payload, sink).await
    }

    /// Send a game packet to the client, wrapping in ELR2 if needed.
    /// Tracks outgoing packet count in server metrics.
    pub(crate) async fn send_to_client(
        &self,
        sink: &mut WsSink,
        data: Vec<u8>,
    ) -> HandlerResult {
        self.metrics.total_packets_out.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let msg = if self.use_elr2 {
            Message::Binary(wrap_elr2_push(data))
        } else {
            Message::Binary(data.into())
        };
        match tokio::time::timeout(std::time::Duration::from_secs(5), sink.send(msg)).await {
            Ok(result) => result?,
            Err(_) => {
                tracing::warn!("send_to_client timed out for {}", self.addr);
                return Err(Box::<dyn std::error::Error + Send + Sync>::from("WebSocket send timeout").into());
            }
        }
        Ok(())
    }

    /// Send multiple packets in a single flush — reduces syscalls and TCP segments.
    /// Uses `feed()` for each packet, then a single `flush()` at the end.
    pub(crate) async fn send_batch_to_client(
        &self,
        sink: &mut WsSink,
        packets: Vec<Vec<u8>>,
    ) -> HandlerResult {
        if packets.is_empty() {
            return Ok(());
        }
        let count = packets.len();
        self.metrics.total_packets_out.fetch_add(count as u64, std::sync::atomic::Ordering::Relaxed);

        let send_all = async {
            if self.use_elr2 {
                for pkt in packets {
                    let frame_bytes = wrap_elr2_push(pkt);
                    sink.feed(Message::Binary(frame_bytes)).await?;
                }
            } else {
                for pkt in packets {
                    sink.feed(Message::Binary(Bytes::from(pkt))).await?;
                }
            }
            sink.flush().await?;
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        };

        match tokio::time::timeout(std::time::Duration::from_secs(10), send_all).await {
            Ok(result) => result?,
            Err(_) => {
                tracing::warn!("send_batch_to_client timed out ({} packets) for {}", count, self.addr);
                return Err(Box::<dyn std::error::Error + Send + Sync>::from("WebSocket batch send timeout").into());
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) async fn send_console_message(
        &self,
        msg: &str,
        sink: &mut WsSink,
    ) -> HandlerResult {
        self.send_to_client(sink, packets::build_console_message(msg)).await
    }

    /// Legacy binary protocol handler (packet ID as first byte).
    /// Uses PacketRouter for route-aware tracing.
    async fn handle_legacy_binary(
        &mut self,
        data: &[u8],
        sink: &mut WsSink,
    ) -> HandlerResult {
        if data.is_empty() {
            return Ok(());
        }

        let mut reader = PacketReader::new(data);
        let packet_id = reader.get_byte()?;

        let route_info = self.router.get(packet_id);
        if let Some(route) = route_info {
            debug!("[{}] {} → {:?}", self.addr, route.name, route.category);
            self.metrics.packets_by_category.increment(route.category);

            if route.requires_character && self.entity_id.is_none() {
                debug!("[{}] Dropped {} — no character connected", self.addr, route.name);
                self.metrics.packets_dropped_no_char.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return Ok(());
            }
        }

        match packet_id {
            server_packet_id::CONNECT_CHARACTER => {
                self.handle_connect_character(&mut reader, sink).await?;
            }
            server_packet_id::PING => {
                let token = reader.get_int()?;
                let mut w = PacketWriter::with_packet_id(client_packet_id::PONG);
                w.write_int(token);
                self.send_to_client(sink, w.into_bytes()).await?;
            }
            server_packet_id::POSITION => {
                let heading = reader.get_byte()?;
                let move_id = reader.get_short()?;

                let redundant_count = if reader.remaining() >= 1 { reader.get_byte()? } else { 0 };
                if redundant_count > 0
                    && let (Some(eid), Some(mid)) = (self.entity_id, self.map_id) {
                        let scene = self.world.get_or_create_scene(mid);
                        if let Some(receiver_entry) = scene.input_receivers.get(&eid)
                            && let Ok(mut receiver) = receiver_entry.try_lock() {
                                let mut frames = Vec::new();
                                for _ in 0..redundant_count {
                                    if reader.remaining() < 3 { break; }
                                    let seq = reader.get_short()? as u64;
                                    let h = reader.get_byte()?;
                                    frames.push(elura::gameplay::netcode::InputFrame {
                                        sequence: seq,
                                        target_tick: receiver.current_tick(),
                                        input: crate::gameplay::input_queue::GameInput::Move { heading: h },
                                    });
                                }
                                if !frames.is_empty() {
                                    let packet = elura::gameplay::netcode::InputPacket {
                                        client_tick: move_id as u64,
                                        acknowledged_server_tick: 0,
                                        inputs: frames,
                                    };
                                    let _ = receiver.receive(packet);
                                }
                            }
                        drop(scene);
                    }

                self.handle_movement(heading, move_id, sink).await?;
            }
            server_packet_id::CHANGE_HEADING => {
                let heading = reader.get_byte()?;
                self.handle_change_heading(heading, sink).await?;
            }
            server_packet_id::DIALOG => {
                let message = reader.get_string()?;
                self.handle_dialog(&message, sink).await?;
            }
            server_packet_id::ATTACK_MELE => {
                self.handle_attack_melee(sink).await?;
            }
            server_packet_id::ATTACK_RANGE => {
                self.handle_attack_range(sink).await?;
            }
            server_packet_id::ATTACK_SPELL => {
                let spell_slot = reader.get_byte()?;
                self.handle_attack_spell(spell_slot, sink).await?;
            }
            server_packet_id::CLICK => {
                let x = reader.get_short()? as i32;
                let y = reader.get_short()? as i32;
                self.handle_click(x, y, sink).await?;
            }
            server_packet_id::USE_ITEM_CLICK => {
                let slot = reader.get_byte()?;
                self.handle_use_item(slot, sink).await?;
            }
            server_packet_id::EQUIPAR_ITEM => {
                let slot = reader.get_byte()?;
                self.handle_equip_item(slot, sink).await?;
            }
            server_packet_id::TIRAR_ITEM => {
                let slot = reader.get_byte()?;
                let qty = reader.get_short()?;
                self.handle_drop_item(slot, qty, sink).await?;
            }
            server_packet_id::AGARRAR_ITEM => {
                self.handle_agarrar_item(sink).await?;
            }
            server_packet_id::CHANGE_SEGURO => {
                self.handle_toggle_safe(sink).await?;
            }
            server_packet_id::RESYNC_POSITION => {
                self.handle_resync_position(sink).await?;
            }
            server_packet_id::CRAFT_ITEM => {
                let recipe_id = reader.get_short()? as i32;
                self.handle_craft_item(recipe_id, sink).await?;
            }
            server_packet_id::BUY_ITEM => {
                let npc_slot = reader.get_byte()? as i32;
                let amount = reader.get_short()? as i32;
                self.handle_buy_item(npc_slot, amount, sink).await?;
            }
            server_packet_id::SELL_ITEM => {
                let inv_slot = reader.get_byte()? as i32;
                let amount = reader.get_short()? as i32;
                self.handle_sell_item(inv_slot, amount, sink).await?;
            }
            server_packet_id::CLOSE_TRADE => {
                self.trade_npc_type = None;
            }
            server_packet_id::USE_ITEM_U => {
                let slot = reader.get_byte()?;
                self.handle_use_item(slot, sink).await?;
            }
            server_packet_id::REORDER_SPELL => {
                let source = reader.get_byte()?;
                let target = reader.get_byte()?;
                self.handle_reorder_spell(source, target, sink).await?;
            }
            server_packet_id::REORDER_INVENTORY_ITEM => {
                let source = reader.get_byte()?;
                let target = reader.get_byte()?;
                self.handle_reorder_inventory(source, target, sink).await?;
            }
            server_packet_id::TOGGLE_HIDDEN_SKILL => {
                self.handle_toggle_hidden(sink).await?;
            }
            server_packet_id::CHANGE_CLAN_SEGURO => {
                self.handle_toggle_clan_safe(sink).await?;
            }
            server_packet_id::CHANGE_BANK_TAB => {
                let _tab = reader.get_byte()?;
            }
            server_packet_id::DEPOSIT_BANK_GOLD => {
                let amount = reader.get_int()?;
                self.handle_deposit_bank_gold(amount as i32, sink).await?;
            }
            server_packet_id::WITHDRAW_BANK_GOLD => {
                let amount = reader.get_int()?;
                self.handle_withdraw_bank_gold(amount as i32, sink).await?;
            }
            server_packet_id::REORDER_BANK_ITEM => {
                let source = reader.get_byte()?;
                let target = reader.get_byte()?;
                self.handle_reorder_bank(source, target, sink).await?;
            }
            server_packet_id::MARKET_ACTION => {
                let payload = reader.get_string()?;
                self.handle_market_action(&payload, sink).await?;
            }
            server_packet_id::RETOS_ACTION => {
                let payload = reader.get_string()?;
                self.handle_retos_action(&payload, sink).await?;
            }
            _ => {
                debug!("Unhandled packet {} from {}", packet_id, self.addr);
            }
        }

        Ok(())
    }
}
