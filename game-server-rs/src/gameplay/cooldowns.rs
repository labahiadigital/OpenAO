use std::collections::HashMap;

/// Per-player spell cooldown tracker.
#[derive(Debug, Clone, Default)]
pub struct CooldownManager {
    cooldowns: HashMap<i32, u64>,
}

impl CooldownManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the spell is ready (not on cooldown).
    pub fn is_ready(&self, spell_id: i32, now_ms: u64) -> bool {
        match self.cooldowns.get(&spell_id) {
            Some(&expires_at) => now_ms >= expires_at,
            None => true,
        }
    }

    /// Puts a spell on cooldown.
    pub fn trigger(&mut self, spell_id: i32, cooldown_ms: u64, now_ms: u64) {
        self.cooldowns.insert(spell_id, now_ms + cooldown_ms);
    }

    /// Returns remaining ms for a spell (0 if ready).
    pub fn remaining(&self, spell_id: i32, now_ms: u64) -> u64 {
        match self.cooldowns.get(&spell_id) {
            Some(&expires_at) if expires_at > now_ms => expires_at - now_ms,
            _ => 0,
        }
    }

    /// Cleans up expired cooldowns.
    pub fn cleanup(&mut self, now_ms: u64) {
        self.cooldowns.retain(|_, &mut expires| expires > now_ms);
    }

    /// Resets all cooldowns.
    pub fn reset_all(&mut self) {
        self.cooldowns.clear();
    }

    pub fn active_count(&self) -> usize {
        self.cooldowns.len()
    }
}

/// Default spell cooldowns (ms) by spell_id. Loaded from game data or defaults.
pub fn default_spell_cooldown(spell_id: i32) -> u64 {
    match spell_id {
        1..=5 => 1000,
        6..=15 => 2000,
        16..=30 => 3000,
        31..=47 => 5000,
        _ => 1500,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_ready_initially() {
        let mgr = CooldownManager::new();
        assert!(mgr.is_ready(1, 0));
    }

    #[test]
    fn spell_on_cooldown() {
        let mut mgr = CooldownManager::new();
        mgr.trigger(1, 2000, 1000);
        assert!(!mgr.is_ready(1, 1500));
        assert!(!mgr.is_ready(1, 2999));
        assert!(mgr.is_ready(1, 3000));
    }

    #[test]
    fn remaining_time() {
        let mut mgr = CooldownManager::new();
        mgr.trigger(1, 5000, 1000);
        assert_eq!(mgr.remaining(1, 1000), 5000);
        assert_eq!(mgr.remaining(1, 4000), 2000);
        assert_eq!(mgr.remaining(1, 6000), 0);
    }

    #[test]
    fn cleanup_expired() {
        let mut mgr = CooldownManager::new();
        mgr.trigger(1, 1000, 0);
        mgr.trigger(2, 5000, 0);
        mgr.cleanup(2000);
        assert_eq!(mgr.active_count(), 1);
        assert!(mgr.is_ready(1, 2000));
        assert!(!mgr.is_ready(2, 2000));
    }

    #[test]
    fn reset_all() {
        let mut mgr = CooldownManager::new();
        mgr.trigger(1, 1000, 0);
        mgr.trigger(2, 2000, 0);
        mgr.reset_all();
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn default_cooldowns() {
        assert_eq!(default_spell_cooldown(1), 1000);
        assert_eq!(default_spell_cooldown(10), 2000);
        assert_eq!(default_spell_cooldown(20), 3000);
        assert_eq!(default_spell_cooldown(40), 5000);
    }
}
