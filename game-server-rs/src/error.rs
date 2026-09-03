use std::fmt;

/// Structured error codes for game server operations.
/// These provide machine-readable error identification while
/// keeping human-readable messages for the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum GameErrorCode {
    // Auth & Session (1xx)
    AuthRequired,
    InvalidTicket,
    SessionExpired,
    AlreadyConnected,
    Banned,

    // Movement & Position (2xx)
    InvalidPosition,
    TileBlocked,
    MapNotFound,
    PvpMapChangeBlocked,

    // Combat (3xx)
    TargetNotFound,
    TargetOutOfRange,
    TargetDead,
    AlreadyDead,
    SafeZoneBlocked,
    InsufficientMana,
    SpellOnCooldown,

    // Inventory (4xx)
    InventoryFull,
    ItemNotFound,
    InsufficientItems,
    InsufficientGold,
    InvalidSlot,
    ItemNotEquippable,

    // Social (5xx)
    PlayerNotFound,
    PlayerOffline,
    PartyFull,
    AlreadyInParty,
    NotPartyLeader,
    ClanFull,
    AlreadyInClan,
    NotClanLeader,

    // General (9xx)
    RateLimited,
    InternalError,
    NotImplemented,
}

impl GameErrorCode {
    pub fn code(&self) -> u16 {
        match self {
            Self::AuthRequired => 100,
            Self::InvalidTicket => 101,
            Self::SessionExpired => 102,
            Self::AlreadyConnected => 103,
            Self::Banned => 104,

            Self::InvalidPosition => 200,
            Self::TileBlocked => 201,
            Self::MapNotFound => 202,
            Self::PvpMapChangeBlocked => 203,

            Self::TargetNotFound => 300,
            Self::TargetOutOfRange => 301,
            Self::TargetDead => 302,
            Self::AlreadyDead => 303,
            Self::SafeZoneBlocked => 304,
            Self::InsufficientMana => 305,
            Self::SpellOnCooldown => 306,

            Self::InventoryFull => 400,
            Self::ItemNotFound => 401,
            Self::InsufficientItems => 402,
            Self::InsufficientGold => 403,
            Self::InvalidSlot => 404,
            Self::ItemNotEquippable => 405,

            Self::PlayerNotFound => 500,
            Self::PlayerOffline => 501,
            Self::PartyFull => 502,
            Self::AlreadyInParty => 503,
            Self::NotPartyLeader => 504,
            Self::ClanFull => 505,
            Self::AlreadyInClan => 506,
            Self::NotClanLeader => 507,

            Self::RateLimited => 900,
            Self::InternalError => 901,
            Self::NotImplemented => 902,
        }
    }
}

/// A structured game error with code, user-facing message, and optional details.
#[derive(Debug, Clone)]
pub struct GameError {
    pub code: GameErrorCode,
    pub message: String,
}

#[allow(dead_code)]
impl GameError {
    pub fn new(code: GameErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn auth_required() -> Self {
        Self::new(GameErrorCode::AuthRequired, "Debes autenticarte primero.")
    }

    pub fn rate_limited() -> Self {
        Self::new(GameErrorCode::RateLimited, "Demasiados paquetes. Intenta de nuevo.")
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        Self::new(GameErrorCode::InternalError, detail)
    }

    pub fn insufficient_gold(needed: i32, have: i32) -> Self {
        Self::new(
            GameErrorCode::InsufficientGold,
            format!("Necesitas {} oro (tienes {}).", needed, have),
        )
    }

    pub fn player_not_found(name: &str) -> Self {
        Self::new(
            GameErrorCode::PlayerNotFound,
            format!("{} no está online.", name),
        )
    }

    pub fn inventory_full() -> Self {
        Self::new(GameErrorCode::InventoryFull, "No tienes espacio en el inventario.")
    }

    pub fn not_in_party() -> Self {
        Self::new(GameErrorCode::AlreadyInParty, "No estás en una party.")
    }

    pub fn party_full() -> Self {
        Self::new(GameErrorCode::PartyFull, "La party ya alcanzó el máximo de 4 miembros.")
    }

    pub fn not_party_leader() -> Self {
        Self::new(GameErrorCode::NotPartyLeader, "Solo el líder de la party puede realizar esta acción.")
    }

    pub fn not_in_clan() -> Self {
        Self::new(GameErrorCode::AlreadyInClan, "No estás en un clan.")
    }

    pub fn not_clan_leader() -> Self {
        Self::new(GameErrorCode::NotClanLeader, "Solo el líder del clan puede realizar esta acción.")
    }

    pub fn clan_full() -> Self {
        Self::new(GameErrorCode::ClanFull, "El clan ya alcanzó el máximo de miembros.")
    }

    pub fn item_not_found(name: &str) -> Self {
        Self::new(GameErrorCode::ItemNotFound, format!("{} no encontrado.", name))
    }

    pub fn insufficient_items(need: i32, item: &str) -> Self {
        Self::new(
            GameErrorCode::InsufficientItems,
            format!("Necesitas {} {} para esta acción.", need, item),
        )
    }

    pub fn target_not_found(msg: impl Into<String>) -> Self {
        Self::new(GameErrorCode::TargetNotFound, msg)
    }

    pub fn invalid_slot() -> Self {
        Self::new(GameErrorCode::InvalidSlot, "Slot inválido.")
    }
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[E{}] {}", self.code.code(), self.message)
    }
}

impl std::error::Error for GameError {}

impl GameError {
    pub fn to_console_packet(&self) -> Vec<u8> {
        crate::gateway::packets::build_console_message(&self.message)
    }
}

/// Unified handler error type that avoids `Box<dyn Error>` dynamic dispatch.
#[derive(Debug)]
pub enum HandlerError {
    Game(GameError),
    Db(sqlx::Error),
    Ws(tokio_tungstenite::tungstenite::Error),
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Game(e) => write!(f, "{}", e),
            Self::Db(e) => write!(f, "DB: {}", e),
            Self::Ws(e) => write!(f, "WS: {}", e),
            Self::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for HandlerError {}

impl From<GameError> for HandlerError {
    fn from(e: GameError) -> Self { Self::Game(e) }
}

impl From<sqlx::Error> for HandlerError {
    fn from(e: sqlx::Error) -> Self { Self::Db(e) }
}

impl From<tokio_tungstenite::tungstenite::Error> for HandlerError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self { Self::Ws(e) }
}

impl From<openao_protocol::reader::ReadError> for HandlerError {
    fn from(e: openao_protocol::reader::ReadError) -> Self { Self::Other(Box::new(e)) }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for HandlerError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self { Self::Other(e) }
}

/// Type alias for all gateway handler return types.
pub type HandlerResult = Result<(), HandlerError>;
