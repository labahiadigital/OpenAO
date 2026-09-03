use std::collections::HashMap;

/// Unique quest identifier.
pub type QuestId = u32;

/// Objective types that a quest can require.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
pub enum QuestObjective {
    #[serde(rename = "kill_npc")]
    KillNpc { npc_type: i32, count: u32 },
    #[serde(rename = "collect_item")]
    CollectItem { item_id: i32, count: u32 },
    #[serde(rename = "visit_map")]
    VisitMap { map_id: i32 },
    #[serde(rename = "talk_npc")]
    TalkNpc { npc_type: i32 },
    #[serde(rename = "reach_level")]
    ReachLevel { level: u32 },
}

/// Reward granted on quest completion.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct QuestReward {
    #[serde(default)]
    pub gold: i32,
    #[serde(default)]
    pub exp: i64,
    #[serde(default)]
    pub items: Vec<(i32, i16)>,
}

/// Static quest definition loaded from JSON.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct QuestDef {
    pub id: QuestId,
    pub name: String,
    pub description: String,
    pub npc_giver: i32,
    pub min_level: u32,
    #[serde(default)]
    pub prerequisite: Option<QuestId>,
    pub objectives: Vec<QuestObjective>,
    pub reward: QuestReward,
    #[serde(default)]
    pub repeatable: bool,
}

/// Runtime progress for a single objective within an active quest.
#[derive(Debug, Clone)]
pub struct ObjectiveProgress {
    pub current: u32,
    pub required: u32,
    pub completed: bool,
}

/// Per-player quest state for one quest.
#[derive(Debug, Clone)]
pub struct ActiveQuest {
    pub quest_id: QuestId,
    pub objectives: Vec<ObjectiveProgress>,
}

impl ActiveQuest {
    pub fn is_complete(&self) -> bool {
        self.objectives.iter().all(|o| o.completed)
    }

    pub fn advance_kill(&mut self, npc_type: i32, quest_def: &QuestDef) {
        for (i, obj) in quest_def.objectives.iter().enumerate() {
            if let QuestObjective::KillNpc { npc_type: t, count } = obj
                && *t == npc_type && !self.objectives[i].completed {
                    self.objectives[i].current += 1;
                    if self.objectives[i].current >= *count {
                        self.objectives[i].completed = true;
                    }
                }
        }
    }

    pub fn advance_collect(&mut self, item_id: i32, amount: u32, quest_def: &QuestDef) {
        for (i, obj) in quest_def.objectives.iter().enumerate() {
            if let QuestObjective::CollectItem { item_id: id, count } = obj
                && *id == item_id && !self.objectives[i].completed {
                    self.objectives[i].current += amount;
                    if self.objectives[i].current >= *count {
                        self.objectives[i].completed = true;
                    }
                }
        }
    }

    pub fn advance_visit_map(&mut self, map_id: i32, quest_def: &QuestDef) {
        for (i, obj) in quest_def.objectives.iter().enumerate() {
            if let QuestObjective::VisitMap { map_id: m } = obj
                && *m == map_id && !self.objectives[i].completed {
                    self.objectives[i].current = 1;
                    self.objectives[i].completed = true;
                }
        }
    }

    pub fn advance_talk_npc(&mut self, npc_type: i32, quest_def: &QuestDef) {
        for (i, obj) in quest_def.objectives.iter().enumerate() {
            if let QuestObjective::TalkNpc { npc_type: t } = obj
                && *t == npc_type && !self.objectives[i].completed {
                    self.objectives[i].current = 1;
                    self.objectives[i].completed = true;
                }
        }
    }

    pub fn advance_level(&mut self, level: u32, quest_def: &QuestDef) {
        for (i, obj) in quest_def.objectives.iter().enumerate() {
            if let QuestObjective::ReachLevel { level: l } = obj
                && level >= *l && !self.objectives[i].completed {
                    self.objectives[i].current = level;
                    self.objectives[i].completed = true;
                }
        }
    }
}

/// Per-player quest manager.
#[derive(Debug, Clone, Default)]
pub struct PlayerQuestLog {
    pub active: Vec<ActiveQuest>,
    pub completed: Vec<QuestId>,
}

impl PlayerQuestLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn can_accept(&self, def: &QuestDef, player_level: u32) -> Result<(), &'static str> {
        if player_level < def.min_level {
            return Err("Nivel insuficiente para esta mision.");
        }
        if let Some(prereq) = def.prerequisite
            && !self.completed.contains(&prereq) {
                return Err("Debes completar la mision anterior primero.");
            }
        if !def.repeatable && self.completed.contains(&def.id) {
            return Err("Ya completaste esta mision.");
        }
        if self.active.iter().any(|a| a.quest_id == def.id) {
            return Err("Ya tienes esta mision activa.");
        }
        if self.active.len() >= 10 {
            return Err("Tu registro de misiones esta lleno (max 10).");
        }
        Ok(())
    }

    pub fn accept(&mut self, def: &QuestDef) {
        let objectives = def.objectives.iter().map(|o| {
            let required = match o {
                QuestObjective::KillNpc { count, .. } => *count,
                QuestObjective::CollectItem { count, .. } => *count,
                QuestObjective::VisitMap { .. } => 1,
                QuestObjective::TalkNpc { .. } => 1,
                QuestObjective::ReachLevel { .. } => 1,
            };
            ObjectiveProgress { current: 0, required, completed: false }
        }).collect();
        self.active.push(ActiveQuest { quest_id: def.id, objectives });
    }

    pub fn abandon(&mut self, quest_id: QuestId) -> bool {
        let before = self.active.len();
        self.active.retain(|a| a.quest_id != quest_id);
        self.active.len() < before
    }

    pub fn complete(&mut self, quest_id: QuestId) -> bool {
        if let Some(idx) = self.active.iter().position(|a| a.quest_id == quest_id && a.is_complete()) {
            self.active.remove(idx);
            self.completed.push(quest_id);
            true
        } else {
            false
        }
    }

    pub fn get_active(&self, quest_id: QuestId) -> Option<&ActiveQuest> {
        self.active.iter().find(|a| a.quest_id == quest_id)
    }

    pub fn get_active_mut(&mut self, quest_id: QuestId) -> Option<&mut ActiveQuest> {
        self.active.iter_mut().find(|a| a.quest_id == quest_id)
    }
}

/// Global quest registry loaded from JSON at startup.
#[derive(Debug, Default)]
pub struct QuestRegistry {
    pub quests: HashMap<QuestId, QuestDef>,
    pub by_npc: HashMap<i32, Vec<QuestId>>,
}

impl QuestRegistry {
    pub fn load(data: &str) -> Result<Self, serde_json::Error> {
        let defs: Vec<QuestDef> = serde_json::from_str(data)?;
        let mut quests = HashMap::new();
        let mut by_npc: HashMap<i32, Vec<QuestId>> = HashMap::new();
        for def in defs {
            by_npc.entry(def.npc_giver).or_default().push(def.id);
            quests.insert(def.id, def);
        }
        Ok(Self { quests, by_npc })
    }

    pub fn get(&self, id: QuestId) -> Option<&QuestDef> {
        self.quests.get(&id)
    }

    pub fn quests_for_npc(&self, npc_type: i32) -> &[QuestId] {
        self.by_npc.get(&npc_type).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_quest() -> QuestDef {
        QuestDef {
            id: 1,
            name: "Caza de lobos".into(),
            description: "Mata 3 lobos".into(),
            npc_giver: 10,
            min_level: 1,
            prerequisite: None,
            objectives: vec![QuestObjective::KillNpc { npc_type: 5, count: 3 }],
            reward: QuestReward { gold: 100, exp: 50, items: vec![] },
            repeatable: false,
        }
    }

    #[test]
    fn accept_and_complete_quest() {
        let def = sample_quest();
        let mut log = PlayerQuestLog::new();
        assert!(log.can_accept(&def, 1).is_ok());
        log.accept(&def);
        assert_eq!(log.active.len(), 1);
        assert!(!log.active[0].is_complete());

        log.active[0].advance_kill(5, &def);
        log.active[0].advance_kill(5, &def);
        log.active[0].advance_kill(5, &def);
        assert!(log.active[0].is_complete());

        assert!(log.complete(1));
        assert!(log.active.is_empty());
        assert!(log.completed.contains(&1));
    }

    #[test]
    fn cannot_accept_twice() {
        let def = sample_quest();
        let mut log = PlayerQuestLog::new();
        log.accept(&def);
        assert!(log.can_accept(&def, 1).is_err());
    }

    #[test]
    fn cannot_accept_completed_non_repeatable() {
        let def = sample_quest();
        let mut log = PlayerQuestLog::new();
        log.completed.push(1);
        assert!(log.can_accept(&def, 1).is_err());
    }

    #[test]
    fn prerequisite_check() {
        let mut def = sample_quest();
        def.id = 2;
        def.prerequisite = Some(1);
        let mut log = PlayerQuestLog::new();
        assert!(log.can_accept(&def, 1).is_err());
        log.completed.push(1);
        assert!(log.can_accept(&def, 1).is_ok());
    }

    #[test]
    fn abandon_quest() {
        let def = sample_quest();
        let mut log = PlayerQuestLog::new();
        log.accept(&def);
        assert!(log.abandon(1));
        assert!(log.active.is_empty());
        assert!(!log.completed.contains(&1));
    }

    #[test]
    fn level_check() {
        let mut def = sample_quest();
        def.min_level = 10;
        let log = PlayerQuestLog::new();
        assert!(log.can_accept(&def, 5).is_err());
        assert!(log.can_accept(&def, 10).is_ok());
    }

    #[test]
    fn max_active_quests() {
        let mut log = PlayerQuestLog::new();
        for i in 0..10 {
            log.active.push(ActiveQuest { quest_id: i, objectives: vec![] });
        }
        let def = sample_quest();
        assert!(log.can_accept(&def, 1).is_err());
    }

    #[test]
    fn quest_registry_load() {
        let json = r#"[
            {"id": 1, "name": "Test", "description": "desc", "npc_giver": 10, "min_level": 1,
             "objectives": [{"type": "kill_npc", "npc_type": 5, "count": 3}],
             "reward": {"gold": 100, "exp": 50}}
        ]"#;
        let reg = QuestRegistry::load(json).unwrap();
        assert_eq!(reg.quests.len(), 1);
        assert_eq!(reg.quests_for_npc(10).len(), 1);
    }

    #[test]
    fn collect_item_objective() {
        let def = QuestDef {
            id: 2,
            name: "Recoleccion".into(),
            description: "Recolecta 5 pieles".into(),
            npc_giver: 10,
            min_level: 1,
            prerequisite: None,
            objectives: vec![QuestObjective::CollectItem { item_id: 100, count: 5 }],
            reward: QuestReward { gold: 50, exp: 30, items: vec![] },
            repeatable: true,
        };
        let mut log = PlayerQuestLog::new();
        log.accept(&def);
        log.active[0].advance_collect(100, 3, &def);
        assert!(!log.active[0].is_complete());
        log.active[0].advance_collect(100, 2, &def);
        assert!(log.active[0].is_complete());
    }
}
