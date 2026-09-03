use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use crate::persistence::Database;
use crate::reconnect::ReconnectManager;
use crate::world::GameWorld;
use crate::ServerMetrics;

fn build_cors_layer() -> CorsLayer {
    match std::env::var("CORS_ALLOWED_ORIGINS") {
        Ok(origins) if !origins.is_empty() => {
            let allowed: Vec<_> = origins
                .split(',')
                .filter_map(|o| o.trim().parse().ok())
                .collect();
            CorsLayer::new()
                .allow_origin(allowed)
                .allow_methods(Any)
                .allow_headers(Any)
        }
        _ => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    }
}

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<Database>,
    pub world: Arc<GameWorld>,
    pub metrics: Arc<ServerMetrics>,
    pub reconnect_mgr: Arc<ReconnectManager>,
}

pub fn create_router(db: Arc<Database>, world: Arc<GameWorld>, metrics: Arc<ServerMetrics>, reconnect_mgr: Arc<ReconnectManager>) -> Router {
    let state = ApiState { db, world, metrics, reconnect_mgr };

    let cors = build_cors_layer();

    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/register", post(register))
        .route("/api/auth/me", get(me))
        .route("/api/auth/request-password-reset", post(request_password_reset))
        .route("/api/auth/signout", post(signout))
        .route("/api/auth/create-character", post(create_character))
        .route("/api/ranking", get(ranking))
        .route("/api/arenas", get(arenas))
        .route("/api/arenas/rooms", get(arenas_rooms))
        .route("/api/arenas/rooms", post(arenas_create_room))
        .route("/api/arenas/rooms/{room_id}/join", post(arenas_join_room))
        .route("/api/arenas/rooms/{room_id}/leave", post(arenas_leave_room))
        .route("/api/arenas/rooms/{room_id}", axum::routing::delete(arenas_cancel_room))
        .route("/api/clans", get(clans_list))
        .route("/api/clans/{clan_id}", get(clan_detail))
        .route("/api/users-online-stats", get(users_online_stats))
        .route("/api/runtime-config", get(get_runtime_config))
        .route("/api/runtime-config", post(set_runtime_config))
        .route("/api/character-settings/{char_id}", get(get_character_settings))
        .route("/api/character-settings/{char_id}", post(save_character_settings))
        .route("/api/world-builder/maps/{map_id}/tiles", post(world_builder_paint_tiles))
        .route("/api/world-builder/maps/{map_id}/entities", post(world_builder_place_entity))
        .route("/api/world-builder/maps/{map_id}/entities/{entity_type}/{entity_id}", axum::routing::delete(world_builder_remove_entity))
        .route("/api/world-builder/maps/{map_id}/blocked", post(world_builder_set_blocked))
        .route("/api/world-builder/maps/{map_id}/teleports", post(world_builder_set_teleport))
        .route("/api/wiki", get(wiki))
        .route("/api/characters/by-account/{account_id}", get(list_characters))
        .route("/api/characters/{char_id}", axum::routing::delete(delete_character_endpoint))
        .route("/api/admin/ban", post(admin_ban))
        .route("/api/admin/unban", post(admin_unban))
        .route("/api/admin/mute", post(admin_mute))
        .route("/api/admin/unmute", post(admin_unmute))
        .route("/api/admin/ip-ban", post(admin_ip_ban))
        .route("/api/admin/ip-unban", post(admin_ip_unban))
        .route("/api/admin/game-data/objects", get(admin_list_objects))
        .route("/api/admin/game-data/npcs", get(admin_list_npcs))
        .route("/api/admin/game-data/spells", get(admin_list_spells))
        .route("/api/health", get(health))
        .route("/api/readiness", get(readiness))
        .route("/api/metrics", get(server_metrics))
        .route("/api/metrics/prometheus", get(prometheus_metrics))
        .layer(cors)
        .with_state(state)
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    ticket: String,
    account_id: String,
    name: String,
}

async fn login(
    State(state): State<ApiState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let account = state.db.find_account_by_email(&body.email).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;

    let account = match account {
        Some(a) => a,
        None => return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Cuenta no encontrada".into() }))),
    };

    let password_valid = if account.password_hash.starts_with("$argon2") {
        PasswordHash::new(&account.password_hash)
            .ok()
            .map(|parsed| Argon2::default().verify_password(body.password.as_bytes(), &parsed).is_ok())
            .unwrap_or(false)
    } else {
        account.password_hash == body.password
    };

    if password_valid && !account.password_hash.starts_with("$argon2")
        && let Ok(new_hash) = Argon2::default().hash_password(body.password.as_bytes()) {
            let _ = state.db.update_password_hash(&account.id, &new_hash.to_string()).await;
        }

    if !password_valid {
        return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { error: "Contraseña incorrecta".into() })));
    }

    let character = state.db.find_character_by_account(&account.id).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error buscando personaje".into() })))?;

    let character = match character {
        Some(c) => c,
        None => return Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "No tienes personaje creado".into() }))),
    };

    let ticket = uuid::Uuid::new_v4().to_string();
    state.db.create_ticket_for_login(&account.id, &character.id, &ticket).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error creando ticket".into() })))?;

    Ok(Json(LoginResponse { ticket, account_id: account.id, name: character.name }))
}

#[derive(Deserialize)]
struct RegisterRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    success: bool,
    account_id: String,
}

async fn register(
    State(state): State<ApiState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, (StatusCode, Json<ErrorResponse>)> {
    if body.name.len() < 2 || body.name.len() > 20 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Nombre debe tener 2-20 caracteres".into() })));
    }
    if body.password.len() < 4 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Contraseña debe tener al menos 4 caracteres".into() })));
    }

    let existing = state.db.find_account_by_email(&body.email).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;

    if existing.is_some() {
        return Err((StatusCode::CONFLICT, Json(ErrorResponse { error: "Email ya registrado".into() })));
    }

    let password_hash = Argon2::default()
        .hash_password(body.password.as_bytes())
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error procesando contraseña".into() })))?
        .to_string();

    let account_id = uuid::Uuid::new_v4().to_string();
    state.db.create_account(&account_id, &body.email, &password_hash).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error creando cuenta".into() })))?;

    let char_id = uuid::Uuid::new_v4().to_string();
    state.db.create_character(&char_id, &account_id, &body.name).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error creando personaje".into() })))?;

    Ok(Json(RegisterResponse { success: true, account_id }))
}

#[derive(Deserialize)]
struct CreateCharacterRequest {
    name: String,
    class: String,
    #[allow(dead_code)]
    race: Option<String>,
    #[allow(dead_code)]
    gender: Option<String>,
}

async fn create_character(
    State(state): State<ApiState>,
    Json(body): Json<CreateCharacterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let trimmed = body.name.trim().to_string();
    if trimmed.len() < 3 || trimmed.len() > 16 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Nombre debe tener 3-16 caracteres".into() })));
    }

    let id_clase = match body.class.as_str() {
        "mago" => 1,
        "clerigo" => 2,
        "guerrero" => 3,
        "asesino" => 4,
        "bardo" => 5,
        "druida" => 6,
        "paladin" => 7,
        "cazador" => 8,
        _ => 3,
    };

    let char_id = uuid::Uuid::new_v4().to_string();
    let account_id = uuid::Uuid::new_v4().to_string();

    state.db.create_character_with_class(&char_id, &account_id, &trimmed, id_clase).await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                (StatusCode::CONFLICT, Json(ErrorResponse { error: "Ese nombre ya está en uso".into() }))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error creando personaje".into() }))
            }
        })?;

    Ok(Json(serde_json::json!({ "success": true, "characterId": char_id })))
}

async fn me() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "not_implemented" }))
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct PasswordResetRequest {
    email: String,
}

async fn request_password_reset(
    Json(_body): Json<PasswordResetRequest>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "success": true, "message": "Si el email existe, recibirás instrucciones" }))
}

async fn signout() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "success": true }))
}

async fn ranking(
    State(state): State<ApiState>,
) -> Result<Json<Vec<RankEntry>>, (StatusCode, Json<ErrorResponse>)> {
    let entries = state.db.get_ranking().await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;
    Ok(Json(entries))
}

async fn arenas() -> Json<Vec<serde_json::Value>> {
    Json(vec![
        serde_json::json!({
            "id": "arena-1v1",
            "name": "Arena 1v1",
            "capacity": 2,
            "memberCount": 0,
        }),
        serde_json::json!({
            "id": "arena-2v2",
            "name": "Arena 2v2",
            "capacity": 4,
            "memberCount": 0,
        }),
        serde_json::json!({
            "id": "arena-ffa",
            "name": "Arena FFA (5 jugadores)",
            "capacity": 5,
            "memberCount": 0,
        }),
    ])
}

async fn users_online_stats(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let mut total = 0u32;
    for scene_ref in state.world.scenes.iter() {
        total += scene_ref.value().players.len() as u32;
    }

    Json(serde_json::json!([{
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "totalUsers": total,
        "pveUsers": total,
        "pvpUsers": 0
    }]))
}

async fn wiki(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let items: Vec<serde_json::Value> = state.world.gd().objects.iter()
        .map(|(&id, obj)| serde_json::json!({
            "id": id,
            "name": obj.name,
            "type": obj.obj_type,
            "grhIndex": obj.grh_index,
        }))
        .collect();

    let npcs: Vec<serde_json::Value> = state.world.gd().npcs.iter()
        .map(|(&id, npc)| serde_json::json!({
            "id": id,
            "name": npc.name,
            "hp": npc.max_hp,
            "exp": npc.exp,
        }))
        .collect();

    let spells: Vec<serde_json::Value> = state.world.gd().spells.iter()
        .map(|(&id, spell)| serde_json::json!({
            "id": id,
            "name": spell.name,
            "manaRequired": spell.mana_required,
            "type": spell.spell_type,
        }))
        .collect();

    Ok(Json(serde_json::json!({
        "items": items,
        "npcs": npcs,
        "spells": spells,
    })))
}

async fn health() -> &'static str {
    "OK"
}

async fn readiness(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let is_draining = state.metrics.shutting_down.load(std::sync::atomic::Ordering::Relaxed);
    if is_draining {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(Json(serde_json::json!({ "status": "ready" })))
}

async fn server_metrics(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let total_players: usize = state.world.scenes.iter().map(|s| s.players.len()).sum();
    let total_npcs: usize = state.world.scenes.iter().map(|s| s.npcs.len()).sum();
    let total_scenes = state.world.scenes.len();

    let tick_count = state.metrics.tick_time_count.load(std::sync::atomic::Ordering::Relaxed);
    let tick_time_avg_us = if tick_count > 0 {
        state.metrics.tick_time_sum_us.load(std::sync::atomic::Ordering::Relaxed) / tick_count
    } else {
        0
    };
    let c = &state.metrics.packets_by_category;
    Json(serde_json::json!({
        "uptime_seconds": state.metrics.uptime_secs(),
        "total_connections": state.metrics.total_connections.load(std::sync::atomic::Ordering::Relaxed),
        "active_connections": state.metrics.active_connections.load(std::sync::atomic::Ordering::Relaxed),
        "total_packets_in": state.metrics.total_packets_in.load(std::sync::atomic::Ordering::Relaxed),
        "total_packets_out": state.metrics.total_packets_out.load(std::sync::atomic::Ordering::Relaxed),
        "packets_rejected": state.metrics.packets_rejected.load(std::sync::atomic::Ordering::Relaxed),
        "packets_dropped_no_char": state.metrics.packets_dropped_no_char.load(std::sync::atomic::Ordering::Relaxed),
        "online_players": total_players,
        "total_npcs": total_npcs,
        "active_scenes": total_scenes,
        "reconnect_tokens_active": state.reconnect_mgr.active_count(),
        "shutting_down": state.metrics.shutting_down.load(std::sync::atomic::Ordering::Relaxed),
        "tick_time_max_us": state.metrics.tick_time_max_us.load(std::sync::atomic::Ordering::Relaxed),
        "tick_time_avg_us": tick_time_avg_us,
        "packets_by_category": {
            "auth": c.auth.load(std::sync::atomic::Ordering::Relaxed),
            "movement": c.movement.load(std::sync::atomic::Ordering::Relaxed),
            "combat": c.combat.load(std::sync::atomic::Ordering::Relaxed),
            "dialog": c.dialog.load(std::sync::atomic::Ordering::Relaxed),
            "inventory": c.inventory.load(std::sync::atomic::Ordering::Relaxed),
            "commerce": c.commerce.load(std::sync::atomic::Ordering::Relaxed),
            "social": c.social.load(std::sync::atomic::Ordering::Relaxed),
            "crafting": c.crafting.load(std::sync::atomic::Ordering::Relaxed),
            "gathering": c.gathering.load(std::sync::atomic::Ordering::Relaxed),
            "bank": c.bank.load(std::sync::atomic::Ordering::Relaxed),
            "market": c.market.load(std::sync::atomic::Ordering::Relaxed),
            "challenge": c.challenge.load(std::sync::atomic::Ordering::Relaxed),
            "admin": c.admin.load(std::sync::atomic::Ordering::Relaxed),
            "system": c.system.load(std::sync::atomic::Ordering::Relaxed),
        },
    }))
}

async fn prometheus_metrics(
    State(state): State<ApiState>,
) -> (StatusCode, [(axum::http::header::HeaderName, &'static str); 1], String) {
    use std::sync::atomic::Ordering::Relaxed;
    use std::fmt::Write;

    let m = &state.metrics;
    let total_players: usize = state.world.scenes.iter().map(|s| s.players.len()).sum();
    let total_npcs: usize = state.world.scenes.iter().map(|s| s.npcs.len()).sum();
    let active_scenes = state.world.scenes.len();

    let mut out = String::with_capacity(2048);

    let _ = writeln!(out, "# HELP openao_uptime_seconds Server uptime in seconds.");
    let _ = writeln!(out, "# TYPE openao_uptime_seconds gauge");
    let _ = writeln!(out, "openao_uptime_seconds {}", m.uptime_secs());

    let _ = writeln!(out, "# HELP openao_connections_total Total WebSocket connections since start.");
    let _ = writeln!(out, "# TYPE openao_connections_total counter");
    let _ = writeln!(out, "openao_connections_total {}", m.total_connections.load(Relaxed));

    let _ = writeln!(out, "# HELP openao_connections_active Current active WebSocket connections.");
    let _ = writeln!(out, "# TYPE openao_connections_active gauge");
    let _ = writeln!(out, "openao_connections_active {}", m.active_connections.load(Relaxed));

    let _ = writeln!(out, "# HELP openao_packets_in_total Total packets received.");
    let _ = writeln!(out, "# TYPE openao_packets_in_total counter");
    let _ = writeln!(out, "openao_packets_in_total {}", m.total_packets_in.load(Relaxed));

    let _ = writeln!(out, "# HELP openao_packets_out_total Total packets sent.");
    let _ = writeln!(out, "# TYPE openao_packets_out_total counter");
    let _ = writeln!(out, "openao_packets_out_total {}", m.total_packets_out.load(Relaxed));

    let _ = writeln!(out, "# HELP openao_packets_rejected_total Total packets rejected (rate limit or oversized).");
    let _ = writeln!(out, "# TYPE openao_packets_rejected_total counter");
    let _ = writeln!(out, "openao_packets_rejected_total {}", m.packets_rejected.load(Relaxed));

    let _ = writeln!(out, "# HELP openao_players_online Current online player count.");
    let _ = writeln!(out, "# TYPE openao_players_online gauge");
    let _ = writeln!(out, "openao_players_online {}", total_players);

    let _ = writeln!(out, "# HELP openao_npcs_active Current active NPC count.");
    let _ = writeln!(out, "# TYPE openao_npcs_active gauge");
    let _ = writeln!(out, "openao_npcs_active {}", total_npcs);

    let _ = writeln!(out, "# HELP openao_scenes_active Current active scene/map count.");
    let _ = writeln!(out, "# TYPE openao_scenes_active gauge");
    let _ = writeln!(out, "openao_scenes_active {}", active_scenes);

    let _ = writeln!(out, "# HELP openao_reconnect_tokens_active Current active reconnect tokens.");
    let _ = writeln!(out, "# TYPE openao_reconnect_tokens_active gauge");
    let _ = writeln!(out, "openao_reconnect_tokens_active {}", state.reconnect_mgr.active_count());

    let _ = writeln!(out, "# HELP openao_tick_time_max_us Maximum tick processing time in microseconds.");
    let _ = writeln!(out, "# TYPE openao_tick_time_max_us gauge");
    let _ = writeln!(out, "openao_tick_time_max_us {}", m.tick_time_max_us.load(Relaxed));
    let tick_count = m.tick_time_count.load(Relaxed);
    let tick_avg = if tick_count > 0 { m.tick_time_sum_us.load(Relaxed) / tick_count } else { 0 };
    let _ = writeln!(out, "# HELP openao_tick_time_avg_us Average tick processing time in microseconds.");
    let _ = writeln!(out, "# TYPE openao_tick_time_avg_us gauge");
    let _ = writeln!(out, "openao_tick_time_avg_us {}", tick_avg);

    let _ = writeln!(out, "# HELP openao_packets_dropped_no_char_total Packets dropped because no character was connected.");
    let _ = writeln!(out, "# TYPE openao_packets_dropped_no_char_total counter");
    let _ = writeln!(out, "openao_packets_dropped_no_char_total {}", m.packets_dropped_no_char.load(Relaxed));

    let _ = writeln!(out, "# HELP openao_packets_by_category_total Packets received per route category.");
    let _ = writeln!(out, "# TYPE openao_packets_by_category_total counter");
    let c = &m.packets_by_category;
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"auth\"}} {}", c.auth.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"movement\"}} {}", c.movement.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"combat\"}} {}", c.combat.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"dialog\"}} {}", c.dialog.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"inventory\"}} {}", c.inventory.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"commerce\"}} {}", c.commerce.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"social\"}} {}", c.social.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"crafting\"}} {}", c.crafting.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"gathering\"}} {}", c.gathering.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"bank\"}} {}", c.bank.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"market\"}} {}", c.market.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"challenge\"}} {}", c.challenge.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"admin\"}} {}", c.admin.load(Relaxed));
    let _ = writeln!(out, "openao_packets_by_category_total{{category=\"system\"}} {}", c.system.load(Relaxed));

    let _ = writeln!(out, "# HELP openao_shutting_down Whether the server is in shutdown drain mode.");
    let _ = writeln!(out, "# TYPE openao_shutting_down gauge");
    let _ = writeln!(out, "openao_shutting_down {}", if m.shutting_down.load(Relaxed) { 1 } else { 0 });

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        out,
    )
}

// ─── Arenas REST ───

async fn arenas_rooms(
    State(state): State<ApiState>,
) -> Json<Vec<serde_json::Value>> {
    let Ok(mgr) = state.world.challenges.try_lock() else {
        return Json(vec![]);
    };
    let rooms: Vec<serde_json::Value> = mgr.list_open().iter().map(|room| {
        serde_json::json!({
            "id": room.id().to_string(),
            "capacity": room.config().capacity,
            "members": room.len(),
            "phase": format!("{:?}", room.phase()),
        })
    }).collect();
    Json(rooms)
}

#[derive(Deserialize)]
struct CreateRoomReq {
    mode: String,
    player_name: String,
    entity_id: u32,
    level: Option<i32>,
    class_name: Option<String>,
}

async fn arenas_create_room(
    State(state): State<ApiState>,
    Json(body): Json<CreateRoomReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let team_size: usize = match body.mode.as_str() {
        "1v1" => 1,
        "2v2" => 2,
        _ => return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "Modo inválido (1v1 o 2v2)".into() }))),
    };
    let data = crate::gameplay::rooms::ChallengeParticipantData {
        character_id: uuid::Uuid::new_v4(),
        name: body.player_name,
        level: body.level.unwrap_or(1),
        class_name: body.class_name.unwrap_or_else(|| "Guerrero".into()),
        race_name: "Humano".into(),
    };
    let now = chrono::Utc::now().timestamp_millis();
    let Ok(mut mgr) = state.world.challenges.try_lock() else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse { error: "Busy, retry.".into() })));
    };
    match mgr.create(team_size, body.entity_id, data, now) {
        Ok(room_id) => Ok(Json(serde_json::json!({ "roomId": room_id.to_string() }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("{:?}", e) }))),
    }
}

async fn arenas_join_room(
    State(state): State<ApiState>,
    axum::extract::Path(room_id_str): axum::extract::Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let room_id: uuid::Uuid = room_id_str.parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "ID de sala inválido".into() })))?;
    let entity_id = body.get("entity_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let player_name = body.get("player_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let data = crate::gameplay::rooms::ChallengeParticipantData {
        character_id: uuid::Uuid::new_v4(),
        name: player_name,
        level: body.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32,
        class_name: body.get("class_name").and_then(|v| v.as_str()).unwrap_or("Guerrero").to_string(),
        race_name: "Humano".into(),
    };
    let Ok(mut mgr) = state.world.challenges.try_lock() else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse { error: "Busy, retry.".into() })));
    };
    match mgr.join(room_id, entity_id, data) {
        Ok(is_full) => Ok(Json(serde_json::json!({ "success": true, "started": is_full }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("{:?}", e) }))),
    }
}

async fn arenas_leave_room(
    State(state): State<ApiState>,
    axum::extract::Path(room_id_str): axum::extract::Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let room_id: uuid::Uuid = room_id_str.parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "ID de sala inválido".into() })))?;
    let entity_id = body.get("entity_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let Ok(mut mgr) = state.world.challenges.try_lock() else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse { error: "Busy, retry.".into() })));
    };
    match mgr.leave(room_id, &entity_id) {
        Ok(_) => Ok(Json(serde_json::json!({ "success": true }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("{:?}", e) }))),
    }
}

async fn arenas_cancel_room(
    State(state): State<ApiState>,
    axum::extract::Path(room_id_str): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let room_id: uuid::Uuid = room_id_str.parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "ID de sala inválido".into() })))?;
    let Ok(mut mgr) = state.world.challenges.try_lock() else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse { error: "Busy, retry.".into() })));
    };
    mgr.cancel(room_id);
    Ok(Json(serde_json::json!({ "success": true })))
}

// ─── Clans REST ───

async fn clans_list(
    State(state): State<ApiState>,
) -> Json<Vec<serde_json::Value>> {
    let list: Vec<serde_json::Value> = state.world.clans.iter().map(|entry| {
        let c = entry.value();
        serde_json::json!({
            "id": c.id,
            "name": c.name,
            "leader": c.leader_name,
            "members": c.member_ids.len(),
        })
    }).collect();
    Json(list)
}

async fn clan_detail(
    State(state): State<ApiState>,
    axum::extract::Path(clan_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match state.world.clans.get(&clan_id) {
        Some(entry) => {
            let c = entry.value();
            Ok(Json(serde_json::json!({
                "id": c.id,
                "name": c.name,
                "leader": c.leader_name,
                "co_leader_ids": c.co_leader_ids,
                "member_ids": c.member_ids,
            })))
        }
        None => Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "Clan no encontrado".into() }))),
    }
}

// ─── Runtime Config ───

async fn get_runtime_config(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "double_exp": state.world.double_exp.load(std::sync::atomic::Ordering::Relaxed),
        "double_gold": state.world.double_gold.load(std::sync::atomic::Ordering::Relaxed),
        "tick_rate": 60,
    }))
}

#[derive(Deserialize)]
struct RuntimeConfigUpdate {
    double_exp: Option<bool>,
    double_gold: Option<bool>,
}

async fn set_runtime_config(
    State(state): State<ApiState>,
    Json(body): Json<RuntimeConfigUpdate>,
) -> Json<serde_json::Value> {
    if let Some(v) = body.double_exp {
        state.world.double_exp.store(v, std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(v) = body.double_gold {
        state.world.double_gold.store(v, std::sync::atomic::Ordering::Relaxed);
    }
    Json(serde_json::json!({ "success": true }))
}

// ─── Character Settings ───

async fn get_character_settings(
    State(state): State<ApiState>,
    axum::extract::Path(char_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let settings = state.db.get_character_settings(&char_id).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;
    match settings {
        Some(json_str) => Ok(Json(serde_json::from_str(&json_str).unwrap_or(serde_json::json!({})))),
        None => Ok(Json(serde_json::json!({}))),
    }
}

async fn save_character_settings(
    State(state): State<ApiState>,
    axum::extract::Path(char_id): axum::extract::Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let json_str = serde_json::to_string(&body).unwrap_or_default();
    state.db.save_character_settings(&char_id, &json_str).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error guardando settings".into() })))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ─── World Builder API ───

#[derive(Deserialize)]
struct PaintTilesReq {
    layer: u8,
    tiles: Vec<TilePaint>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct TilePaint {
    x: i32,
    y: i32,
    grh: i32,
}

async fn world_builder_paint_tiles(
    State(state): State<ApiState>,
    axum::extract::Path(map_id): axum::extract::Path<i32>,
    Json(body): Json<PaintTilesReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let gd = state.world.gd();
    let td = gd.tile_data.get(&map_id);
    if td.is_none() {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "Mapa no encontrado en tile data".into() })));
    }
    let count = body.tiles.len();
    // In a full implementation this would persist tile changes to terrain data files.
    // For now we acknowledge the request and log it.
    tracing::info!("World builder: paint {} tiles on map {} layer {}", count, map_id, body.layer);
    Ok(Json(serde_json::json!({ "success": true, "painted": count })))
}

#[derive(Deserialize)]
struct PlaceEntityReq {
    entity_type: String,
    npc_id: Option<i32>,
    x: i32,
    y: i32,
}

async fn world_builder_place_entity(
    State(state): State<ApiState>,
    axum::extract::Path(map_id): axum::extract::Path<i32>,
    Json(body): Json<PlaceEntityReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match body.entity_type.as_str() {
        "npc" => {
            let npc_type = body.npc_id.unwrap_or(1);
            let scene = state.world.get_or_create_scene(map_id);
            let eid = state.world.next_id();
            let npc_data = state.world.gd().npcs.get(&npc_type).cloned();
            if let Some(npc) = npc_data {
                let npc_spells: Vec<crate::world::NpcSpellSlot> = npc.spells.iter()
                    .map(|s| crate::world::NpcSpellSlot { spell_id: s.id_spell })
                    .collect();
                let npc_state = crate::world::NpcState {
                    id: eid,
                    npc_type,
                    pos: crate::world::Position { map: map_id, x: body.x, y: body.y },
                    heading: 3,
                    hp: npc.max_hp,
                    max_hp: npc.max_hp,
                    min_hit: npc.min_hit,
                    max_hit: npc.max_hit,
                    defense: npc.def,
                    exp_reward: npc.exp,
                    movement: npc.movement,
                    dead: false,
                    paralizado: false,
                    inmovilizado: false,
                    cc_expire_tick: 0,
                    aggro_target: None,
                    spells: npc_spells,
                    spell_cast_interval_ms: npc.spell_cast_interval_ms.unwrap_or(2000),
                    last_spell_cast_at: 0,
                    spell_range: npc.spell_range.unwrap_or(8),
                    magic_def: npc.magic_def,
                    magic_resistance: npc.magic_resistance,
                    summoned_by: None,
                    summon_expires_at_ms: 0,
                    admin_bot_owner: None,
                };
                scene.npcs.insert(eid, npc_state);
                scene.aoi_insert(eid, &crate::world::Position { map: map_id, x: body.x, y: body.y });
                Ok(Json(serde_json::json!({ "success": true, "entityId": eid })))
            } else {
                Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "NPC type no encontrado".into() })))
            }
        }
        _ => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "entity_type debe ser 'npc'".into() }))),
    }
}

async fn world_builder_remove_entity(
    State(state): State<ApiState>,
    axum::extract::Path((map_id, entity_type, entity_id)): axum::extract::Path<(i32, String, u32)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let scene = state.world.get_or_create_scene(map_id);
    match entity_type.as_str() {
        "npc" => {
            if let Some((_, _npc)) = scene.npcs.remove(&entity_id) {
                scene.aoi_remove(entity_id);
                Ok(Json(serde_json::json!({ "success": true })))
            } else {
                Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "NPC no encontrado".into() })))
            }
        }
        _ => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "entity_type debe ser 'npc'".into() }))),
    }
}

#[derive(Deserialize)]
struct SetBlockedReq {
    tiles: Vec<BlockedTile>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct BlockedTile {
    x: i32,
    y: i32,
    blocked: bool,
}

async fn world_builder_set_blocked(
    axum::extract::Path(map_id): axum::extract::Path<i32>,
    Json(body): Json<SetBlockedReq>,
) -> Json<serde_json::Value> {
    let count = body.tiles.len();
    tracing::info!("World builder: set {} blocked tiles on map {}", count, map_id);
    Json(serde_json::json!({ "success": true, "updated": count }))
}

#[derive(Deserialize)]
struct SetTeleportReq {
    x: i32,
    y: i32,
    target_map: i32,
    target_x: i32,
    target_y: i32,
}

async fn world_builder_set_teleport(
    axum::extract::Path(map_id): axum::extract::Path<i32>,
    Json(body): Json<SetTeleportReq>,
) -> Json<serde_json::Value> {
    tracing::info!("World builder: set teleport on map {} at ({},{}) -> map {} ({},{})",
        map_id, body.x, body.y, body.target_map, body.target_x, body.target_y);
    Json(serde_json::json!({ "success": true }))
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct RankEntry {
    pub name: String,
    pub level: i32,
    pub gold: i32,
}

// --- Multi-character endpoints ---

async fn list_characters(
    State(state): State<ApiState>,
    axum::extract::Path(account_id): axum::extract::Path<String>,
) -> Result<Json<Vec<crate::persistence::CharacterSummary>>, (StatusCode, Json<ErrorResponse>)> {
    let chars = state.db.list_characters_by_account(&account_id).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;
    Ok(Json(chars))
}

#[derive(Deserialize)]
struct DeleteCharReq {
    account_id: String,
}

async fn delete_character_endpoint(
    State(state): State<ApiState>,
    axum::extract::Path(char_id): axum::extract::Path<String>,
    Json(body): Json<DeleteCharReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let deleted = state.db.delete_character(&char_id, &body.account_id).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;
    if !deleted {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse { error: "Personaje no encontrado o no pertenece a esta cuenta".into() })));
    }
    Ok(Json(serde_json::json!({ "success": true })))
}

// --- Moderation REST endpoints ---

#[derive(Deserialize)]
struct ModerationReq {
    target: String,
    reason: Option<String>,
    admin: Option<String>,
}

async fn admin_ban(
    State(state): State<ApiState>,
    Json(body): Json<ModerationReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    state.db.add_ban(&body.target, body.reason.as_deref().unwrap_or("REST ban"), body.admin.as_deref().unwrap_or("admin")).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn admin_unban(
    State(state): State<ApiState>,
    Json(body): Json<ModerationReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let removed = state.db.remove_ban(&body.target).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;
    Ok(Json(serde_json::json!({ "success": true, "removed": removed })))
}

async fn admin_mute(
    State(state): State<ApiState>,
    Json(body): Json<ModerationReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    state.db.add_mute(&body.target, body.reason.as_deref().unwrap_or("REST mute"), body.admin.as_deref().unwrap_or("admin")).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn admin_unmute(
    State(state): State<ApiState>,
    Json(body): Json<ModerationReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let removed = state.db.remove_mute(&body.target).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;
    Ok(Json(serde_json::json!({ "success": true, "removed": removed })))
}

async fn admin_ip_ban(
    State(state): State<ApiState>,
    Json(body): Json<ModerationReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    state.db.add_ip_ban(&body.target, body.reason.as_deref().unwrap_or("REST IP ban"), body.admin.as_deref().unwrap_or("admin")).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn admin_ip_unban(
    State(state): State<ApiState>,
    Json(body): Json<ModerationReq>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let removed = state.db.remove_ip_ban(&body.target).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Error de base de datos".into() })))?;
    Ok(Json(serde_json::json!({ "success": true, "removed": removed })))
}

// --- Game Data Admin API ---

async fn admin_list_objects(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let gd = state.world.gd();
    let items: Vec<serde_json::Value> = gd.objects.iter().map(|(id, obj)| {
        serde_json::json!({
            "id": id,
            "name": obj.name,
            "obj_type": obj.obj_type,
            "grh_index": obj.grh_index,
        })
    }).collect();
    Json(serde_json::json!({ "items": items, "count": items.len() }))
}

async fn admin_list_npcs(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let gd = state.world.gd();
    let npcs: Vec<serde_json::Value> = gd.npcs.iter().map(|(id, npc)| {
        serde_json::json!({
            "id": id,
            "name": npc.name,
            "hp": npc.hp,
            "exp": npc.exp,
        })
    }).collect();
    Json(serde_json::json!({ "npcs": npcs, "count": npcs.len() }))
}

async fn admin_list_spells(
    State(state): State<ApiState>,
) -> Json<serde_json::Value> {
    let gd = state.world.gd();
    let spells: Vec<serde_json::Value> = gd.spells.iter().map(|(id, spell)| {
        serde_json::json!({
            "id": id,
            "name": spell.name,
            "mana_cost": spell.mana_required,
            "min_hp": spell.min_hp,
            "max_hp": spell.max_hp,
        })
    }).collect();
    Json(serde_json::json!({ "spells": spells, "count": spells.len() }))
}
