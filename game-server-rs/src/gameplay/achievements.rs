use std::collections::HashSet;

pub type AchievementId = u32;

/// Achievement trigger conditions.
#[derive(Debug, Clone)]
pub enum AchievementCondition {
    ReachLevel(u32),
    KillNpcs(u32),
    CollectGold(i32),
    CompleteQuests(u32),
    JoinClan,
    WinChallenge,
    CatchFish(u32),
    CraftItems(u32),
    VisitMaps(u32),
    Die(u32),
}

/// Static achievement definition.
#[derive(Debug, Clone)]
pub struct AchievementDef {
    pub id: AchievementId,
    pub name: String,
    pub description: String,
    pub condition: AchievementCondition,
    pub reward_gold: i32,
    pub reward_exp: i32,
}

/// Per-player achievement tracker.
#[derive(Debug, Clone, Default)]
pub struct AchievementTracker {
    pub unlocked: HashSet<AchievementId>,
    pub stats: PlayerStats,
}

/// Cumulative player stats for achievement checks.
#[derive(Debug, Clone, Default)]
pub struct PlayerStats {
    pub total_npc_kills: u32,
    pub total_quests_completed: u32,
    pub total_fish_caught: u32,
    pub total_items_crafted: u32,
    pub total_maps_visited: u32,
    pub total_deaths: u32,
    pub joined_clan: bool,
    pub won_challenge: bool,
}

impl AchievementTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_unlocked(&self, id: AchievementId) -> bool {
        self.unlocked.contains(&id)
    }

    pub fn check_and_unlock(&mut self, defs: &[AchievementDef], level: u32, gold: i32) -> Vec<AchievementId> {
        let mut newly_unlocked = Vec::new();
        for def in defs {
            if self.unlocked.contains(&def.id) {
                continue;
            }
            let met = match &def.condition {
                AchievementCondition::ReachLevel(l) => level >= *l,
                AchievementCondition::KillNpcs(n) => self.stats.total_npc_kills >= *n,
                AchievementCondition::CollectGold(g) => gold >= *g,
                AchievementCondition::CompleteQuests(n) => self.stats.total_quests_completed >= *n,
                AchievementCondition::JoinClan => self.stats.joined_clan,
                AchievementCondition::WinChallenge => self.stats.won_challenge,
                AchievementCondition::CatchFish(n) => self.stats.total_fish_caught >= *n,
                AchievementCondition::CraftItems(n) => self.stats.total_items_crafted >= *n,
                AchievementCondition::VisitMaps(n) => self.stats.total_maps_visited >= *n,
                AchievementCondition::Die(n) => self.stats.total_deaths >= *n,
            };
            if met {
                self.unlocked.insert(def.id);
                newly_unlocked.push(def.id);
            }
        }
        newly_unlocked
    }
}

/// Default achievement definitions.
pub fn default_achievements() -> Vec<AchievementDef> {
    vec![
        AchievementDef {
            id: 1, name: "Primer nivel".into(), description: "Alcanza el nivel 2.".into(),
            condition: AchievementCondition::ReachLevel(2), reward_gold: 50, reward_exp: 100,
        },
        AchievementDef {
            id: 2, name: "Guerrero".into(), description: "Alcanza el nivel 10.".into(),
            condition: AchievementCondition::ReachLevel(10), reward_gold: 200, reward_exp: 500,
        },
        AchievementDef {
            id: 3, name: "Veterano".into(), description: "Alcanza el nivel 25.".into(),
            condition: AchievementCondition::ReachLevel(25), reward_gold: 1000, reward_exp: 2000,
        },
        AchievementDef {
            id: 4, name: "Cazador novato".into(), description: "Mata 10 NPCs.".into(),
            condition: AchievementCondition::KillNpcs(10), reward_gold: 100, reward_exp: 200,
        },
        AchievementDef {
            id: 5, name: "Exterminador".into(), description: "Mata 100 NPCs.".into(),
            condition: AchievementCondition::KillNpcs(100), reward_gold: 500, reward_exp: 1000,
        },
        AchievementDef {
            id: 6, name: "Ricachón".into(), description: "Acumula 1000 de oro.".into(),
            condition: AchievementCondition::CollectGold(1000), reward_gold: 0, reward_exp: 300,
        },
        AchievementDef {
            id: 7, name: "Misionero".into(), description: "Completa 3 misiones.".into(),
            condition: AchievementCondition::CompleteQuests(3), reward_gold: 300, reward_exp: 500,
        },
        AchievementDef {
            id: 8, name: "Social".into(), description: "Únete a un clan.".into(),
            condition: AchievementCondition::JoinClan, reward_gold: 100, reward_exp: 200,
        },
        AchievementDef {
            id: 9, name: "Campeón".into(), description: "Gana un desafío.".into(),
            condition: AchievementCondition::WinChallenge, reward_gold: 200, reward_exp: 500,
        },
        AchievementDef {
            id: 10, name: "Pescador".into(), description: "Pesca 10 veces.".into(),
            condition: AchievementCondition::CatchFish(10), reward_gold: 150, reward_exp: 300,
        },
        AchievementDef {
            id: 11, name: "Artesano".into(), description: "Craftea 5 items.".into(),
            condition: AchievementCondition::CraftItems(5), reward_gold: 200, reward_exp: 400,
        },
        AchievementDef {
            id: 12, name: "Explorador".into(), description: "Visita 10 mapas distintos.".into(),
            condition: AchievementCondition::VisitMaps(10), reward_gold: 300, reward_exp: 600,
        },
        AchievementDef {
            id: 13, name: "Persistente".into(), description: "Muere 5 veces (y sigue jugando).".into(),
            condition: AchievementCondition::Die(5), reward_gold: 50, reward_exp: 100,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlock_level_achievement() {
        let defs = default_achievements();
        let mut tracker = AchievementTracker::new();
        let unlocked = tracker.check_and_unlock(&defs, 2, 0);
        assert!(unlocked.contains(&1));
        assert!(!unlocked.contains(&2));
    }

    #[test]
    fn no_double_unlock() {
        let defs = default_achievements();
        let mut tracker = AchievementTracker::new();
        tracker.check_and_unlock(&defs, 2, 0);
        let unlocked2 = tracker.check_and_unlock(&defs, 2, 0);
        assert!(unlocked2.is_empty());
    }

    #[test]
    fn kill_achievement() {
        let defs = default_achievements();
        let mut tracker = AchievementTracker::new();
        tracker.stats.total_npc_kills = 10;
        let unlocked = tracker.check_and_unlock(&defs, 1, 0);
        assert!(unlocked.contains(&4));
    }

    #[test]
    fn gold_achievement() {
        let defs = default_achievements();
        let mut tracker = AchievementTracker::new();
        let unlocked = tracker.check_and_unlock(&defs, 1, 1000);
        assert!(unlocked.contains(&6));
    }

    #[test]
    fn multiple_achievements_at_once() {
        let defs = default_achievements();
        let mut tracker = AchievementTracker::new();
        tracker.stats.total_npc_kills = 100;
        let unlocked = tracker.check_and_unlock(&defs, 10, 1000);
        assert!(unlocked.len() >= 4);
    }

    #[test]
    fn clan_and_challenge_achievements() {
        let defs = default_achievements();
        let mut tracker = AchievementTracker::new();
        tracker.stats.joined_clan = true;
        tracker.stats.won_challenge = true;
        let unlocked = tracker.check_and_unlock(&defs, 1, 0);
        assert!(unlocked.contains(&8));
        assert!(unlocked.contains(&9));
    }
}
