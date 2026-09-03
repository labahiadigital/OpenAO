/// Buff system with tick-based duration and automatic expiry.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuffType {
    Strength,
    Agility,
}

#[derive(Debug, Clone)]
pub struct Buff {
    pub buff_type: BuffType,
    pub magnitude: i32,
    pub remaining_ticks: u64,
}

/// Per-player buff manager. Stores active buffs and processes tick-based expiry.
#[derive(Debug, Clone, Default)]
pub struct BuffManager {
    buffs: Vec<Buff>,
}

impl BuffManager {
    pub fn new() -> Self {
        Self { buffs: Vec::new() }
    }

    /// Apply a buff. If a buff of the same type exists, replace it.
    pub fn apply(&mut self, buff_type: BuffType, magnitude: i32, duration_ticks: u64) {
        self.buffs.retain(|b| b.buff_type != buff_type);
        self.buffs.push(Buff {
            buff_type,
            magnitude,
            remaining_ticks: duration_ticks,
        });
    }

    /// Tick all buffs. Returns expired buff types.
    pub fn tick(&mut self) -> Vec<BuffType> {
        let mut expired = Vec::new();
        for buff in &mut self.buffs {
            buff.remaining_ticks = buff.remaining_ticks.saturating_sub(1);
            if buff.remaining_ticks == 0 {
                expired.push(buff.buff_type);
            }
        }
        self.buffs.retain(|b| b.remaining_ticks > 0);
        expired
    }

    /// Get the total bonus for a given buff type.
    pub fn bonus(&self, buff_type: BuffType) -> i32 {
        self.buffs
            .iter()
            .filter(|b| b.buff_type == buff_type)
            .map(|b| b.magnitude)
            .sum()
    }

    pub fn has_buff(&self, buff_type: BuffType) -> bool {
        self.buffs.iter().any(|b| b.buff_type == buff_type)
    }

    pub fn active_buffs(&self) -> &[Buff] {
        &self.buffs
    }

    pub fn clear(&mut self) {
        self.buffs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_and_tick_buff() {
        let mut mgr = BuffManager::new();
        mgr.apply(BuffType::Strength, 10, 3);
        assert_eq!(mgr.bonus(BuffType::Strength), 10);
        assert!(mgr.has_buff(BuffType::Strength));

        let expired = mgr.tick();
        assert!(expired.is_empty());
        assert_eq!(mgr.bonus(BuffType::Strength), 10);

        mgr.tick();
        let expired = mgr.tick();
        assert_eq!(expired, vec![BuffType::Strength]);
        assert_eq!(mgr.bonus(BuffType::Strength), 0);
    }

    #[test]
    fn replace_same_type_buff() {
        let mut mgr = BuffManager::new();
        mgr.apply(BuffType::Agility, 5, 100);
        mgr.apply(BuffType::Agility, 15, 200);
        assert_eq!(mgr.bonus(BuffType::Agility), 15);
        assert_eq!(mgr.active_buffs().len(), 1);
    }

    #[test]
    fn independent_buff_types() {
        let mut mgr = BuffManager::new();
        mgr.apply(BuffType::Strength, 10, 5);
        mgr.apply(BuffType::Agility, 20, 10);
        assert_eq!(mgr.bonus(BuffType::Strength), 10);
        assert_eq!(mgr.bonus(BuffType::Agility), 20);
        assert_eq!(mgr.active_buffs().len(), 2);
    }

    #[test]
    fn clear_removes_all() {
        let mut mgr = BuffManager::new();
        mgr.apply(BuffType::Strength, 10, 100);
        mgr.apply(BuffType::Agility, 20, 100);
        mgr.clear();
        assert!(mgr.active_buffs().is_empty());
    }
}
