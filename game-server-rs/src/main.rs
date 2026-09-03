mod api;
mod elr2;
mod error;
mod game_data;
mod game_module;
mod gateway;
mod gameplay;
mod persistence;
mod rate_limit;
mod reconnect;
mod replication;
mod routes;
mod simulation;
mod world;

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use futures_util::StreamExt;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response, ErrorResponse};
use tokio_tungstenite::tungstenite::http;
use tracing::{info, error, warn};

use crate::elr2::SUBPROTOCOL;
use crate::gateway::GameSession;
use crate::reconnect::ReconnectManager;
use crate::world::GameWorld;
use crate::persistence::Database;

/// Server metrics tracked via atomics for lock-free concurrent access.
pub struct ServerMetrics {
    pub total_connections: AtomicU64,
    pub active_connections: AtomicU64,
    pub total_packets_in: AtomicU64,
    pub total_packets_out: AtomicU64,
    pub packets_rejected: AtomicU64,
    pub packets_dropped_no_char: AtomicU64,
    pub uptime_start: Instant,
    pub shutting_down: AtomicBool,
    pub packets_by_category: CategoryCounters,
    pub current_tick: AtomicU64,
    pub tick_time_max_us: AtomicU64,
    pub tick_time_sum_us: AtomicU64,
    pub tick_time_count: AtomicU64,
}

pub struct CategoryCounters {
    pub auth: AtomicU64,
    pub movement: AtomicU64,
    pub combat: AtomicU64,
    pub dialog: AtomicU64,
    pub inventory: AtomicU64,
    pub commerce: AtomicU64,
    pub social: AtomicU64,
    pub crafting: AtomicU64,
    pub gathering: AtomicU64,
    pub bank: AtomicU64,
    pub market: AtomicU64,
    pub challenge: AtomicU64,
    pub admin: AtomicU64,
    pub system: AtomicU64,
}

impl CategoryCounters {
    fn new() -> Self {
        Self {
            auth: AtomicU64::new(0),
            movement: AtomicU64::new(0),
            combat: AtomicU64::new(0),
            dialog: AtomicU64::new(0),
            inventory: AtomicU64::new(0),
            commerce: AtomicU64::new(0),
            social: AtomicU64::new(0),
            crafting: AtomicU64::new(0),
            gathering: AtomicU64::new(0),
            bank: AtomicU64::new(0),
            market: AtomicU64::new(0),
            challenge: AtomicU64::new(0),
            admin: AtomicU64::new(0),
            system: AtomicU64::new(0),
        }
    }

    pub fn increment(&self, category: crate::routes::RouteCategory) {
        use crate::routes::RouteCategory;
        match category {
            RouteCategory::Auth => &self.auth,
            RouteCategory::Movement => &self.movement,
            RouteCategory::Combat => &self.combat,
            RouteCategory::Dialog => &self.dialog,
            RouteCategory::Inventory => &self.inventory,
            RouteCategory::Commerce => &self.commerce,
            RouteCategory::Social => &self.social,
            RouteCategory::Crafting => &self.crafting,
            RouteCategory::Gathering => &self.gathering,
            RouteCategory::Bank => &self.bank,
            RouteCategory::Market => &self.market,
            RouteCategory::Challenge => &self.challenge,
            RouteCategory::Admin => &self.admin,
            RouteCategory::System => &self.system,
        }.fetch_add(1, Ordering::Relaxed);
    }
}

impl ServerMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            total_connections: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            total_packets_in: AtomicU64::new(0),
            total_packets_out: AtomicU64::new(0),
            packets_rejected: AtomicU64::new(0),
            packets_dropped_no_char: AtomicU64::new(0),
            uptime_start: Instant::now(),
            shutting_down: AtomicBool::new(false),
            packets_by_category: CategoryCounters::new(),
            current_tick: AtomicU64::new(0),
            tick_time_max_us: AtomicU64::new(0),
            tick_time_sum_us: AtomicU64::new(0),
            tick_time_count: AtomicU64::new(0),
        })
    }

    pub fn uptime_secs(&self) -> u64 {
        self.uptime_start.elapsed().as_secs()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "openao_server=info".parse().unwrap()),
        )
        .init();

    info!("╔══════════════════════════════════════╗");
    info!("║       OpenAO Game Server v0.1.0      ║");
    info!("╚══════════════════════════════════════╝");

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:openao.db".into());
    let bind_addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:7666".into())
        .parse()
        .expect("BIND_ADDR must be a valid socket address (e.g. 0.0.0.0:7666)");

    info!("  Database : {database_url}");
    info!("  WS bind  : {bind_addr}");

    let db = Database::connect(&database_url).await?;

    info!("Running migrations...");
    db.run_migrations().await?;

    info!("Seeding test data...");
    db.seed_test_data().await?;

    info!("Loading game data (items, NPCs, spells, maps)...");
    let data_dir = std::path::Path::new("data");
    let game_data = Arc::new(game_data::GameData::load(data_dir)?);

    let reconnect_mgr = ReconnectManager::new();
    let metrics = ServerMetrics::new();

    info!("Building game module registry...");
    let router = Arc::new(game_module::build_router_from_modules());

    info!("Loading game world...");
    let world = Arc::new(GameWorld::new(db, game_data));

    {
        let ban_count = match world.db.load_all_bans().await {
            Ok(bans) => {
                let count = bans.len();
                for acc_id in bans {
                    world.banned_accounts.insert(acc_id.clone(), acc_id);
                }
                count
            }
            Err(e) => { tracing::warn!("Failed to load bans: {e}"); 0 }
        };
        if ban_count > 0 {
            info!("Loaded {} ban(s) from database", ban_count);
        }

        let ip_ban_count = match world.db.load_all_ip_bans().await {
            Ok(ips) => {
                let count = ips.len();
                for ip in ips {
                    world.banned_ips.insert(ip.clone(), ip);
                }
                count
            }
            Err(e) => { tracing::warn!("Failed to load IP bans: {e}"); 0 }
        };
        if ip_ban_count > 0 {
            info!("Loaded {} IP ban(s) from database", ip_ban_count);
        }
    }

    info!("Starting game loop (60 TPS)...");
    {
        let game_loop_world = world.clone();
        let game_loop_reconnect = reconnect_mgr.clone();
        let game_loop_metrics = metrics.clone();
        let rt_handle = tokio::runtime::Handle::current();
        std::thread::Builder::new()
            .name("game-loop".into())
            .spawn(move || {
                loop {
                    let w = game_loop_world.clone();
                    let r = game_loop_reconnect.clone();
                    let m = game_loop_metrics.clone();
                    let rt = rt_handle.clone();
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                        crate::simulation::run_game_loop(w, r, m, rt);
                    }));
                    match result {
                        Ok(()) => {
                            tracing::error!("Game loop exited unexpectedly (no panic). Restarting in 1s...");
                        }
                        Err(panic_info) => {
                            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                                s.to_string()
                            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                                s.clone()
                            } else {
                                "unknown panic".to_string()
                            };
                            tracing::error!("GAME LOOP THREAD PANICKED: {}. Restarting in 1s...", msg);
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            })
            .expect("Failed to spawn game loop thread");
    }

    let http_addr: SocketAddr = std::env::var("HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:7667".into())
        .parse()
        .expect("HTTP_ADDR must be a valid socket address (e.g. 0.0.0.0:7667)");

    let api_db = Arc::new(world.db.clone());
    let api_router = api::create_router(api_db, world.clone(), metrics.clone(), reconnect_mgr.clone());

    info!("  HTTP API : {http_addr}");
    let http_listener = tokio::net::TcpListener::bind(http_addr).await?;
    tokio::spawn(async move {
        axum::serve(http_listener, api_router).await.unwrap();
    });

    let ip_limiter = rate_limit::IpRateLimiter::new(20, 60);

    info!("Starting ELR2 WebSocket server on {bind_addr} (subprotocol: {SUBPROTOCOL})");
    let listener = TcpListener::bind(bind_addr).await?;

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
                    accept_result = listener.accept() => {
                        let (stream, addr) = accept_result?;

                        if metrics.shutting_down.load(Ordering::Relaxed) {
                            info!("Rejecting connection from {addr} — server is draining");
                            continue;
                        }

                        if !ip_limiter.check(addr.ip()) {
                            warn!("Rejecting connection from {addr} — IP rate limited");
                            continue;
                        }

                        if world.banned_ips.contains_key(&addr.ip().to_string()) {
                            warn!("Rejecting connection from {addr} — IP banned");
                            continue;
                        }

                        let world = world.clone();
                        let reconnect_mgr = reconnect_mgr.clone();
                        let conn_metrics = metrics.clone();
                        let session_metrics = metrics.clone();
                        let session_router = router.clone();

                        conn_metrics.total_connections.fetch_add(1, Ordering::Relaxed);
                        conn_metrics.active_connections.fetch_add(1, Ordering::Relaxed);

                        tokio::spawn(async move {
                    let negotiated_elr2 = Arc::new(std::sync::atomic::AtomicBool::new(false));
                    let elr2_flag = negotiated_elr2.clone();

                    let ws_stream = match tokio_tungstenite::accept_hdr_async(
                        stream,
                        #[allow(clippy::result_large_err)]
                        move |request: &Request, mut response: Response| -> Result<Response, ErrorResponse> {
                            let protocols = request
                                .headers()
                                .get(http::header::SEC_WEBSOCKET_PROTOCOL)
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("");

                            let has_elura = protocols.split(',').any(|p| p.trim() == SUBPROTOCOL);

                            if has_elura {
                                response.headers_mut().insert(
                                    http::header::SEC_WEBSOCKET_PROTOCOL,
                                    http::HeaderValue::from_static(SUBPROTOCOL),
                                );
                                elr2_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                                Ok(response)
                            } else {
                                Ok(response)
                            }
                        },
                    )
                    .await
                    {
                        Ok(ws) => ws,
                        Err(e) => {
                            error!("WebSocket handshake failed for {addr}: {e}");
                            return;
                        }
                    };

                            let is_elr2 = negotiated_elr2.load(std::sync::atomic::Ordering::SeqCst);
                            info!("New connection from {addr} (ELR2: {is_elr2})");
                            let (write, read) = ws_stream.split();
                            let session = GameSession::new(addr, world, is_elr2, reconnect_mgr, session_metrics, session_router);
                            if let Err(e) = session.run(write, read).await {
                                warn!("Session {addr} ended with error: {e}");
                            }

                            conn_metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
                        });
            }
            _ = &mut shutdown => {
                info!("Shutdown signal received, stopping server...");
                break;
            }
        }
    }

    info!("Server stopped. Beginning graceful drain...");
    metrics.shutting_down.store(true, Ordering::SeqCst);

    let drain_start = Instant::now();
    let drain_timeout = std::time::Duration::from_secs(10);
    let active = metrics.active_connections.load(Ordering::Relaxed);
    if active > 0 {
        info!("Waiting up to {}s for {} active connection(s) to close...",
            drain_timeout.as_secs(), active);
        while metrics.active_connections.load(Ordering::Relaxed) > 0
            && drain_start.elapsed() < drain_timeout
        {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let remaining = metrics.active_connections.load(Ordering::Relaxed);
        if remaining > 0 {
            warn!("{remaining} connection(s) still active after drain timeout");
        }
    }

    info!("Saving all connected players...");
    let mut saved_count = 0u32;
    for scene_ref in world.scenes.iter() {
        let scene = scene_ref.value();
        for player_ref in scene.players.iter() {
            let player = player_ref.value();
            let char_id = &player.character_id;
            let bank_gold = world.db.get_bank_gold(char_id).await.unwrap_or(0);
            let _ = world.db.save_character_state(
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
            saved_count += 1;
        }
    }

    let uptime = metrics.uptime_secs();
    let total_conns = metrics.total_connections.load(Ordering::Relaxed);
    info!("Saved {saved_count} player(s).");
    info!("Server stats: uptime={uptime}s, total_connections={total_conns}");
    info!("Goodbye!");

    Ok(())
}
