use elura::gameplay::lag_compensation::{LagCompensationConfig, LagCompensationHistory};

use crate::world::Position;

/// Snapshot of combat-relevant state for one entity at a given tick,
/// used for lag-compensated hit validation.
#[derive(Debug, Clone)]
pub struct CombatSnapshot {
    pub entity_id: u32,
    pub pos: Position,
    pub hp: i32,
    pub dead: bool,
}

/// Per-scene history used to validate ranged/spell hits against where
/// entities actually were when the attacker fired.
pub struct SceneLagHistory {
    inner: LagCompensationHistory<Vec<CombatSnapshot>>,
}

impl SceneLagHistory {
    pub fn new() -> Self {
        let mut config = LagCompensationConfig::default();
        config.history_capacity = 64;
        config.max_rewind_ticks = 30;
        Self {
            inner: LagCompensationHistory::new(config)
                .expect("lag compensation config is valid"),
        }
    }

    /// Record the current combat state of all entities visible in a scene.
    pub fn record_tick(&mut self, tick: u64, snapshots: Vec<CombatSnapshot>) {
        let _ = self.inner.record(tick, snapshots);
    }

    /// Query whether an entity was at a given position at a historical tick.
    /// Returns the snapshot if found and within the rewind window.
    pub fn query_entity_at_tick(
        &mut self,
        target_tick: u64,
        entity_id: u32,
    ) -> Option<CombatSnapshot> {
        self.inner
            .with_rewind(target_tick, |_ctx, snapshots| {
                snapshots
                    .iter()
                    .find(|s| s.entity_id == entity_id)
                    .cloned()
            })
            .ok()
            .flatten()
    }

    #[allow(dead_code)]
    pub fn current_tick(&self) -> Option<u64> {
        self.inner.current_tick()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lag_history_records_and_rewinds() {
        let mut history = SceneLagHistory::new();

        let snap1 = vec![CombatSnapshot {
            entity_id: 1,
            pos: Position { map: 1, x: 10, y: 10 },
            hp: 100,
            dead: false,
        }];
        history.record_tick(1, snap1);

        let snap2 = vec![CombatSnapshot {
            entity_id: 1,
            pos: Position { map: 1, x: 15, y: 10 },
            hp: 80,
            dead: false,
        }];
        history.record_tick(2, snap2);

        let result = history.query_entity_at_tick(1, 1);
        assert!(result.is_some());
        let s = result.unwrap();
        assert_eq!(s.pos.x, 10);
        assert_eq!(s.hp, 100);

        let result2 = history.query_entity_at_tick(2, 1);
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().pos.x, 15);
    }

    #[test]
    fn lag_history_returns_none_for_unknown_entity() {
        let mut history = SceneLagHistory::new();
        history.record_tick(1, vec![CombatSnapshot {
            entity_id: 1,
            pos: Position { map: 1, x: 10, y: 10 },
            hp: 100,
            dead: false,
        }]);

        assert!(history.query_entity_at_tick(1, 999).is_none());
    }

    #[test]
    fn lag_history_rewind_limit() {
        let mut history = SceneLagHistory::new();
        for tick in 1..=50 {
            history.record_tick(tick, vec![]);
        }
        // Tick 10 is beyond rewind window (50 - 30 = 20 is oldest valid)
        assert!(history.query_entity_at_tick(10, 1).is_none());
    }
}
