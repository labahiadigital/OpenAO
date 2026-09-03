use std::collections::HashMap;
use std::sync::RwLock;

use crate::game_data::ObjectData;

pub const OBJ_TYPE_PUERTA: i32 = 6;
const DOOR_TOGGLE_COOLDOWN_MS: u64 = 250;
const DOOR_MAX_RANGE: i32 = 2;
const SND_PUERTA: u16 = 5;

#[derive(Debug, Clone)]
pub struct DoorState {
    pub obj_id: i32,
    pub is_open: bool,
}

pub struct DoorManager {
    doors: RwLock<HashMap<(i32, i32, i32), DoorState>>,
    cooldowns: RwLock<HashMap<u32, u64>>,
}

impl DoorManager {
    pub fn new() -> Self {
        Self {
            doors: RwLock::new(HashMap::new()),
            cooldowns: RwLock::new(HashMap::new()),
        }
    }

    pub fn can_toggle(&self, entity_id: u32, now_ms: u64) -> bool {
        let cd = self.cooldowns.read().unwrap();
        cd.get(&entity_id).map_or(true, |&next| now_ms >= next)
    }

    pub fn set_cooldown(&self, entity_id: u32, now_ms: u64) {
        let mut cd = self.cooldowns.write().unwrap();
        cd.insert(entity_id, now_ms + DOOR_TOGGLE_COOLDOWN_MS);
    }

    pub fn is_in_range(px: i32, py: i32, dx: i32, dy: i32) -> bool {
        (px - dx).abs() <= DOOR_MAX_RANGE && (py - dy).abs() <= DOOR_MAX_RANGE
    }

    pub fn sound_id() -> u16 {
        SND_PUERTA
    }

    pub fn try_toggle_door(
        &self,
        map_id: i32,
        x: i32,
        y: i32,
        obj_id: i32,
        obj: &ObjectData,
    ) -> Option<ToggleResult> {
        if obj.obj_type != OBJ_TYPE_PUERTA {
            return None;
        }

        if obj.llave != 0 {
            return Some(ToggleResult::Locked);
        }

        let key = (map_id, x, y);
        let mut doors = self.doors.write().unwrap();

        let state = doors.entry(key).or_insert_with(|| DoorState {
            obj_id,
            is_open: obj_id == obj.index_abierta,
        });

        if state.is_open {
            state.is_open = false;
            state.obj_id = obj.index_cerrada;
            Some(ToggleResult::Closed {
                new_obj_id: obj.index_cerrada,
            })
        } else {
            state.is_open = true;
            state.obj_id = obj.index_abierta;
            Some(ToggleResult::Opened {
                new_obj_id: obj.index_abierta,
            })
        }
    }

    pub fn get_state(&self, map_id: i32, x: i32, y: i32) -> Option<DoorState> {
        let doors = self.doors.read().unwrap();
        doors.get(&(map_id, x, y)).cloned()
    }
}

pub enum ToggleResult {
    Opened { new_obj_id: i32 },
    Closed { new_obj_id: i32 },
    Locked,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_door_obj(open_id: i32, closed_id: i32, locked: bool) -> ObjectData {
        ObjectData {
            name: "Puerta".to_string(),
            obj_type: OBJ_TYPE_PUERTA,
            index_abierta: open_id,
            index_cerrada: closed_id,
            llave: if locked { 1 } else { 0 },
            grh_index: 0,
            min_hit: 0,
            max_hit: 0,
            min_def: 0,
            max_def: 0,
            valor: 0,
            anim: 0,
            min_def_mag: 0,
            max_def_mag: 0,
            resistencia_magica: 0,
            magic_damage_bonus: 0,
            magic_penetration: 0,
            staff_damage_bonus: 0,
            spell_index: 0,
            proyectil: 0,
            newbie: 0,
            no_se_cae: 0,
            porcentaje: 0,
            tipo_pocion: 0,
            min_modificador: 0,
            max_modificador: 0,
            clases_no_permitidas: vec![],
            raza_enana: 0,
            agarrable: 0,
            cerrada: 0,
            apu: 0,
            tier: None,
            travel_ticket_destination: None,
        }
    }

    #[test]
    fn open_and_close_door() {
        let mgr = DoorManager::new();
        let obj = make_door_obj(200, 201, false);

        let result = mgr.try_toggle_door(1, 10, 10, 201, &obj);
        assert!(matches!(result, Some(ToggleResult::Opened { new_obj_id: 200 })));

        let result = mgr.try_toggle_door(1, 10, 10, 200, &obj);
        assert!(matches!(result, Some(ToggleResult::Closed { new_obj_id: 201 })));
    }

    #[test]
    fn locked_door_cannot_toggle() {
        let mgr = DoorManager::new();
        let obj = make_door_obj(200, 201, true);

        let result = mgr.try_toggle_door(1, 10, 10, 201, &obj);
        assert!(matches!(result, Some(ToggleResult::Locked)));
    }

    #[test]
    fn cooldown_prevents_rapid_toggle() {
        let mgr = DoorManager::new();
        assert!(mgr.can_toggle(1, 1000));
        mgr.set_cooldown(1, 1000);
        assert!(!mgr.can_toggle(1, 1100));
        assert!(mgr.can_toggle(1, 1250));
    }

    #[test]
    fn range_check() {
        assert!(DoorManager::is_in_range(10, 10, 11, 11));
        assert!(DoorManager::is_in_range(10, 10, 12, 12));
        assert!(!DoorManager::is_in_range(10, 10, 13, 13));
    }
}
