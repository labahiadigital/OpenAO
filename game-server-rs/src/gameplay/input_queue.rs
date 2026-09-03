use elura::gameplay::netcode::{
    InputAck, InputPacket, InputReceiveReport, InputReceiver, InputReceiverConfig,
};
use serde::{Deserialize, Serialize};

/// Application-level game input sent by a client for a specific tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum GameInput {
    Move { heading: u8 },
    ChangeHeading { heading: u8 },
    AttackMelee,
    AttackRanged,
    AttackSpell { slot: u8 },
    UseItem { slot: u8 },
}

/// Per-player server-side input receiver wrapping Elura's InputReceiver.
/// Validates, de-duplicates, and reorders client inputs by sequence.
pub struct PlayerInputReceiver {
    inner: InputReceiver,
    current_tick: u64,
}

impl PlayerInputReceiver {
    pub fn new() -> Self {
        let mut config = InputReceiverConfig::default();
        config.max_inputs_per_packet = 16;
        config.reorder_window = 256;
        config.max_past_ticks = 12;
        config.max_future_ticks = 120;
        Self {
            inner: InputReceiver::new(config).expect("valid input receiver config"),
            current_tick: 0,
        }
    }

    pub fn set_tick(&mut self, tick: u64) {
        self.current_tick = tick;
    }

    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    #[allow(dead_code)]
    pub fn receive(
        &mut self,
        packet: InputPacket<GameInput>,
    ) -> Result<InputReceiveReport<GameInput>, String> {
        self.inner
            .receive(self.current_tick, packet)
            .map_err(|e| format!("input receive error: {e:?}"))
    }

    #[allow(dead_code)]
    pub fn build_ack(&self) -> InputAck {
        InputAck {
            server_tick: self.current_tick,
            acknowledged_sequence: self.inner.acknowledged_sequence(),
        }
    }

    #[allow(dead_code)]
    pub fn acknowledged_sequence(&self) -> u64 {
        self.inner.acknowledged_sequence()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elura::gameplay::netcode::InputFrame;

    #[test]
    fn receive_accepts_valid_input() {
        let mut receiver = PlayerInputReceiver::new();
        receiver.set_tick(10);

        let packet = InputPacket {
            client_tick: 10,
            acknowledged_server_tick: 0,
            inputs: vec![InputFrame {
                sequence: 1,
                target_tick: 10,
                input: GameInput::Move { heading: 1 },
            }],
        };

        let report = receiver.receive(packet).unwrap();
        assert_eq!(report.accepted.len(), 1);
        assert_eq!(report.duplicates, 0);
        assert_eq!(report.acknowledgement.acknowledged_sequence, 1);
    }

    #[test]
    fn receive_deduplicates_redundant_inputs() {
        let mut receiver = PlayerInputReceiver::new();
        receiver.set_tick(10);

        let packet1 = InputPacket {
            client_tick: 10,
            acknowledged_server_tick: 0,
            inputs: vec![InputFrame {
                sequence: 1,
                target_tick: 10,
                input: GameInput::AttackMelee,
            }],
        };
        receiver.receive(packet1).unwrap();

        let packet2 = InputPacket {
            client_tick: 11,
            acknowledged_server_tick: 10,
            inputs: vec![
                InputFrame {
                    sequence: 1,
                    target_tick: 10,
                    input: GameInput::AttackMelee,
                },
                InputFrame {
                    sequence: 2,
                    target_tick: 11,
                    input: GameInput::Move { heading: 2 },
                },
            ],
        };

        receiver.set_tick(11);
        let report = receiver.receive(packet2).unwrap();
        assert_eq!(report.accepted.len(), 1);
        assert_eq!(report.duplicates, 1);
        assert_eq!(report.acknowledgement.acknowledged_sequence, 2);
    }

    #[test]
    fn receive_rejects_future_server_tick_ack() {
        let mut receiver = PlayerInputReceiver::new();
        receiver.set_tick(5);

        let packet = InputPacket {
            client_tick: 10,
            acknowledged_server_tick: 100,
            inputs: vec![InputFrame {
                sequence: 1,
                target_tick: 5,
                input: GameInput::Move { heading: 3 },
            }],
        };

        assert!(receiver.receive(packet).is_err());
    }

    #[test]
    fn acknowledged_sequence_tracks_progress() {
        let mut receiver = PlayerInputReceiver::new();
        receiver.set_tick(10);
        assert_eq!(receiver.acknowledged_sequence(), 0);

        let packet = InputPacket {
            client_tick: 10,
            acknowledged_server_tick: 0,
            inputs: vec![
                InputFrame { sequence: 1, target_tick: 10, input: GameInput::AttackMelee },
                InputFrame { sequence: 2, target_tick: 10, input: GameInput::AttackRanged },
            ],
        };

        let report = receiver.receive(packet).unwrap();
        assert_eq!(report.acknowledgement.acknowledged_sequence, 2);
        assert_eq!(receiver.acknowledged_sequence(), 2);
    }
}
