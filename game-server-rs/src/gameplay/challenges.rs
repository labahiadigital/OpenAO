use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::world::EntityId;

#[derive(Debug, Clone)]
pub struct ChallengeParticipant {
    pub entity_id: EntityId,
    pub character_id: Uuid,
    pub name: String,
    pub level: i32,
    pub class_name: String,
    pub race_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamSize {
    Solo = 1,
    Duo = 2,
}

#[derive(Debug, Clone)]
pub struct Challenge {
    pub id: Uuid,
    pub created_at: i64,
    pub team_size: TeamSize,
    pub proposer: ChallengeParticipant,
    pub participants: Vec<ChallengeParticipant>,
}

pub struct ChallengeManager {
    active_challenges: HashMap<Uuid, Challenge>,
}

impl ChallengeManager {
    pub fn new() -> Self {
        Self {
            active_challenges: HashMap::new(),
        }
    }

    pub fn create_challenge(
        &mut self,
        proposer: ChallengeParticipant,
        team_size: TeamSize,
    ) -> &Challenge {
        let id = Uuid::new_v4();
        let challenge = Challenge {
            id,
            created_at: Utc::now().timestamp_millis(),
            team_size,
            proposer: proposer.clone(),
            participants: vec![proposer],
        };
        self.active_challenges.insert(id, challenge);
        self.active_challenges.get(&id).unwrap()
    }

    pub fn join_challenge(
        &mut self,
        challenge_id: Uuid,
        participant: ChallengeParticipant,
    ) -> Result<(), &'static str> {
        let challenge = self
            .active_challenges
            .get_mut(&challenge_id)
            .ok_or("Challenge not found")?;

        let max = challenge.team_size as usize * 2;
        if challenge.participants.len() >= max {
            return Err("Challenge is full");
        }

        if challenge
            .participants
            .iter()
            .any(|p| p.entity_id == participant.entity_id)
        {
            return Err("Already in this challenge");
        }

        challenge.participants.push(participant);
        Ok(())
    }

    pub fn remove_challenge(&mut self, id: Uuid) -> Option<Challenge> {
        self.active_challenges.remove(&id)
    }

    pub fn list_challenges(&self) -> Vec<&Challenge> {
        self.active_challenges.values().collect()
    }

    pub fn is_ready(&self, challenge_id: Uuid) -> bool {
        self.active_challenges
            .get(&challenge_id)
            .map(|c| c.participants.len() == (c.team_size as usize * 2))
            .unwrap_or(false)
    }
}

impl Default for ChallengeManager {
    fn default() -> Self {
        Self::new()
    }
}
