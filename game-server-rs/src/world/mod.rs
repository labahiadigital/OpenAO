use std::sync::{Arc, Mutex, RwLock};

use dashmap::DashMap;
use elura::gameplay::aoi::{AoiConfig, AoiGrid, Point2};
use openao_protocol::constants::{
    CLIENT_VIEW_RANGE_X, CLIENT_VIEW_RANGE_Y, MAP_MAX_COORDINATE,
};
use tokio::sync::{broadcast, mpsc};

use crate::game_data::GameData;
use crate::gameplay::entity_replication::ObserverReplicator;
use crate::gameplay::input_queue::PlayerInputReceiver;
use crate::gameplay::netcode::SceneLagHistory;
use crate::persistence::Database;

pub type MapId = i32;
pub type EntityId = u32;

/// Cell size for the AOI grid. Entities within CLIENT_VIEW_RANGE tiles
/// of each other share visibility. Using a cell size that matches the
/// view range keeps queries efficient (typically 1-4 cells checked).
const AOI_CELL_SIZE: f64 = CLIENT_VIEW_RANGE_X as f64;

#[derive(Debug, Clone)]
pub struct Position {
    pub map: MapId,
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn to_point2(&self) -> Point2 {
        Point2::new(self.x as f64, self.y as f64)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlayerState {
    pub id: EntityId,
    pub account_id: String,
    pub character_id: String,
    pub name: String,
    pub client_ip: String,
    pub pos: Position,
    pub heading: u8,
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub level: i32,
    pub exp: i32,
    pub exp_next_level: i32,
    pub gold: i32,
    pub dead: bool,
    pub criminal: bool,
    pub faction: String,
    pub faction_rank: i32,
    pub faction_score: i32,
    pub faction_rank_armada: i32,
    pub faction_score_armada: i32,
    pub faction_rank_caos: i32,
    pub faction_score_caos: i32,
    pub min_hit: i32,
    pub max_hit: i32,
    pub id_clase: i32,
    pub id_raza: i32,
    pub attr_fuerza: i32,
    pub attr_agilidad: i32,
    pub attr_inteligencia: i32,
    pub attr_constitucion: i32,
    pub id_head: i32,
    pub id_body: i32,
    pub id_helmet: i32,
    pub id_weapon: i32,
    pub id_shield: i32,
    pub id_arrow_slot: i32,
    pub id_ring_slot: i32,
    pub navegando: bool,
    pub party_id: Option<String>,
    pub clan_id: Option<String>,
    pub home_map: i32,
    pub home_x: i32,
    pub home_y: i32,
    pub pvp_block_until_ms: u64,
    pub revive_at_ms: u64,
    pub fishing: Option<FishingState>,
    pub harvesting: Option<HarvestingState>,
    pub buffs: crate::gameplay::buffs::BuffManager,
    pub invisible: bool,
    pub hidden_skill: bool,
    pub hidden_skill_expire_tick: u64,
    pub hidden_skill_cooldown_tick: u64,
    pub jail_until_ms: u64,
    pub quest_log: crate::gameplay::quests::PlayerQuestLog,
    pub pets: crate::gameplay::pets::PetManager,
    pub spell_cooldowns: crate::gameplay::cooldowns::CooldownManager,
    pub achievements: crate::gameplay::achievements::AchievementTracker,
    pub paralizado: bool,
    pub paralizado_until_ms: u64,
    pub inmovilizado: bool,
    pub inmovilizado_until_ms: u64,
    pub invisible_spell: bool,
    pub invisible_spell_until_ms: u64,
    pub seguro_activado: bool,
    pub seguro_clan_activado: bool,
    pub dead_world_active: bool,
    pub dead_world_transition_at_ms: u64,
    pub logout_expires_at_ms: u64,
    pub logout_origin_x: i32,
    pub logout_origin_y: i32,
    pub criminales_matados: i32,
    pub ciudadanos_matados: i32,
    pub meditar: bool,
    pub action_cooldowns: ActionCooldowns,
    /// Entity IDs of NPCs summoned by this player.
    pub summons: Vec<EntityId>,
}

/// Per-action combat cooldowns ported from vars.timing.actionCooldowns (Node.js).
/// All values in milliseconds from epoch (world.uptime_ms()).
#[derive(Debug, Clone, Default)]
pub struct ActionCooldowns {
    pub next_melee_at: u64,
    pub next_range_at: u64,
    pub next_spell_at: u64,
    pub next_use_item_at: u64,
    pub next_spell_after_melee_at: u64,
    pub next_melee_after_spell_at: u64,
    pub next_use_item_after_melee_at: u64,
    pub next_dialog_at: u64,
    pub next_drop_item_at: u64,
    pub next_equip_toggle_at: u64,
    pub next_click_at: u64,
}

impl ActionCooldowns {
    pub const MELEE_MS: u64 = 950;
    pub const RANGE_MS: u64 = 950;
    pub const SPELL_MS: u64 = 850;
    pub const USE_ITEM_MS: u64 = 250;
    pub const MELEE_TO_SPELL_MS: u64 = 800;
    pub const SPELL_TO_MELEE_MS: u64 = 800;
    pub const MELEE_TO_USE_ITEM_MS: u64 = 550;
    pub const DIALOG_MS: u64 = 500;
    pub const DROP_ITEM_MS: u64 = 150;
    pub const EQUIP_TOGGLE_MS: u64 = 125;
    pub const CLICK_MS: u64 = 150;

    pub fn can_melee(&self, now: u64) -> bool {
        now >= self.next_melee_at && now >= self.next_melee_after_spell_at
    }
    pub fn trigger_melee(&mut self, now: u64) {
        self.next_melee_at = now + Self::MELEE_MS;
        self.next_spell_after_melee_at = now + Self::MELEE_TO_SPELL_MS;
        self.next_use_item_after_melee_at = now + Self::MELEE_TO_USE_ITEM_MS;
    }
    pub fn can_range(&self, now: u64) -> bool {
        now >= self.next_range_at && now >= self.next_melee_after_spell_at
    }
    pub fn trigger_range(&mut self, now: u64) {
        self.next_range_at = now + Self::RANGE_MS;
        self.next_spell_after_melee_at = now + Self::MELEE_TO_SPELL_MS;
    }
    pub fn can_spell(&self, now: u64) -> bool {
        now >= self.next_spell_at && now >= self.next_spell_after_melee_at
    }
    pub fn trigger_spell(&mut self, now: u64) {
        self.next_spell_at = now + Self::SPELL_MS;
        self.next_melee_after_spell_at = now + Self::SPELL_TO_MELEE_MS;
    }
    pub fn can_use_item(&self, now: u64) -> bool {
        now >= self.next_use_item_at && now >= self.next_use_item_after_melee_at
    }
    pub fn trigger_use_item(&mut self, now: u64) {
        self.next_use_item_at = now + Self::USE_ITEM_MS;
    }
    pub fn can_dialog(&self, now: u64) -> bool {
        now >= self.next_dialog_at
    }
    pub fn trigger_dialog(&mut self, now: u64) {
        self.next_dialog_at = now + Self::DIALOG_MS;
    }
    pub fn can_drop_item(&self, now: u64) -> bool {
        now >= self.next_drop_item_at
    }
    pub fn trigger_drop_item(&mut self, now: u64) {
        self.next_drop_item_at = now + Self::DROP_ITEM_MS;
    }
    pub fn can_equip_toggle(&self, now: u64) -> bool {
        now >= self.next_equip_toggle_at
    }
    pub fn trigger_equip_toggle(&mut self, now: u64) {
        self.next_equip_toggle_at = now + Self::EQUIP_TOGGLE_MS;
    }
    pub fn can_click(&self, now: u64) -> bool {
        now >= self.next_click_at
    }
    pub fn trigger_click(&mut self, now: u64) {
        self.next_click_at = now + Self::CLICK_MS;
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FishingState {
    pub active: bool,
    pub pending_target: bool,
    pub slot: u8,
    pub item_id: i32,
    pub power: i32,
    pub target_x: i32,
    pub target_y: i32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub next_tick_at_ms: u64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HarvestingState {
    pub active: bool,
    pub pending_target: bool,
    pub skill: HarvestingSkill,
    pub slot: u8,
    pub item_id: i32,
    pub target_x: i32,
    pub target_y: i32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub next_tick_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HarvestingSkill {
    Woodcutting,
    Mining,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Clan {
    pub id: String,
    pub name: String,
    pub leader_id: EntityId,
    pub leader_name: String,
    pub member_ids: Vec<EntityId>,
    pub co_leader_ids: Vec<EntityId>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClanRequest {
    pub id: String,
    pub applicant_id: EntityId,
    pub applicant_name: String,
    pub clan_id: String,
    pub message: String,
}

const PARTY_MAX_MEMBERS: usize = 4;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Party {
    pub id: String,
    pub leader_id: EntityId,
    pub member_ids: Vec<EntityId>,
}

impl Party {
    pub fn is_full(&self) -> bool {
        self.member_ids.len() >= PARTY_MAX_MEMBERS
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NpcState {
    pub id: EntityId,
    pub npc_type: i32,
    pub pos: Position,
    pub heading: u8,
    pub hp: i32,
    pub max_hp: i32,
    pub min_hit: i32,
    pub max_hit: i32,
    pub defense: i32,
    pub exp_reward: i32,
    pub movement: i32,
    pub dead: bool,
    pub paralizado: bool,
    pub inmovilizado: bool,
    pub cc_expire_tick: u64,
    pub aggro_target: Option<EntityId>,
    /// NPC spells (spell_id list from NPC data). Empty for melee-only NPCs.
    pub spells: Vec<NpcSpellSlot>,
    /// Minimum interval between spell casts (ms from uptime).
    pub spell_cast_interval_ms: u64,
    /// Last time this NPC cast a spell (uptime ms).
    pub last_spell_cast_at: u64,
    /// Maximum spell range in tiles (Manhattan distance).
    pub spell_range: i32,
    /// Magic defense for magic resistance calculation.
    pub magic_def: i32,
    /// Magic resistance stat for spell damage reduction.
    pub magic_resistance: i32,
    /// If this NPC was summoned by a player, their entity ID.
    pub summoned_by: Option<EntityId>,
    /// Tick at which this summon expires (0 = no expiry / not summoned).
    pub summon_expires_at_ms: u64,
    /// If spawned via `/bot`, the admin owner's entity ID. Auto-heals in game loop.
    pub admin_bot_owner: Option<EntityId>,
}

#[derive(Debug, Clone)]
pub struct NpcSpellSlot {
    pub spell_id: i32,
}

#[derive(Debug, Clone)]
pub struct GroundItem {
    pub x: i32,
    pub y: i32,
    pub item_id: i32,
    pub amount: i32,
    pub grh_index: u16,
    pub dropped_at_ms: u64,
}

#[derive(Debug, Clone)]
pub struct BroadcastPacket {
    pub sender_entity_id: EntityId,
    pub data: Vec<u8>,
}

pub struct Scene {
    pub map_id: MapId,
    pub players: DashMap<EntityId, PlayerState>,
    pub npcs: DashMap<EntityId, NpcState>,
    pub ground_items: DashMap<(i32, i32), GroundItem>,
    #[allow(dead_code)]
    pub blocked: RwLock<Vec<Vec<bool>>>,
    pub broadcast_tx: broadcast::Sender<BroadcastPacket>,
    pub personal_tx: DashMap<EntityId, mpsc::UnboundedSender<Vec<u8>>>,
    pub aoi: Mutex<AoiGrid<EntityId>>,
    pub lag_history: Mutex<SceneLagHistory>,
    pub replicators: DashMap<EntityId, Mutex<ObserverReplicator>>,
    pub input_receivers: DashMap<EntityId, Mutex<PlayerInputReceiver>>,
    /// Per-player outbound queue pressure counters for priority-based packet dropping.
    pub outbound_pressure: DashMap<EntityId, std::sync::atomic::AtomicU32>,
}

impl Scene {
    pub fn new(map_id: MapId) -> Self {
        let size = (MAP_MAX_COORDINATE + 1) as usize;
        let (tx, _) = broadcast::channel(256);

        let mut aoi_config = AoiConfig::default();
        aoi_config.cell_size = AOI_CELL_SIZE;
        aoi_config.max_query_cells = 64;
        let aoi = AoiGrid::new(aoi_config).expect("valid AOI config");

        Self {
            map_id,
            players: DashMap::new(),
            npcs: DashMap::new(),
            ground_items: DashMap::new(),
            blocked: RwLock::new(vec![vec![false; size]; size]),
            broadcast_tx: tx,
            personal_tx: DashMap::new(),
            aoi: Mutex::new(aoi),
            lag_history: Mutex::new(SceneLagHistory::new()),
            replicators: DashMap::new(),
            input_receivers: DashMap::new(),
            outbound_pressure: DashMap::new(),
        }
    }

    pub fn broadcast(&self, sender_entity_id: EntityId, data: Vec<u8>) {
        let _ = self.broadcast_tx.send(BroadcastPacket {
            sender_entity_id,
            data,
        });
    }

    /// Send a packet only to players within AOI range of a given position.
    /// Falls back to full broadcast if AOI query fails.
    ///
    /// Uses `Arc<[u8]>` internally so the payload is ref-counted: each
    /// recipient gets a cheap `Arc::clone` (atomic increment) + a single
    /// `to_vec()` at send time. For broadcasts to many players (e.g. NPC
    /// movement to 20+ observers) this avoids N-1 redundant allocations
    /// of the source buffer.
    pub fn broadcast_in_range(&self, sender_entity_id: EntityId, center: &Position, data: Vec<u8>) {
        let radius = ((CLIENT_VIEW_RANGE_X.max(CLIENT_VIEW_RANGE_Y)) as f64) + 0.5;
        let nearby = if let Ok(grid) = self.aoi.try_lock() {
            grid.query(center.to_point2(), radius).unwrap_or_default()
        } else {
            let _ = self.broadcast_tx.send(BroadcastPacket { sender_entity_id, data });
            return;
        };

        let shared: std::sync::Arc<[u8]> = data.into();
        for eid in nearby {
            if eid != sender_entity_id
                && let Some(tx) = self.personal_tx.get(&eid)
            {
                let _ = tx.send(shared.to_vec());
            }
        }
    }

    pub fn send_to_player(&self, entity_id: EntityId, data: Vec<u8>) {
        if let Some(tx) = self.personal_tx.get(&entity_id) {
            let _ = tx.send(data);
        }
    }

    /// Send a packet to a player, but drop it if the outbound queue is congested
    /// and the packet's priority is below the congestion threshold.
    /// Uses a sliding-window pressure counter that resets periodically in the game loop.
    #[allow(dead_code)]
    pub fn send_to_player_prioritized(
        &self,
        entity_id: EntityId,
        data: Vec<u8>,
        priority: crate::routes::PacketPriority,
    ) {
        if let Some(tx) = self.personal_tx.get(&entity_id) {
            let pressure = self.outbound_pressure
                .get(&entity_id)
                .map(|v| v.load(std::sync::atomic::Ordering::Relaxed))
                .unwrap_or(0);
            let drop = match priority {
                crate::routes::PacketPriority::Low => pressure > 64,
                crate::routes::PacketPriority::Normal => pressure > 128,
                _ => false,
            };
            if !drop {
                if let Some(counter) = self.outbound_pressure.get(&entity_id) {
                    counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                let _ = tx.send(data);
            }
        }
    }

    /// Reset all outbound pressure counters (called periodically from the game loop).
    pub fn reset_outbound_pressure(&self) {
        for entry in self.outbound_pressure.iter() {
            entry.value().store(0, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Insert an entity into the AOI grid.
    pub fn aoi_insert(&self, entity_id: EntityId, pos: &Position) {
        if let Ok(mut grid) = self.aoi.try_lock() {
            let _ = grid.insert(entity_id, pos.to_point2());
        }
    }

    /// Move an entity in the AOI grid.
    pub fn aoi_move(&self, entity_id: EntityId, pos: &Position) {
        if let Ok(mut grid) = self.aoi.try_lock() {
            let _ = grid.move_entity(&entity_id, pos.to_point2());
        }
    }

    /// Remove an entity from the AOI grid.
    pub fn aoi_remove(&self, entity_id: EntityId) {
        if let Ok(mut grid) = self.aoi.try_lock() {
            let _ = grid.remove(&entity_id);
        }
    }

    pub fn add_replicator(&self, entity_id: EntityId) {
        self.replicators.insert(entity_id, Mutex::new(ObserverReplicator::new()));
    }

    pub fn remove_replicator(&self, entity_id: EntityId) {
        self.replicators.remove(&entity_id);
    }

    pub fn add_input_receiver(&self, entity_id: EntityId) {
        self.input_receivers.insert(entity_id, Mutex::new(PlayerInputReceiver::new()));
    }

    pub fn remove_input_receiver(&self, entity_id: EntityId) {
        self.input_receivers.remove(&entity_id);
    }

    /// Mark an entity as broadcast-announced in the replicators of all nearby observers.
    /// This prevents duplicate Spawn events from the replication system.
    pub fn mark_entity_broadcast_announced(&self, entity_id: EntityId, pos: &Position) {
        let nearby = self.entities_in_range(pos);
        for observer_id in nearby {
            if observer_id == entity_id { continue; }
            if let Some(rep) = self.replicators.get(&observer_id)
                && let Ok(mut r) = rep.try_lock() {
                    r.mark_broadcast_announced(entity_id);
                }
        }
    }

    pub fn entities_in_range(&self, center: &Position) -> Vec<EntityId> {
        let radius = ((CLIENT_VIEW_RANGE_X.max(CLIENT_VIEW_RANGE_Y)) as f64) + 0.5;
        if let Ok(grid) = self.aoi.try_lock() {
            grid.query(center.to_point2(), radius).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    #[allow(dead_code)]
    pub fn players_in_range(&self, center: &Position) -> Vec<PlayerState> {
        let entity_ids = self.entities_in_range(center);
        entity_ids.iter()
            .filter_map(|eid| self.players.get(eid).map(|p| p.clone()))
            .collect()
    }

    #[allow(dead_code)]
    pub fn npcs_in_range(&self, center: &Position) -> Vec<NpcState> {
        let entity_ids = self.entities_in_range(center);
        entity_ids.iter()
            .filter_map(|eid| self.npcs.get(eid).map(|n| n.clone()))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PartyInvite {
    pub inviter_id: EntityId,
    pub expires_at: std::time::Instant,
}

pub struct GameWorld {
    pub db: Database,
    pub game_data: std::sync::RwLock<std::sync::Arc<GameData>>,
    pub scenes: DashMap<MapId, Scene>,
    pub parties: DashMap<String, Party>,
    pub party_invites: DashMap<EntityId, PartyInvite>,
    pub clans: DashMap<String, Clan>,
    pub clan_requests: DashMap<String, ClanRequest>,
    pub banned_accounts: DashMap<String, String>,
    pub banned_ips: DashMap<String, String>,
    pub muted_players: DashMap<EntityId, bool>,
    pub double_exp: std::sync::atomic::AtomicBool,
    pub double_gold: std::sync::atomic::AtomicBool,
    pub challenges: std::sync::Mutex<crate::gameplay::rooms::ChallengeRoomManager>,
    /// Per-character inventory cache: loaded once on connect, mutated in-memory,
    /// flushed to DB on disconnect / worldsave / periodic save.
    pub inventory_cache: DashMap<String, Vec<crate::persistence::InventoryRow>>,
    /// Tracks which characters have unsaved inventory changes.
    pub inventory_dirty: DashMap<String, bool>,
    /// Pending P2P trade requests: initiator_entity -> target_entity
    pub trade_requests: DashMap<EntityId, EntityId>,
    /// Active trades: trade_id -> TradeSession
    pub active_trades: DashMap<String, TradeSession>,
    /// Maps entity_id -> trade_id for quick lookup
    pub entity_trade: DashMap<EntityId, String>,
    pub territories: std::sync::Mutex<crate::gameplay::territory::TerritoryManager>,
    pub npc_respawn_cooldowns: DashMap<(i32, i32), u64>,
    /// Working lock: maps IP address to entity_id of the player currently gathering.
    /// Prevents multi-botting (same IP) from gathering simultaneously.
    pub working_lock: DashMap<String, EntityId>,
    /// PvP faction score rekill protection: (attacker_entity, victim_entity) -> last_kill_timestamp_ms.
    /// Prevents farming faction score by killing the same player repeatedly within 5 minutes.
    pub faction_rekill_tracker: DashMap<(EntityId, EntityId), u64>,
    /// Active character sessions: character_id -> (entity_id, map_id, evict_flag).
    /// Used to detect and evict duplicate connections for the same character.
    pub active_characters: DashMap<String, (EntityId, MapId, Arc<std::sync::atomic::AtomicBool>)>,
    /// Runtime-modifiable timing values (via /intervalo command).
    pub runtime_timings: RuntimeTimings,
    next_entity_id: std::sync::atomic::AtomicU32,
    uptime_start: std::time::Instant,
}

pub struct RuntimeTimings {
    pub melee_ms: std::sync::atomic::AtomicU64,
    pub range_ms: std::sync::atomic::AtomicU64,
    pub spell_ms: std::sync::atomic::AtomicU64,
    pub use_item_ms: std::sync::atomic::AtomicU64,
    pub dialog_ms: std::sync::atomic::AtomicU64,
    pub regen_ticks: std::sync::atomic::AtomicU64,
    pub npc_ai_ticks: std::sync::atomic::AtomicU64,
}

impl RuntimeTimings {
    pub fn new() -> Self {
        use std::sync::atomic::AtomicU64;
        Self {
            melee_ms: AtomicU64::new(950),
            range_ms: AtomicU64::new(950),
            spell_ms: AtomicU64::new(850),
            use_item_ms: AtomicU64::new(250),
            dialog_ms: AtomicU64::new(500),
            regen_ticks: AtomicU64::new(60),
            npc_ai_ticks: AtomicU64::new(30),
        }
    }
}

/// Represents one side of a P2P trade.
#[derive(Debug, Clone, Default)]
pub struct TradeOffer {
    #[allow(dead_code)]
    pub items: Vec<(u8, i32, i16)>,
    pub gold: i32,
    pub confirmed: bool,
}

/// Active P2P trade session between two players.
#[derive(Debug, Clone)]
pub struct TradeSession {
    pub player_a: EntityId,
    pub player_b: EntityId,
    pub offer_a: TradeOffer,
    pub offer_b: TradeOffer,
}

impl GameWorld {
    pub fn new(db: Database, game_data: std::sync::Arc<GameData>) -> Self {
        Self {
            db,
            game_data: std::sync::RwLock::new(game_data),
            scenes: DashMap::with_shard_amount(32),
            parties: DashMap::new(),
            party_invites: DashMap::new(),
            clans: DashMap::new(),
            clan_requests: DashMap::new(),
            banned_accounts: DashMap::new(),
            banned_ips: DashMap::new(),
            muted_players: DashMap::new(),
            double_exp: std::sync::atomic::AtomicBool::new(false),
            double_gold: std::sync::atomic::AtomicBool::new(false),
            challenges: std::sync::Mutex::new(crate::gameplay::rooms::ChallengeRoomManager::new()),
            inventory_cache: DashMap::with_shard_amount(32),
            inventory_dirty: DashMap::with_shard_amount(32),
            trade_requests: DashMap::new(),
            active_trades: DashMap::new(),
            entity_trade: DashMap::new(),
            territories: std::sync::Mutex::new(crate::gameplay::territory::TerritoryManager::new()),
            npc_respawn_cooldowns: DashMap::new(),
            working_lock: DashMap::new(),
            faction_rekill_tracker: DashMap::new(),
            active_characters: DashMap::new(),
            runtime_timings: RuntimeTimings::new(),
            next_entity_id: std::sync::atomic::AtomicU32::new(1),
            uptime_start: std::time::Instant::now(),
        }
    }

    pub fn uptime_ms(&self) -> u64 {
        self.uptime_start.elapsed().as_millis() as u64
    }

    pub fn gd(&self) -> std::sync::Arc<GameData> {
        match self.game_data.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                tracing::error!("game_data RwLock poisoned, recovering");
                poisoned.into_inner().clone()
            }
        }
    }

    pub fn reload_game_data(&self) -> Result<(), String> {
        let data_dir = std::path::Path::new("data");
        match GameData::load(data_dir) {
            Ok(new_data) => {
                let mut lock = match self.game_data.write() {
                    Ok(l) => l,
                    Err(poisoned) => {
                        tracing::error!("game_data RwLock poisoned during reload, recovering");
                        poisoned.into_inner()
                    }
                };
                *lock = std::sync::Arc::new(new_data);
                Ok(())
            }
            Err(e) => Err(format!("Error al recargar game data: {}", e)),
        }
    }

    pub fn has_active_gathering_on_ip(&self, ip: &str, exclude_entity: EntityId) -> bool {
        if let Some(locked_eid) = self.working_lock.get(ip) {
            if *locked_eid != exclude_entity {
                return true;
            }
        }
        false
    }

    pub fn acquire_working_lock(&self, ip: &str, entity_id: EntityId) {
        self.working_lock.insert(ip.to_string(), entity_id);
    }

    pub fn release_working_lock(&self, ip: &str, entity_id: EntityId) {
        if let Some(entry) = self.working_lock.get(ip) {
            if *entry == entity_id {
                drop(entry);
                self.working_lock.remove(ip);
            }
        }
    }

    pub fn get_duplicate_account_penalized_entities(&self) -> std::collections::HashSet<EntityId> {
        let mut by_account: std::collections::HashMap<String, Vec<(EntityId, bool)>> = std::collections::HashMap::new();

        for scene_ref in self.scenes.iter() {
            let scene = scene_ref.value();
            for player_ref in scene.players.iter() {
                let p = player_ref.value();
                let acct = p.account_id.trim().to_string();
                if acct.is_empty() {
                    continue;
                }
                let is_gathering = p.fishing.as_ref().is_some_and(|f| f.active)
                    || p.harvesting.is_some();
                by_account.entry(acct).or_default().push((p.id, is_gathering));
            }
        }

        let mut penalized = std::collections::HashSet::new();
        for entries in by_account.values() {
            if entries.len() < 2 || !entries.iter().any(|(_, g)| *g) {
                continue;
            }
            for (eid, gathering) in entries {
                if !*gathering {
                    penalized.insert(*eid);
                }
            }
        }
        penalized
    }

    pub fn next_id(&self) -> EntityId {
        loop {
            let id = self.next_entity_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if id == 0 {
                continue;
            }
            return id;
        }
    }

    pub fn get_or_create_scene(&self, map_id: MapId) -> dashmap::mapref::one::Ref<'_, MapId, Scene> {
        if !self.scenes.contains_key(&map_id) {
            let scene = Scene::new(map_id);
            self.spawn_npcs_from_data(map_id, &scene);
            self.scenes.entry(map_id).or_insert(scene);
        }
        self.scenes.get(&map_id).expect("scene must exist after or_insert")
    }

    fn spawn_npcs_from_data(&self, map_id: MapId, scene: &Scene) {
        let gd = self.gd();
        let Some(spawns) = gd.get_map_spawns(map_id) else {
            return;
        };

        for spawn in spawns {
            let npc_index = spawn.npc_index;
            let template = match gd.get_npc(npc_index) {
                Some(t) => t,
                None => {
                    tracing::warn!("NPC template {} not found for map {}", npc_index, map_id);
                    continue;
                }
            };

            let id = self.next_id();
            let pos = Position { map: map_id, x: spawn.x, y: spawn.y };
            scene.aoi_insert(id, &pos);

            let movement = spawn.movement.unwrap_or(template.movement);

            let npc_spells: Vec<NpcSpellSlot> = template.spells.iter()
                .map(|s| NpcSpellSlot { spell_id: s.id_spell })
                .collect();
            scene.npcs.insert(id, NpcState {
                id,
                npc_type: npc_index,
                pos,
                heading: 3,
                hp: template.max_hp,
                max_hp: template.max_hp,
                min_hit: template.min_hit,
                max_hit: template.max_hit,
                defense: template.def,
                exp_reward: template.exp,
                movement,
                dead: template.max_hp <= 0,
                paralizado: false,
                inmovilizado: false,
                cc_expire_tick: 0,
                aggro_target: None,
                spells: npc_spells,
                spell_cast_interval_ms: template.spell_cast_interval_ms.unwrap_or(2000),
                last_spell_cast_at: 0,
                spell_range: template.spell_range.unwrap_or(8),
                magic_def: template.magic_def,
                magic_resistance: template.magic_resistance,
                summoned_by: None,
                summon_expires_at_ms: 0,
                admin_bot_owner: None,
            });
        }

        let npc_count = scene.npcs.len();
        if npc_count > 0 {
            tracing::debug!("Map {} spawned {} NPCs", map_id, npc_count);
        }
    }

    // --- Inventory cache helpers ---

    /// Load inventory into cache (called on character connect).
    pub async fn cache_load_inventory(&self, character_id: &str) -> Result<(), sqlx::Error> {
        let rows = self.db.load_inventory(character_id).await?;
        self.inventory_cache.insert(character_id.to_string(), rows);
        self.inventory_dirty.remove(character_id);
        Ok(())
    }

    /// Get a snapshot of the cached inventory for a character.
    pub fn cache_get_inventory(&self, character_id: &str) -> Vec<crate::persistence::InventoryRow> {
        self.inventory_cache
            .get(character_id)
            .map(|v| v.value().clone())
            .unwrap_or_default()
    }

    /// Update or insert a slot in the cached inventory.
    pub fn cache_update_slot(
        &self,
        character_id: &str,
        slot: i32,
        item_id: i32,
        amount: i32,
        equipped: bool,
    ) {
        let mut inv = self.inventory_cache.entry(character_id.to_string()).or_default();
        if let Some(row) = inv.iter_mut().find(|r| r.slot == slot) {
            row.item_id = item_id;
            row.amount = amount;
            row.equipped = equipped;
        } else {
            inv.push(crate::persistence::InventoryRow { slot, item_id, amount, equipped });
        }
        self.inventory_dirty.insert(character_id.to_string(), true);
    }

    /// Remove a slot from the cached inventory.
    pub fn cache_delete_slot(&self, character_id: &str, slot: i32) {
        if let Some(mut inv) = self.inventory_cache.get_mut(character_id) {
            inv.retain(|r| r.slot != slot);
        }
        self.inventory_dirty.insert(character_id.to_string(), true);
    }

    /// Flush dirty inventory to DB. Called on disconnect, worldsave, shutdown.
    pub async fn cache_flush_inventory(&self, character_id: &str) {
        let is_dirty = self.inventory_dirty.get(character_id).map(|v| *v).unwrap_or(false);
        if !is_dirty {
            return;
        }
        let inv = self.cache_get_inventory(character_id);
        if let Err(e) = self.db.save_full_inventory(character_id, &inv).await {
            tracing::error!("Failed to flush inventory for {}: {}", character_id, e);
        }
        self.inventory_dirty.insert(character_id.to_string(), false);
    }

    /// Remove character from cache entirely (called after flush on disconnect).
    pub fn cache_remove(&self, character_id: &str) {
        self.inventory_cache.remove(character_id);
        self.inventory_dirty.remove(character_id);
    }

    /// Find the first empty slot (0..19) in cached inventory, or None if full.
    pub fn cache_find_empty_slot(&self, character_id: &str) -> Option<i32> {
        let inv = self.cache_get_inventory(character_id);
        let used: std::collections::HashSet<i32> = inv.iter().map(|r| r.slot).collect();
        (0..20).find(|s| !used.contains(s))
    }

    /// Find a slot with matching item_id (for stacking).
    pub fn cache_find_item_slot(&self, character_id: &str, item_id: i32) -> Option<i32> {
        let inv = self.cache_get_inventory(character_id);
        inv.iter().find(|r| r.item_id == item_id).map(|r| r.slot)
    }

    /// Add an item to inventory cache (stacks if possible, else finds empty slot).
    pub fn cache_add_item(&self, character_id: &str, item_id: i32, amount: i32) -> bool {
        if let Some(slot) = self.cache_find_item_slot(character_id, item_id) {
            let inv = self.cache_get_inventory(character_id);
            let current = inv.iter().find(|r| r.slot == slot).map(|r| r.amount).unwrap_or(0);
            let equipped = inv.iter().find(|r| r.slot == slot).map(|r| r.equipped).unwrap_or(false);
            self.cache_update_slot(character_id, slot, item_id, current + amount, equipped);
            true
        } else if let Some(slot) = self.cache_find_empty_slot(character_id) {
            self.cache_update_slot(character_id, slot, item_id, amount, false);
            true
        } else {
            false
        }
    }

    /// Remove items from inventory cache (across multiple stacks if needed).
    pub fn cache_remove_items(&self, character_id: &str, item_id: i32, mut amount_to_remove: i32) {
        let inv = self.cache_get_inventory(character_id);
        for row in inv {
            if row.item_id == item_id && amount_to_remove > 0 {
                let remove = amount_to_remove.min(row.amount);
                let remaining = row.amount - remove;
                if remaining <= 0 {
                    self.cache_delete_slot(character_id, row.slot);
                } else {
                    self.cache_update_slot(character_id, row.slot, row.item_id, remaining, row.equipped);
                }
                amount_to_remove -= remove;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_cooldowns_melee_blocks_until_ready() {
        let mut cd = ActionCooldowns::default();
        assert!(cd.can_melee(0));
        cd.trigger_melee(1000);
        assert!(!cd.can_melee(1500));
        assert!(cd.can_melee(1000 + ActionCooldowns::MELEE_MS));
    }

    #[test]
    fn action_cooldowns_cross_gate_melee_to_spell() {
        let mut cd = ActionCooldowns::default();
        cd.trigger_melee(1000);
        assert!(!cd.can_spell(1000 + 100));
        assert!(cd.can_spell(1000 + ActionCooldowns::MELEE_TO_SPELL_MS));
    }

    #[test]
    fn action_cooldowns_cross_gate_spell_to_melee() {
        let mut cd = ActionCooldowns::default();
        cd.trigger_spell(1000);
        assert!(!cd.can_melee(1000 + 100));
        assert!(cd.can_melee(1000 + ActionCooldowns::SPELL_TO_MELEE_MS));
    }

    #[test]
    fn action_cooldowns_use_item_after_melee() {
        let mut cd = ActionCooldowns::default();
        cd.trigger_melee(1000);
        assert!(!cd.can_use_item(1000 + 100));
        assert!(cd.can_use_item(1000 + ActionCooldowns::MELEE_TO_USE_ITEM_MS));
    }

    #[test]
    fn action_cooldowns_constants_match_original() {
        assert_eq!(ActionCooldowns::MELEE_MS, 950);
        assert_eq!(ActionCooldowns::RANGE_MS, 950);
        assert_eq!(ActionCooldowns::SPELL_MS, 850);
        assert_eq!(ActionCooldowns::USE_ITEM_MS, 250);
        assert_eq!(ActionCooldowns::DIALOG_MS, 500);
        assert_eq!(ActionCooldowns::DROP_ITEM_MS, 150);
        assert_eq!(ActionCooldowns::EQUIP_TOGGLE_MS, 125);
        assert_eq!(ActionCooldowns::CLICK_MS, 150);
    }

    #[test]
    fn runtime_timings_defaults() {
        let t = RuntimeTimings::new();
        assert_eq!(t.melee_ms.load(std::sync::atomic::Ordering::Relaxed), 950);
        assert_eq!(t.regen_ticks.load(std::sync::atomic::Ordering::Relaxed), 60);
        assert_eq!(t.npc_ai_ticks.load(std::sync::atomic::Ordering::Relaxed), 30);
    }
}
