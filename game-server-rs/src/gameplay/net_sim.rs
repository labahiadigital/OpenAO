//! Deterministic network simulation tests using Elura's `SimulatedLink`.
//! Validates game protocol resilience under adverse network conditions.

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use elura::gameplay::net_sim::{NetSimConfig, SendOutcome, SimulatedLink};

    #[derive(Debug, Clone, PartialEq)]
    struct GamePacket {
        opcode: u8,
        sequence: u32,
        payload: Vec<u8>,
    }

    fn make_movement_packet(seq: u32) -> GamePacket {
        GamePacket {
            opcode: 2,
            sequence: seq,
            payload: vec![1, 0, 50, 0, 50],
        }
    }

    fn make_attack_packet(seq: u32) -> GamePacket {
        GamePacket {
            opcode: 10,
            sequence: seq,
            payload: vec![],
        }
    }

    #[test]
    fn fixed_latency_delivers_after_delay() {
        let mut config = NetSimConfig::default();
        config.latency = Duration::from_millis(100);
        config.seed = 1;

        let mut link = SimulatedLink::new(config).unwrap();

        let pkt = make_movement_packet(1);
        let outcome = link.send(Duration::ZERO, 10, pkt.clone()).unwrap();
        assert!(matches!(outcome, SendOutcome::Queued { copies: 1, .. }));

        let empty = link.receive(Duration::from_millis(99)).unwrap();
        assert!(empty.is_empty());

        let delivered = link.receive(Duration::from_millis(100)).unwrap();
        assert_eq!(delivered.len(), 1);
        assert_eq!(delivered[0].payload, pkt);
    }

    #[test]
    fn total_packet_loss_drops_all() {
        let mut config = NetSimConfig::default();
        config.loss_rate = 1.0;
        config.seed = 42;

        let mut link = SimulatedLink::new(config).unwrap();

        for seq in 0..10 {
            let outcome = link.send(Duration::ZERO, 5, make_movement_packet(seq)).unwrap();
            assert_eq!(outcome, SendOutcome::DroppedByLoss);
        }

        let stats = link.stats();
        assert_eq!(stats.packets_sent, 10);
        assert_eq!(stats.packets_lost, 10);
        assert_eq!(stats.packets_delivered, 0);
    }

    #[test]
    fn partial_loss_drops_some_packets() {
        let mut config = NetSimConfig::default();
        config.loss_rate = 0.5;
        config.seed = 123;

        let mut link = SimulatedLink::new(config).unwrap();

        for seq in 0..100 {
            link.send(Duration::ZERO, 5, make_movement_packet(seq)).unwrap();
        }

        let delivered = link.receive(Duration::from_secs(10)).unwrap();
        let stats = link.stats();

        assert!(stats.packets_lost > 0, "some should be lost");
        assert!(delivered.len() < 100, "not all should arrive");
        assert!(delivered.len() > 0, "some should arrive");
    }

    #[test]
    fn reordering_changes_delivery_order() {
        let mut config = NetSimConfig::default();
        config.latency = Duration::from_millis(50);
        config.reorder_rate = 1.0;
        config.max_reorder_delay = Duration::from_millis(100);
        config.seed = 7;

        let mut link = SimulatedLink::new(config).unwrap();

        for seq in 0..20u32 {
            link.send(Duration::ZERO, 5, make_movement_packet(seq)).unwrap();
        }

        let delivered = link.receive(Duration::from_secs(1)).unwrap();
        assert_eq!(delivered.len(), 20);

        let sequences: Vec<u32> = delivered.iter().map(|d| d.payload.sequence).collect();
        let is_sorted = sequences.windows(2).all(|w| w[0] <= w[1]);
        assert!(!is_sorted, "with 100% reorder rate, packets should arrive out of order");
    }

    #[test]
    fn queue_overflow_drops_excess() {
        let mut config = NetSimConfig::default();
        config.latency = Duration::from_secs(10);
        config.max_queued_packets = 5;
        config.seed = 1;

        let mut link = SimulatedLink::new(config).unwrap();

        for seq in 0..5u32 {
            let outcome = link.send(Duration::ZERO, 10, make_attack_packet(seq)).unwrap();
            assert!(matches!(outcome, SendOutcome::Queued { .. }));
        }

        let overflow = link.send(Duration::ZERO, 10, make_attack_packet(5)).unwrap();
        assert_eq!(overflow, SendOutcome::DroppedByQueue);

        let stats = link.stats();
        assert_eq!(stats.packets_queue_dropped, 1);
        assert_eq!(link.queued_packets(), 5);
    }

    #[test]
    fn bandwidth_throttling_serializes_sequentially() {
        let mut config = NetSimConfig::default();
        config.bandwidth_bytes_per_second = 100;
        config.seed = 1;

        let mut link = SimulatedLink::new(config).unwrap();

        link.send(Duration::ZERO, 100, make_movement_packet(1)).unwrap();
        link.send(Duration::ZERO, 100, make_movement_packet(2)).unwrap();

        let at_999ms = link.receive(Duration::from_millis(999)).unwrap();
        assert_eq!(at_999ms.len(), 0);

        let at_1s = link.receive(Duration::from_secs(1)).unwrap();
        assert_eq!(at_1s.len(), 1);
        assert_eq!(at_1s[0].payload.sequence, 1);

        let at_2s = link.receive(Duration::from_secs(2)).unwrap();
        assert_eq!(at_2s.len(), 1);
        assert_eq!(at_2s[0].payload.sequence, 2);
    }

    #[test]
    fn deterministic_replay_produces_identical_results() {
        let mut config = NetSimConfig::default();
        config.latency = Duration::from_millis(50);
        config.jitter = Duration::from_millis(20);
        config.loss_rate = 0.1;
        config.reorder_rate = 0.3;
        config.max_reorder_delay = Duration::from_millis(60);
        config.seed = 999;

        let run = |_name: &str| -> Vec<GamePacket> {
            let mut link = SimulatedLink::new(config).unwrap();
            for seq in 0..50 {
                let now = Duration::from_millis(seq * 16);
                link.send(now, 20, make_movement_packet(seq as u32)).unwrap();
            }
            link.receive(Duration::from_secs(5))
                .unwrap()
                .into_iter()
                .map(|d| d.payload)
                .collect()
        };

        let first = run("run1");
        let second = run("run2");
        assert_eq!(first, second, "identical seeds must produce identical results");
        assert!(!first.is_empty());
    }

    #[test]
    fn redundant_input_survives_partial_loss() {
        let mut config = NetSimConfig::default();
        config.loss_rate = 0.5;
        config.seed = 42;

        let mut link = SimulatedLink::new(config).unwrap();

        let redundancy = 3;
        for tick in 0..20u32 {
            for redundant_copy in 0..redundancy {
                let _ = link.send(
                    Duration::from_millis(tick as u64 * 16),
                    10,
                    make_movement_packet(tick * 100 + redundant_copy),
                );
            }
        }

        let delivered = link.receive(Duration::from_secs(10)).unwrap();
        let unique_ticks: std::collections::HashSet<u32> = delivered
            .iter()
            .map(|d| d.payload.sequence / 100)
            .collect();

        assert!(
            unique_ticks.len() > 10,
            "redundancy should recover most ticks despite 50% loss; got {} unique ticks",
            unique_ticks.len()
        );
    }

    #[test]
    fn jitter_varies_delivery_times() {
        let mut config = NetSimConfig::default();
        config.latency = Duration::from_millis(100);
        config.jitter = Duration::from_millis(40);
        config.seed = 55;

        let mut link = SimulatedLink::new(config).unwrap();

        for seq in 0..30u32 {
            link.send(Duration::ZERO, 5, make_movement_packet(seq)).unwrap();
        }

        let delivered = link.receive(Duration::from_secs(1)).unwrap();
        let times: Vec<u128> = delivered.iter().map(|d| d.delivered_at.as_millis()).collect();

        let min = *times.iter().min().unwrap();
        let max = *times.iter().max().unwrap();
        assert!(
            max - min > 10,
            "jitter should spread delivery times; range was {}ms",
            max - min
        );
    }
}
