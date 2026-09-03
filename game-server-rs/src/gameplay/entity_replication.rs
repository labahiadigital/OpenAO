use bytes::Bytes;
use elura::gameplay::replication::{
    ReplicationAck, ReplicationBatch, ReplicationConfig, ReplicationPacket, ReplicationSender,
    VersionedState,
};

/// Entity state serialized as opaque bytes for transport.
pub type EntityState = Bytes;
/// Delta between two entity states, also opaque bytes.
pub type EntityDelta = Bytes;
/// Entity identifier in the replication stream.
pub type EntityId = u32;
/// Full versioned state for an entity.
pub type EntityVersionedState = VersionedState<EntityState>;

/// Per-observer replication tracker.
/// Wraps Elura's `ReplicationSender` to track which entities each
/// client has been told about, enabling efficient spawn/despawn/delta.
///
/// Maintains a `broadcast_announced` set to skip Spawn events for entities
/// that were already announced via `broadcast_in_range` in the same tick window,
/// avoiding redundant character packets.
pub struct ObserverReplicator {
    inner: ReplicationSender<EntityId, EntityState, EntityDelta>,
    tick: u64,
    /// Entities announced via direct broadcast since last reset.
    /// Cleared every time reset_broadcast_announced() is called (once per second from game loop).
    broadcast_announced: std::collections::HashSet<EntityId>,
}

impl ObserverReplicator {
    pub fn new() -> Self {
        let config = ReplicationConfig::default();
        Self {
            inner: ReplicationSender::new(config)
                .expect("default replication config is valid"),
            tick: 0,
            broadcast_announced: std::collections::HashSet::new(),
        }
    }

    /// Reconcile the full visible set for this observer at a given tick.
    /// `visible` is an iterator of (entity_id, versioned_state).
    /// Returns the number of batches queued.
    pub fn reconcile(
        &mut self,
        tick: u64,
        visible: impl IntoIterator<Item = (EntityId, EntityVersionedState)>,
    ) -> usize {
        self.tick = tick;
        self.inner
            .update(tick, visible, |_entity, _old, _new| -> Option<EntityDelta> {
                // For now, always send keyframes (full state).
                // Delta encoding can be added later by comparing old/new states.
                None
            })
            .unwrap_or(0)
    }

    /// Build a packet containing redundant unacknowledged batches.
    pub fn build_packet(&self) -> ReplicationPacket<EntityId, EntityState, EntityDelta> {
        self.inner.packet()
    }

    /// Apply client acknowledgment of received batch.
    pub fn acknowledge(&mut self, ack: ReplicationAck) -> usize {
        self.inner.acknowledge(ack).unwrap_or(0)
    }

    /// Force full keyframes on the next reconcile (e.g. after reconnect).
    pub fn force_keyframes(&mut self) {
        self.inner.force_keyframes();
    }

    /// Number of entities projected as visible after pending batches.
    pub fn projected_count(&self) -> usize {
        self.inner.projected_entities()
    }

    /// Number of unacknowledged batches.
    pub fn pending_batches(&self) -> usize {
        self.inner.pending_batches()
    }

    /// Reset stream state (on reconnect or full resync).
    pub fn reset(&mut self) {
        self.inner.reset();
        self.tick = 0;
        self.broadcast_announced.clear();
    }

    /// Mark an entity as already announced via broadcast (prevents duplicate Spawn).
    pub fn mark_broadcast_announced(&mut self, entity_id: EntityId) {
        self.broadcast_announced.insert(entity_id);
    }

    /// Check if a Spawn event should be suppressed because the entity was already
    /// announced via broadcast_in_range.
    pub fn is_broadcast_announced(&self, entity_id: EntityId) -> bool {
        self.broadcast_announced.contains(&entity_id)
    }

    /// Clear the broadcast-announced set (called periodically from game loop).
    pub fn reset_broadcast_announced(&mut self) {
        self.broadcast_announced.clear();
    }
}

/// Helper to create a versioned state for replication.
pub fn make_versioned_state(version: u64, data: Vec<u8>) -> EntityVersionedState {
    VersionedState {
        version,
        prediction_key: None,
        state: Bytes::from(data),
    }
}

/// Type alias for the batch type used by our replication system.
pub type GameReplicationBatch = ReplicationBatch<EntityId, EntityState, EntityDelta>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observer_replicator_lifecycle() {
        let mut replicator = ObserverReplicator::new();
        assert_eq!(replicator.projected_count(), 0);
        assert_eq!(replicator.pending_batches(), 0);

        let state = make_versioned_state(1, b"entity_data".to_vec());
        let visible = vec![(42u32, state)];
        let batches = replicator.reconcile(1, visible);
        assert!(batches > 0);
        assert_eq!(replicator.projected_count(), 1);
        assert!(replicator.pending_batches() > 0);
    }

    #[test]
    fn observer_replicator_despawn() {
        let mut replicator = ObserverReplicator::new();

        let state = make_versioned_state(1, b"data".to_vec());
        replicator.reconcile(1, vec![(1, state)]);
        assert_eq!(replicator.projected_count(), 1);

        // Empty visible set causes despawn
        replicator.reconcile(2, vec![]);
        assert_eq!(replicator.projected_count(), 0);
    }

    #[test]
    fn observer_replicator_acknowledge() {
        let mut replicator = ObserverReplicator::new();

        let state = make_versioned_state(1, b"data".to_vec());
        replicator.reconcile(1, vec![(1, state)]);

        let packet = replicator.build_packet();
        assert!(!packet.batches.is_empty());

        let ack = ReplicationAck {
            acknowledged_sequence: packet.batches[0].sequence,
            applied_tick: packet.batches[0].tick,
        };
        let released = replicator.acknowledge(ack);
        assert!(released > 0);
        assert_eq!(replicator.pending_batches(), 0);
    }

    #[test]
    fn observer_replicator_reset() {
        let mut replicator = ObserverReplicator::new();

        let state = make_versioned_state(1, b"data".to_vec());
        replicator.reconcile(1, vec![(1, state)]);
        assert_eq!(replicator.projected_count(), 1);

        replicator.reset();
        assert_eq!(replicator.projected_count(), 0);
        assert_eq!(replicator.pending_batches(), 0);
    }
}
