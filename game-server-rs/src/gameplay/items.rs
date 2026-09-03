use crate::game_data::GameData;
use crate::persistence::InventoryRow;
use crate::world::PlayerState;

/// Object type IDs matching the original `vars.objType`.
pub const OBJ_TYPE_ARMAS: i32 = 2;
pub const OBJ_TYPE_ARMADURAS: i32 = 3;
pub const OBJ_TYPE_CASCOS: i32 = 4;
pub const OBJ_TYPE_ESCUDOS: i32 = 8;
pub const OBJ_TYPE_FLECHAS: i32 = 9;
pub const OBJ_TYPE_ANILLOS: i32 = 10;

/// Item slot in the player's inventory.
#[derive(Debug, Clone)]
pub struct InventorySlot {
    pub item_id: i32,
    pub quantity: i32,
    pub equipped: bool,
}

/// Maximum inventory size.
pub const MAX_INVENTORY_SLOTS: usize = 20;

/// Maximum bank slots.
pub const MAX_BANK_SLOTS: usize = 50;

/// Maximum spell slots.
pub const MAX_SPELL_SLOTS: usize = 35;

#[derive(Debug, Clone)]
pub struct Inventory {
    pub slots: Vec<Option<InventorySlot>>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            slots: vec![None; MAX_INVENTORY_SLOTS],
        }
    }

    pub fn find_empty_slot(&self) -> Option<usize> {
        self.slots.iter().position(|s| s.is_none())
    }

    pub fn add_item(&mut self, item_id: i32, quantity: i32) -> bool {
        if let Some(existing) = self.slots.iter_mut().find(|s| {
            matches!(s, Some(slot) if slot.item_id == item_id && !slot.equipped)
        })
            && let Some(slot) = existing
        {
            slot.quantity += quantity;
            return true;
        }

        if let Some(empty) = self.find_empty_slot() {
            self.slots[empty] = Some(InventorySlot {
                item_id,
                quantity,
                equipped: false,
            });
            return true;
        }

        false
    }

    pub fn remove_item(&mut self, slot_idx: usize, quantity: i32) -> bool {
        if slot_idx >= self.slots.len() {
            return false;
        }

        if let Some(ref mut slot) = self.slots[slot_idx] {
            if slot.quantity < quantity {
                return false;
            }
            slot.quantity -= quantity;
            if slot.quantity <= 0 {
                self.slots[slot_idx] = None;
            }
            true
        } else {
            false
        }
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

/// Recalculates visual equipment IDs on a `PlayerState` from the current
/// inventory cache. Ported 1:1 from `rebuildEquippedInventoryState` in the
/// original Node.js `game.ts`.
///
/// When `navegando` is true, weapon/body/shield/helmet visuals remain 0
/// because boat mode hides worn gear — matching the original behavior.
pub fn rebuild_equipped_visuals(player: &mut PlayerState, inv: &[InventoryRow], gd: &GameData) {
    player.id_weapon = 0;
    player.id_body = 0;
    player.id_shield = 0;
    player.id_helmet = 0;
    player.id_arrow_slot = 0;
    player.id_ring_slot = 0;

    for row in inv {
        if !row.equipped {
            continue;
        }
        let obj = match gd.get_object(row.item_id) {
            Some(o) => o,
            None => continue,
        };
        let anim = obj.anim as i32;
        match obj.obj_type {
            OBJ_TYPE_ARMAS => {
                if !player.navegando {
                    player.id_weapon = anim;
                }
            }
            OBJ_TYPE_ARMADURAS => {
                if !player.navegando {
                    player.id_body = anim;
                }
            }
            OBJ_TYPE_CASCOS => {
                if !player.navegando {
                    player.id_helmet = anim;
                }
            }
            OBJ_TYPE_ESCUDOS => {
                if !player.navegando {
                    player.id_shield = anim;
                }
            }
            OBJ_TYPE_FLECHAS => {
                player.id_arrow_slot = row.item_id;
            }
            OBJ_TYPE_ANILLOS => {
                player.id_ring_slot = row.item_id;
            }
            _ => {}
        }
    }
}

/// Checks whether an item can be auto-equipped by a character, considering
/// class and race restrictions. Ported from `canAutoEquipInventoryItem`.
pub fn can_auto_equip(player_class: i32, player_race: i32, obj: &crate::game_data::ObjectData) -> bool {
    if !obj.clases_no_permitidas.is_empty() && obj.clases_no_permitidas.contains(&player_class) {
        return false;
    }
    let is_dwarf_race = player_race == 4 || player_race == 5;
    if obj.obj_type == OBJ_TYPE_ARMADURAS {
        if obj.raza_enana == 1 && !is_dwarf_race {
            return false;
        }
        if obj.raza_enana == 0 && is_dwarf_race {
            return false;
        }
    }
    true
}

/// Returns true if the given object type is an equipment slot type.
pub fn is_equipment_type(obj_type: i32) -> bool {
    matches!(
        obj_type,
        OBJ_TYPE_ARMAS | OBJ_TYPE_ARMADURAS | OBJ_TYPE_CASCOS
        | OBJ_TYPE_ESCUDOS | OBJ_TYPE_FLECHAS | OBJ_TYPE_ANILLOS
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_equipment_type_matches_all_slots() {
        assert!(is_equipment_type(OBJ_TYPE_ARMAS));
        assert!(is_equipment_type(OBJ_TYPE_ARMADURAS));
        assert!(is_equipment_type(OBJ_TYPE_CASCOS));
        assert!(is_equipment_type(OBJ_TYPE_ESCUDOS));
        assert!(is_equipment_type(OBJ_TYPE_FLECHAS));
        assert!(is_equipment_type(OBJ_TYPE_ANILLOS));
        assert!(!is_equipment_type(1));  // potions
        assert!(!is_equipment_type(14)); // boats
    }

    #[test]
    fn add_and_remove() {
        let mut inv = Inventory::new();
        assert!(inv.add_item(100, 5));
        assert!(inv.remove_item(0, 3));
        assert_eq!(inv.slots[0].as_ref().unwrap().quantity, 2);
        assert!(inv.remove_item(0, 2));
        assert!(inv.slots[0].is_none());
    }

    #[test]
    fn stack_items() {
        let mut inv = Inventory::new();
        inv.add_item(100, 5);
        inv.add_item(100, 3);
        assert_eq!(inv.slots[0].as_ref().unwrap().quantity, 8);
        assert!(inv.slots[1].is_none());
    }

    #[test]
    fn inventory_full() {
        let mut inv = Inventory::new();
        for i in 0..MAX_INVENTORY_SLOTS {
            assert!(inv.add_item(i as i32 + 1, 1));
        }
        assert!(!inv.add_item(999, 1));
    }
}
