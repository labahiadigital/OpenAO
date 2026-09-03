use crate::routes::{PacketRouter, RouteCategory};
use openao_protocol::opcodes::server_packet_id as sid;

/// Elura-inspired WorldModule pattern for organizing game packet handlers.
///
/// Each module groups related routes by domain, mirroring Elura's
/// `WorldModule` / `WorldModuleRegistry` / `route_raw()` structure.
/// Modules register their routes into the existing `PacketRouter`,
/// maintaining full compatibility with the custom binary transport.
pub trait GameModule: Send + Sync {
    fn name(&self) -> &str;
    fn register(&self, router: &mut PacketRouter);
}

pub struct CoreGameModule;

impl GameModule for CoreGameModule {
    fn name(&self) -> &str {
        "core"
    }

    fn register(&self, r: &mut PacketRouter) {
        r.register_route(sid::CONNECT_CHARACTER, "ConnectCharacter", RouteCategory::Auth);
        r.register_route(sid::POSITION, "Position", RouteCategory::Movement);
        r.register_route(sid::CHANGE_HEADING, "ChangeHeading", RouteCategory::Movement);
        r.register_route(sid::RESYNC_POSITION, "ResyncPosition", RouteCategory::Movement);
        r.register_route(sid::DIALOG, "Dialog", RouteCategory::Dialog);
        r.register_route(sid::ATTACK_MELE, "AttackMelee", RouteCategory::Combat);
        r.register_route(sid::ATTACK_RANGE, "AttackRange", RouteCategory::Combat);
        r.register_route(sid::ATTACK_SPELL, "AttackSpell", RouteCategory::Combat);
        r.register_route(sid::USE_ITEM_CLICK, "UseItemClick", RouteCategory::Inventory);
        r.register_route(sid::USE_ITEM_U, "UseItemU", RouteCategory::Inventory);
        r.register_route(sid::TIRAR_ITEM, "DropItem", RouteCategory::Inventory);
        r.register_route(sid::AGARRAR_ITEM, "PickupItem", RouteCategory::Inventory);
        r.register_route(sid::EQUIPAR_ITEM, "EquipItem", RouteCategory::Inventory);
        r.register_route(sid::REORDER_INVENTORY_ITEM, "ReorderInventory", RouteCategory::Inventory);
        r.register_route(sid::REORDER_SPELL, "ReorderSpell", RouteCategory::Inventory);
        r.register_route(sid::CLICK, "Click", RouteCategory::System);
    }
}

pub struct CommerceModule;

impl GameModule for CommerceModule {
    fn name(&self) -> &str {
        "commerce"
    }

    fn register(&self, r: &mut PacketRouter) {
        r.register_route(sid::BUY_ITEM, "BuyItem", RouteCategory::Commerce);
        r.register_route(sid::SELL_ITEM, "SellItem", RouteCategory::Commerce);
        r.register_route(sid::CLOSE_TRADE, "CloseTrade", RouteCategory::Commerce);
        r.register_route(sid::DEPOSIT_BANK_GOLD, "DepositBankGold", RouteCategory::Bank);
        r.register_route(sid::WITHDRAW_BANK_GOLD, "WithdrawBankGold", RouteCategory::Bank);
        r.register_route(sid::REORDER_BANK_ITEM, "ReorderBankItem", RouteCategory::Bank);
        r.register_route(sid::CHANGE_BANK_TAB, "ChangeBankTab", RouteCategory::Bank);
        r.register_route(sid::MARKET_ACTION, "MarketAction", RouteCategory::Market);
    }
}

pub struct SocialModule;

impl GameModule for SocialModule {
    fn name(&self) -> &str {
        "social"
    }

    fn register(&self, r: &mut PacketRouter) {
        r.register_route(sid::RETOS_ACTION, "RetosAction", RouteCategory::Challenge);
    }
}

pub struct GatheringModule;

impl GameModule for GatheringModule {
    fn name(&self) -> &str {
        "gathering"
    }

    fn register(&self, r: &mut PacketRouter) {
        r.register_route(sid::CRAFT_ITEM, "CraftItem", RouteCategory::Crafting);
    }
}

pub struct SystemModule;

impl GameModule for SystemModule {
    fn name(&self) -> &str {
        "system"
    }

    fn register(&self, r: &mut PacketRouter) {
        r.register_route(sid::PING, "Ping", RouteCategory::System);
        r.register_route(sid::CHANGE_SEGURO, "ToggleSafe", RouteCategory::System);
        r.register_route(sid::TOGGLE_HIDDEN_SKILL, "ToggleHidden", RouteCategory::System);
        r.register_route(sid::CHANGE_CLAN_SEGURO, "ToggleClanSafe", RouteCategory::System);
    }
}

/// Build a `PacketRouter` from all installed game modules.
pub fn build_router_from_modules() -> PacketRouter {
    let modules: Vec<Box<dyn GameModule>> = vec![
        Box::new(CoreGameModule),
        Box::new(CommerceModule),
        Box::new(SocialModule),
        Box::new(GatheringModule),
        Box::new(SystemModule),
    ];

    let mut router = PacketRouter::empty();
    for module in &modules {
        module.register(&mut router);
        tracing::debug!("Installed game module: {}", module.name());
    }
    tracing::info!(
        "Game module registry: {} routes across {} modules",
        router.len(),
        modules.len()
    );
    router
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_modules_register_without_panic() {
        let router = build_router_from_modules();
        assert!(router.len() > 20);
    }

    #[test]
    fn modules_produce_same_routes_as_original() {
        let original = PacketRouter::new();
        let modular = build_router_from_modules();
        assert_eq!(original.len(), modular.len());

        for route in original.all() {
            let found = modular.get(route.id);
            assert!(found.is_some(), "Missing route: {} ({})", route.name, route.id);
            assert_eq!(found.unwrap().name, route.name);
            assert_eq!(found.unwrap().category, route.category);
            assert_eq!(found.unwrap().requires_character, route.requires_character);
        }
    }

    #[test]
    fn module_names_are_correct() {
        assert_eq!(CoreGameModule.name(), "core");
        assert_eq!(CommerceModule.name(), "commerce");
        assert_eq!(SocialModule.name(), "social");
        assert_eq!(GatheringModule.name(), "gathering");
        assert_eq!(SystemModule.name(), "system");
    }
}
