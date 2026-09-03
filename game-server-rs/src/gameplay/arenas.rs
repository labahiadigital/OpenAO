use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use uuid::Uuid;

use crate::world::{EntityId, MapId};

const ARENA_MAP_START: i32 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaState {
    WaitingForPlayers,
    InProgress,
    Finished,
}

#[derive(Debug, Clone)]
pub struct ArenaParticipant {
    pub entity_id: EntityId,
    pub character_id: Uuid,
    pub account_id: String,
    pub team: u8,
    pub kills: u32,
    pub deaths: u32,
}

pub struct ArenaInstance {
    pub id: Uuid,
    pub challenge_id: Uuid,
    pub base_map_id: MapId,
    pub map_id: MapId,
    pub state: ArenaState,
    pub participants: Vec<ArenaParticipant>,
    pub start_time: Option<i64>,
    pub npc_entity_ids: Vec<EntityId>,
}

pub fn is_arena_map(map_id: i32) -> bool {
    map_id >= ARENA_MAP_START
}

pub struct ArenaManager {
    instances: HashMap<Uuid, ArenaInstance>,
    room_to_arena: HashMap<String, Uuid>,
    pending_handovers: HashMap<String, String>,
    next_map_id: AtomicI32,
}

impl ArenaManager {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            room_to_arena: HashMap::new(),
            pending_handovers: HashMap::new(),
            next_map_id: AtomicI32::new(ARENA_MAP_START),
        }
    }

    fn allocate_map_id(&self) -> MapId {
        self.next_map_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn get_or_create_instance(
        &mut self,
        room_id: &str,
        base_map_id: MapId,
        challenge_id: Uuid,
    ) -> &ArenaInstance {
        if let Some(arena_id) = self.room_to_arena.get(room_id) {
            if self.instances.contains_key(arena_id) {
                return self.instances.get(arena_id).unwrap();
            }
        }

        let id = Uuid::new_v4();
        let map_id = self.allocate_map_id();

        let instance = ArenaInstance {
            id,
            challenge_id,
            base_map_id,
            map_id,
            state: ArenaState::WaitingForPlayers,
            participants: Vec::new(),
            start_time: None,
            npc_entity_ids: Vec::new(),
        };

        self.instances.insert(id, instance);
        self.room_to_arena.insert(room_id.to_string(), id);
        self.instances.get(&id).unwrap()
    }

    pub fn add_participant(
        &mut self,
        arena_id: Uuid,
        participant: ArenaParticipant,
    ) -> Result<(), &'static str> {
        let arena = self.instances.get_mut(&arena_id).ok_or("Arena not found")?;
        if arena.state == ArenaState::Finished {
            return Err("Arena already finished");
        }
        arena.participants.push(participant);
        Ok(())
    }

    pub fn start_arena(&mut self, arena_id: Uuid) -> Result<MapId, &'static str> {
        let arena = self.instances.get_mut(&arena_id).ok_or("Arena not found")?;
        if arena.state != ArenaState::WaitingForPlayers {
            return Err("Arena already started");
        }
        arena.state = ArenaState::InProgress;
        arena.start_time = Some(chrono::Utc::now().timestamp_millis());
        Ok(arena.map_id)
    }

    pub fn finish_arena(&mut self, arena_id: Uuid) -> Option<ArenaInstance> {
        if let Some(arena) = self.instances.get_mut(&arena_id) {
            arena.state = ArenaState::Finished;
        }
        let instance = self.instances.remove(&arena_id)?;
        self.room_to_arena.retain(|_, v| *v != arena_id);
        Some(instance)
    }

    pub fn get_arena(&self, arena_id: Uuid) -> Option<&ArenaInstance> {
        self.instances.get(&arena_id)
    }

    pub fn get_arena_by_room(&self, room_id: &str) -> Option<&ArenaInstance> {
        let arena_id = self.room_to_arena.get(room_id)?;
        self.instances.get(arena_id)
    }

    pub fn get_arena_map_id(&self, room_id: &str) -> Option<MapId> {
        self.get_arena_by_room(room_id).map(|a| a.map_id)
    }

    pub fn find_player_arena(&self, entity_id: EntityId) -> Option<&ArenaInstance> {
        self.instances.values().find(|a| {
            a.participants.iter().any(|p| p.entity_id == entity_id)
        })
    }

    pub fn count_players_in_room(&self, room_id: &str) -> usize {
        self.get_arena_by_room(room_id)
            .map(|a| a.participants.len())
            .unwrap_or(0)
    }

    pub fn is_account_in_room(&self, room_id: &str, account_id: &str, exclude_entity: Option<EntityId>) -> bool {
        let Some(arena) = self.get_arena_by_room(room_id) else { return false; };
        arena.participants.iter().any(|p| {
            p.account_id == account_id && exclude_entity.map_or(true, |eid| p.entity_id != eid)
        })
    }

    pub fn begin_handover(&mut self, room_id: &str, account_id: &str) {
        self.pending_handovers.insert(account_id.to_string(), room_id.to_string());
    }

    pub fn end_handover(&mut self, room_id: &str, account_id: &str) {
        if self.pending_handovers.get(account_id).map(|r| r.as_str()) == Some(room_id) {
            self.pending_handovers.remove(account_id);
        }
    }

    pub fn has_pending_handover(&self, room_id: &str, account_id: &str) -> bool {
        self.pending_handovers.get(account_id).map(|r| r.as_str()) == Some(room_id)
    }

    pub fn remove_participant(&mut self, arena_id: Uuid, entity_id: EntityId) {
        if let Some(arena) = self.instances.get_mut(&arena_id) {
            arena.participants.retain(|p| p.entity_id != entity_id);
        }
    }

    pub fn register_npc(&mut self, arena_id: Uuid, npc_entity_id: EntityId) {
        if let Some(arena) = self.instances.get_mut(&arena_id) {
            arena.npc_entity_ids.push(npc_entity_id);
        }
    }

    pub fn destroy_instance(&mut self, room_id: &str) -> Option<ArenaInstance> {
        let arena_id = self.room_to_arena.remove(room_id)?;
        let instance = self.instances.remove(&arena_id)?;
        self.pending_handovers.retain(|_, v| v != room_id);
        Some(instance)
    }

    pub fn record_kill(&mut self, arena_id: Uuid, killer_entity: EntityId, victim_entity: EntityId) {
        let Some(arena) = self.instances.get_mut(&arena_id) else { return; };
        if let Some(killer) = arena.participants.iter_mut().find(|p| p.entity_id == killer_entity) {
            killer.kills += 1;
        }
        if let Some(victim) = arena.participants.iter_mut().find(|p| p.entity_id == victim_entity) {
            victim.deaths += 1;
        }
    }

    pub fn get_team_scores(&self, arena_id: Uuid) -> HashMap<u8, (u32, u32)> {
        let Some(arena) = self.instances.get(&arena_id) else {
            return HashMap::new();
        };
        let mut scores: HashMap<u8, (u32, u32)> = HashMap::new();
        for p in &arena.participants {
            let entry = scores.entry(p.team).or_insert((0, 0));
            entry.0 += p.kills;
            entry.1 += p.deaths;
        }
        scores
    }

    pub fn get_winning_team(&self, arena_id: Uuid) -> Option<u8> {
        let scores = self.get_team_scores(arena_id);
        scores.into_iter().max_by_key(|(_, (kills, _))| *kills).map(|(team, _)| team)
    }

    pub fn active_count(&self) -> usize {
        self.instances.values().filter(|a| a.state != ArenaState::Finished).count()
    }

    pub fn list_active(&self) -> Vec<(Uuid, MapId, ArenaState, usize)> {
        self.instances.values()
            .filter(|a| a.state != ArenaState::Finished)
            .map(|a| (a.id, a.map_id, a.state, a.participants.len()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_start_arena() {
        let mut mgr = ArenaManager::new();
        let cid = Uuid::new_v4();
        let inst = mgr.get_or_create_instance("room-1", 272, cid);
        let aid = inst.id;
        assert_eq!(inst.state, ArenaState::WaitingForPlayers);
        assert!(inst.map_id >= ARENA_MAP_START);

        let map_id = mgr.start_arena(aid).unwrap();
        assert!(map_id >= ARENA_MAP_START);

        let arena = mgr.get_arena(aid).unwrap();
        assert_eq!(arena.state, ArenaState::InProgress);
    }

    #[test]
    fn duplicate_room_returns_same_instance() {
        let mut mgr = ArenaManager::new();
        let cid = Uuid::new_v4();
        let id1 = mgr.get_or_create_instance("room-x", 272, cid).id;
        let id2 = mgr.get_or_create_instance("room-x", 272, cid).id;
        assert_eq!(id1, id2);
    }

    #[test]
    fn handover_lifecycle() {
        let mut mgr = ArenaManager::new();
        assert!(!mgr.has_pending_handover("r1", "acc1"));
        mgr.begin_handover("r1", "acc1");
        assert!(mgr.has_pending_handover("r1", "acc1"));
        mgr.end_handover("r1", "acc1");
        assert!(!mgr.has_pending_handover("r1", "acc1"));
    }

    #[test]
    fn destroy_cleans_up() {
        let mut mgr = ArenaManager::new();
        let cid = Uuid::new_v4();
        mgr.get_or_create_instance("room-del", 272, cid);
        assert_eq!(mgr.active_count(), 1);
        mgr.destroy_instance("room-del");
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn participant_tracking() {
        let mut mgr = ArenaManager::new();
        let cid = Uuid::new_v4();
        let aid = mgr.get_or_create_instance("room-p", 272, cid).id;
        mgr.add_participant(aid, ArenaParticipant {
            entity_id: 42,
            character_id: Uuid::new_v4(),
            account_id: "acc-1".to_string(),
            team: 1,
            kills: 0,
            deaths: 0,
        }).unwrap();
        assert!(mgr.is_account_in_room("room-p", "acc-1", None));
        assert!(!mgr.is_account_in_room("room-p", "acc-2", None));

        let found = mgr.find_player_arena(42);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, aid);
    }
}
