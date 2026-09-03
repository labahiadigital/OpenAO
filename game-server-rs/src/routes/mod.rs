//! Typed route system for decomposing the game packet dispatch.
//!
//! Each packet type maps to a unique `RouteId`. The `PacketRouter` holds
//! metadata about every route (name, category) and is used by the gateway
//! to log, meter, and organize packet handling.
//!
//! This mirrors Elura's `WorldRegistrar` / `Route` pattern while keeping
//! full backward compatibility with the existing `handle_legacy_binary` dispatch.

use std::collections::HashMap;

use openao_protocol::opcodes::server_packet_id;

/// Logical category for a route, used for logging and metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum RouteCategory {
    Auth,
    Movement,
    Combat,
    Dialog,
    Inventory,
    Commerce,
    Social,
    Crafting,
    Gathering,
    Bank,
    Market,
    Challenge,
    Admin,
    System,
}

/// Packet priority for congestion management.
/// Under load, low-priority packets are dropped first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum PacketPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl RouteCategory {
    #[allow(dead_code)]
    pub fn priority(&self) -> PacketPriority {
        match self {
            RouteCategory::Combat => PacketPriority::Critical,
            RouteCategory::Auth => PacketPriority::Critical,
            RouteCategory::Movement => PacketPriority::High,
            RouteCategory::Inventory => PacketPriority::High,
            RouteCategory::Dialog => PacketPriority::Normal,
            RouteCategory::Commerce => PacketPriority::Normal,
            RouteCategory::Bank => PacketPriority::Normal,
            RouteCategory::Market => PacketPriority::Normal,
            RouteCategory::Social => PacketPriority::Normal,
            RouteCategory::Challenge => PacketPriority::Normal,
            RouteCategory::Crafting => PacketPriority::Normal,
            RouteCategory::Gathering => PacketPriority::Normal,
            RouteCategory::Admin => PacketPriority::High,
            RouteCategory::System => PacketPriority::Low,
        }
    }
}

/// Metadata about a registered game route.
#[derive(Debug, Clone)]
pub struct RouteInfo {
    #[allow(dead_code)]
    pub id: u8,
    pub name: &'static str,
    pub category: RouteCategory,
    #[allow(dead_code)]
    pub priority: PacketPriority,
    /// Whether this route requires the player to have a connected character.
    /// If true, packets will be rejected at dispatch level when `entity_id` is None.
    pub requires_character: bool,
}

/// Registry of all known game routes with O(1) lookup by packet ID.
pub struct PacketRouter {
    index: HashMap<u8, RouteInfo>,
}

impl PacketRouter {
    pub fn new() -> Self {
        let mut router = Self::empty();
        router.register_all();
        router
    }

    /// Create an empty router for modular registration.
    pub fn empty() -> Self {
        Self { index: HashMap::with_capacity(40) }
    }

    fn register(&mut self, id: u8, name: &'static str, category: RouteCategory) {
        let requires_character = !matches!(
            id,
            server_packet_id::CONNECT_CHARACTER | server_packet_id::PING
        );
        let priority = category.priority();
        self.index.insert(id, RouteInfo { id, name, category, priority, requires_character });
    }

    /// Public registration for use by GameModule implementations.
    pub fn register_route(&mut self, id: u8, name: &'static str, category: RouteCategory) {
        self.register(id, name, category);
    }

    /// Number of registered routes.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Look up a route by its packet ID (O(1)).
    pub fn get(&self, packet_id: u8) -> Option<&RouteInfo> {
        self.index.get(&packet_id)
    }

    /// Returns all registered routes.
    #[allow(dead_code)]
    pub fn all(&self) -> Vec<&RouteInfo> {
        self.index.values().collect()
    }

    /// Returns all routes in a given category.
    #[allow(dead_code)]
    pub fn by_category(&self, cat: RouteCategory) -> Vec<&RouteInfo> {
        self.index.values().filter(|r| r.category == cat).collect()
    }

    fn register_all(&mut self) {
        // Auth / Connection
        self.register(server_packet_id::CONNECT_CHARACTER, "ConnectCharacter", RouteCategory::Auth);
        self.register(server_packet_id::PING, "Ping", RouteCategory::System);

        // Movement
        self.register(server_packet_id::POSITION, "Position", RouteCategory::Movement);
        self.register(server_packet_id::CHANGE_HEADING, "ChangeHeading", RouteCategory::Movement);
        self.register(server_packet_id::RESYNC_POSITION, "ResyncPosition", RouteCategory::Movement);

        // Combat
        self.register(server_packet_id::ATTACK_MELE, "AttackMelee", RouteCategory::Combat);
        self.register(server_packet_id::ATTACK_RANGE, "AttackRange", RouteCategory::Combat);
        self.register(server_packet_id::ATTACK_SPELL, "AttackSpell", RouteCategory::Combat);

        // Dialog
        self.register(server_packet_id::DIALOG, "Dialog", RouteCategory::Dialog);

        // Click
        self.register(server_packet_id::CLICK, "Click", RouteCategory::System);

        // Inventory
        self.register(server_packet_id::USE_ITEM_CLICK, "UseItemClick", RouteCategory::Inventory);
        self.register(server_packet_id::EQUIPAR_ITEM, "EquipItem", RouteCategory::Inventory);
        self.register(server_packet_id::TIRAR_ITEM, "DropItem", RouteCategory::Inventory);
        self.register(server_packet_id::AGARRAR_ITEM, "PickupItem", RouteCategory::Inventory);
        self.register(server_packet_id::USE_ITEM_U, "UseItemU", RouteCategory::Inventory);
        self.register(server_packet_id::REORDER_INVENTORY_ITEM, "ReorderInventory", RouteCategory::Inventory);

        // Commerce
        self.register(server_packet_id::BUY_ITEM, "BuyItem", RouteCategory::Commerce);
        self.register(server_packet_id::SELL_ITEM, "SellItem", RouteCategory::Commerce);
        self.register(server_packet_id::CLOSE_TRADE, "CloseTrade", RouteCategory::Commerce);

        // Crafting
        self.register(server_packet_id::CRAFT_ITEM, "CraftItem", RouteCategory::Crafting);

        // Spells
        self.register(server_packet_id::REORDER_SPELL, "ReorderSpell", RouteCategory::Inventory);

        // Toggles
        self.register(server_packet_id::CHANGE_SEGURO, "ToggleSafe", RouteCategory::System);
        self.register(server_packet_id::TOGGLE_HIDDEN_SKILL, "ToggleHidden", RouteCategory::System);
        self.register(server_packet_id::CHANGE_CLAN_SEGURO, "ToggleClanSafe", RouteCategory::System);

        // Bank
        self.register(server_packet_id::CHANGE_BANK_TAB, "ChangeBankTab", RouteCategory::Bank);
        self.register(server_packet_id::DEPOSIT_BANK_GOLD, "DepositBankGold", RouteCategory::Bank);
        self.register(server_packet_id::WITHDRAW_BANK_GOLD, "WithdrawBankGold", RouteCategory::Bank);
        self.register(server_packet_id::REORDER_BANK_ITEM, "ReorderBankItem", RouteCategory::Bank);

        // Market
        self.register(server_packet_id::MARKET_ACTION, "MarketAction", RouteCategory::Market);

        // Challenges
        self.register(server_packet_id::RETOS_ACTION, "RetosAction", RouteCategory::Challenge);
    }
}

impl Default for PacketRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_registers_all_routes() {
        let router = PacketRouter::new();
        assert!(router.index.len() > 20, "Expected at least 20 routes");
    }

    #[test]
    fn router_finds_connect_route() {
        let router = PacketRouter::new();
        let route = router.get(server_packet_id::CONNECT_CHARACTER);
        assert!(route.is_some());
        assert_eq!(route.unwrap().name, "ConnectCharacter");
    }

    #[test]
    fn router_returns_none_for_unknown() {
        let router = PacketRouter::new();
        assert!(router.get(255).is_none());
    }

    #[test]
    fn combat_category_has_routes() {
        let router = PacketRouter::new();
        let combat = router.by_category(RouteCategory::Combat);
        assert!(combat.len() >= 3);
    }
}
