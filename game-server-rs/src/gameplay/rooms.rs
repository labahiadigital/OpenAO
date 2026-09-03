use std::collections::HashMap;
use uuid::Uuid;

use elura::gameplay::room::{Room, RoomConfig, RoomError, RoomPhase};

use crate::world::EntityId;

pub type ChallengeRoomId = Uuid;

#[derive(Debug, Clone)]
pub struct ChallengeParticipantData {
    pub character_id: Uuid,
    pub name: String,
    pub level: i32,
    pub class_name: String,
    pub race_name: String,
}

pub struct ChallengeRoomManager {
    rooms: HashMap<ChallengeRoomId, Room<ChallengeRoomId, EntityId, ChallengeParticipantData>>,
    created_at: HashMap<ChallengeRoomId, i64>,
}

impl ChallengeRoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            created_at: HashMap::new(),
        }
    }

    pub fn create(
        &mut self,
        team_size: usize,
        creator_entity: EntityId,
        data: ChallengeParticipantData,
        now_millis: i64,
    ) -> Result<ChallengeRoomId, RoomError> {
        let id = Uuid::new_v4();
        let capacity = team_size * 2;

        let mut config = RoomConfig::default();
        config.capacity = capacity;
        config.minimum_to_start = capacity;
        config.require_all_ready = false;
        config.allow_join_in_progress = false;

        let mut room = Room::new(id, config)?;
        room.join(creator_entity, data)?;

        self.rooms.insert(id, room);
        self.created_at.insert(id, now_millis);
        Ok(id)
    }

    pub fn join(
        &mut self,
        room_id: ChallengeRoomId,
        entity: EntityId,
        data: ChallengeParticipantData,
    ) -> Result<bool, RoomError> {
        let room = self.rooms.get_mut(&room_id).ok_or(RoomError::MemberNotFound)?;

        if room.phase() != RoomPhase::Open {
            return Err(RoomError::NotOpen);
        }

        room.join(entity, data)?;

        let is_full = room.len() == room.config().capacity;
        if is_full {
            room.start().ok();
        }
        Ok(is_full)
    }

    pub fn cancel(&mut self, room_id: ChallengeRoomId) -> bool {
        let removed = self.rooms.remove(&room_id).is_some();
        if removed {
            self.created_at.remove(&room_id);
        }
        removed
    }

    pub fn leave(&mut self, room_id: ChallengeRoomId, entity: &EntityId) -> Result<bool, RoomError> {
        let room = self.rooms.get_mut(&room_id).ok_or(RoomError::MemberNotFound)?;
        let departure = room.leave(entity)?;
        if departure.empty {
            self.rooms.remove(&room_id);
            self.created_at.remove(&room_id);
        }
        Ok(departure.empty)
    }

    pub fn get_room(&self, room_id: &ChallengeRoomId) -> Option<&Room<ChallengeRoomId, EntityId, ChallengeParticipantData>> {
        self.rooms.get(room_id)
    }

    pub fn created_at(&self, room_id: &ChallengeRoomId) -> Option<i64> {
        self.created_at.get(room_id).copied()
    }

    pub fn list_open(&self) -> Vec<&Room<ChallengeRoomId, EntityId, ChallengeParticipantData>> {
        self.rooms.values().filter(|r| r.phase() == RoomPhase::Open).collect()
    }

    pub fn list_all(&self) -> Vec<&Room<ChallengeRoomId, EntityId, ChallengeParticipantData>> {
        self.rooms.values().collect()
    }

    pub fn is_ready(&self, room_id: &ChallengeRoomId) -> bool {
        self.rooms.get(room_id).is_some_and(|r| r.phase() == RoomPhase::Active)
    }
}

impl Default for ChallengeRoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data(name: &str) -> ChallengeParticipantData {
        ChallengeParticipantData {
            character_id: Uuid::new_v4(),
            name: name.to_string(),
            level: 10,
            class_name: "Guerrero".to_string(),
            race_name: "Humano".to_string(),
        }
    }

    #[test]
    fn create_and_join_solo_challenge() {
        let mut mgr = ChallengeRoomManager::new();
        let id = mgr.create(1, 100, make_data("Player1"), 1000).unwrap();

        assert!(!mgr.is_ready(&id));

        let is_full = mgr.join(id, 200, make_data("Player2")).unwrap();
        assert!(is_full);
        assert!(mgr.is_ready(&id));
    }

    #[test]
    fn create_and_join_duo_challenge() {
        let mut mgr = ChallengeRoomManager::new();
        let id = mgr.create(2, 10, make_data("P1"), 1000).unwrap();

        mgr.join(id, 20, make_data("P2")).unwrap();
        assert!(!mgr.is_ready(&id));

        mgr.join(id, 30, make_data("P3")).unwrap();
        assert!(!mgr.is_ready(&id));

        let is_full = mgr.join(id, 40, make_data("P4")).unwrap();
        assert!(is_full);
        assert!(mgr.is_ready(&id));
    }

    #[test]
    fn cancel_removes_room() {
        let mut mgr = ChallengeRoomManager::new();
        let id = mgr.create(1, 100, make_data("P1"), 1000).unwrap();
        assert!(mgr.cancel(id));
        assert!(!mgr.cancel(id));
        assert!(mgr.get_room(&id).is_none());
    }

    #[test]
    fn duplicate_join_rejected() {
        let mut mgr = ChallengeRoomManager::new();
        let id = mgr.create(1, 100, make_data("P1"), 1000).unwrap();
        let result = mgr.join(id, 100, make_data("P1"));
        assert!(matches!(result, Err(RoomError::AlreadyMember)));
    }

    #[test]
    fn full_room_rejects_extra() {
        let mut mgr = ChallengeRoomManager::new();
        let id = mgr.create(1, 100, make_data("P1"), 1000).unwrap();
        mgr.join(id, 200, make_data("P2")).unwrap();
        let result = mgr.join(id, 300, make_data("P3"));
        assert!(result.is_err());
    }

    #[test]
    fn leave_auto_removes_empty_room() {
        let mut mgr = ChallengeRoomManager::new();
        let id = mgr.create(1, 100, make_data("P1"), 1000).unwrap();
        let empty = mgr.leave(id, &100).unwrap();
        assert!(empty);
        assert!(mgr.get_room(&id).is_none());
    }

    #[test]
    fn list_open_only_shows_open_rooms() {
        let mut mgr = ChallengeRoomManager::new();
        let id1 = mgr.create(1, 100, make_data("P1"), 1000).unwrap();
        let id2 = mgr.create(1, 200, make_data("P2"), 2000).unwrap();

        mgr.join(id1, 300, make_data("P3")).unwrap();

        let open = mgr.list_open();
        assert_eq!(open.len(), 1);
        assert_eq!(*open[0].id(), id2);
    }
}
