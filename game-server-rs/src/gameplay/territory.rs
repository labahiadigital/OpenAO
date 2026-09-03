use std::collections::HashMap;

pub type TerritoryId = i32;

/// A capturable territory zone (typically tied to a map).
#[derive(Debug, Clone)]
pub struct Territory {
    pub id: TerritoryId,
    pub map_id: i32,
    pub name: String,
    pub owner_clan: Option<String>,
    pub capture_progress: u32,
    pub capture_threshold: u32,
    pub capturing_clan: Option<String>,
    pub bonus_exp_pct: i32,
    pub bonus_gold_pct: i32,
}

impl Territory {
    pub fn new(id: TerritoryId, map_id: i32, name: String) -> Self {
        Self {
            id,
            map_id,
            name,
            owner_clan: None,
            capture_progress: 0,
            capture_threshold: 100,
            capturing_clan: None,
            bonus_exp_pct: 10,
            bonus_gold_pct: 10,
        }
    }

    pub fn advance_capture(&mut self, clan_id: &str) -> bool {
        match &self.capturing_clan {
            Some(current) if current == clan_id => {
                self.capture_progress += 1;
                if self.capture_progress >= self.capture_threshold {
                    self.owner_clan = Some(clan_id.to_string());
                    self.capture_progress = 0;
                    self.capturing_clan = None;
                    return true;
                }
            }
            _ => {
                self.capturing_clan = Some(clan_id.to_string());
                self.capture_progress = 1;
            }
        }
        false
    }

    pub fn is_owned_by(&self, clan_id: &str) -> bool {
        self.owner_clan.as_deref() == Some(clan_id)
    }

    pub fn reset_capture(&mut self) {
        self.capture_progress = 0;
        self.capturing_clan = None;
    }
}

/// Global territory manager.
#[derive(Debug, Default)]
pub struct TerritoryManager {
    pub territories: HashMap<TerritoryId, Territory>,
    pub by_map: HashMap<i32, TerritoryId>,
}

impl TerritoryManager {
    pub fn new() -> Self {
        let mut mgr = Self::default();
        let default_territories = vec![
            (1, 44, "Bosque Oscuro"),
            (2, 80, "Gran Desierto"),
            (3, 60, "Montañas del Norte"),
            (4, 100, "Isla de los Piratas"),
            (5, 120, "Catacumbas Profundas"),
        ];
        for (id, map_id, name) in default_territories {
            let t = Territory::new(id, map_id, name.to_string());
            mgr.by_map.insert(map_id, id);
            mgr.territories.insert(id, t);
        }
        mgr
    }

    pub fn get_territory_for_map(&self, map_id: i32) -> Option<&Territory> {
        self.by_map.get(&map_id).and_then(|tid| self.territories.get(tid))
    }

    pub fn get_territory_for_map_mut(&mut self, map_id: i32) -> Option<&mut Territory> {
        let tid = self.by_map.get(&map_id).copied()?;
        self.territories.get_mut(&tid)
    }

    pub fn clan_territories(&self, clan_id: &str) -> Vec<&Territory> {
        self.territories.values()
            .filter(|t| t.is_owned_by(clan_id))
            .collect()
    }

    pub fn territory_info(&self, territory_id: TerritoryId) -> Option<String> {
        self.territories.get(&territory_id).map(|t| {
            let owner = t.owner_clan.as_deref().unwrap_or("Sin dueño");
            let capturing = match &t.capturing_clan {
                Some(c) => format!(" (capturando: {} {}/{})", c, t.capture_progress, t.capture_threshold),
                None => String::new(),
            };
            format!("{} (mapa {}) - Dueño: {}{}", t.name, t.map_id, owner, capturing)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_territory() {
        let mut t = Territory::new(1, 44, "Test".into());
        t.capture_threshold = 3;
        assert!(!t.advance_capture("clan_a"));
        assert!(!t.advance_capture("clan_a"));
        assert!(t.advance_capture("clan_a"));
        assert_eq!(t.owner_clan.as_deref(), Some("clan_a"));
    }

    #[test]
    fn capture_contested() {
        let mut t = Territory::new(1, 44, "Test".into());
        t.advance_capture("clan_a");
        assert_eq!(t.capture_progress, 1);
        t.advance_capture("clan_b");
        assert_eq!(t.capture_progress, 1);
        assert_eq!(t.capturing_clan.as_deref(), Some("clan_b"));
    }

    #[test]
    fn territory_manager_find() {
        let mgr = TerritoryManager::new();
        assert!(mgr.get_territory_for_map(44).is_some());
        assert!(mgr.get_territory_for_map(999).is_none());
    }

    #[test]
    fn clan_territories_count() {
        let mut mgr = TerritoryManager::new();
        mgr.territories.get_mut(&1).unwrap().owner_clan = Some("test_clan".into());
        mgr.territories.get_mut(&2).unwrap().owner_clan = Some("test_clan".into());
        assert_eq!(mgr.clan_territories("test_clan").len(), 2);
    }

    #[test]
    fn reset_capture() {
        let mut t = Territory::new(1, 44, "Test".into());
        t.advance_capture("clan_a");
        t.reset_capture();
        assert_eq!(t.capture_progress, 0);
        assert!(t.capturing_clan.is_none());
    }
}
