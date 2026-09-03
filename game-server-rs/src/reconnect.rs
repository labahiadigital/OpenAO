use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use uuid::Uuid;

const TOKEN_TTL: Duration = Duration::from_secs(120);
const MAX_TOKENS: usize = 10_000;

/// Captures the state needed to restore a player session after reconnect.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ReconnectState {
    pub account_id: String,
    pub character_id: String,
    pub character_name: String,
    pub entity_id: u32,
    pub map_id: i32,
}

#[allow(dead_code)]
struct TokenEntry {
    token: String,
    state: ReconnectState,
    created_at: Instant,
}

/// Manages short-lived reconnect tokens that allow a client to resume
/// a disconnected session without re-authenticating via the HTTP API.
pub struct ReconnectManager {
    tokens: Mutex<HashMap<String, TokenEntry>>,
}

impl ReconnectManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            tokens: Mutex::new(HashMap::new()),
        })
    }

    /// Issues a new reconnect token for the given session state.
    /// The token is valid for `TOKEN_TTL` seconds.
    pub fn issue_token(&self, state: ReconnectState) -> String {
        let token = Uuid::new_v4().to_string();
        let Ok(mut map) = self.tokens.lock() else {
            tracing::error!("ReconnectManager mutex poisoned in issue_token");
            return token;
        };

        if map.len() > MAX_TOKENS {
            self.evict_expired_inner(&mut map);
        }

        map.insert(token.clone(), TokenEntry {
            token: token.clone(),
            state,
            created_at: Instant::now(),
        });

        token
    }

    /// Consumes a reconnect token, returning the stored session state
    /// if the token exists and hasn't expired.
    pub fn consume_token(&self, token: &str) -> Option<ReconnectState> {
        let Ok(mut map) = self.tokens.lock() else {
            tracing::error!("ReconnectManager mutex poisoned in consume_token");
            return None;
        };
        let entry = map.remove(token)?;

        if entry.created_at.elapsed() > TOKEN_TTL {
            return None;
        }

        Some(entry.state)
    }

    /// Removes all expired tokens. Called periodically from the game loop.
    pub fn evict_expired(&self) {
        let Ok(mut map) = self.tokens.lock() else {
            tracing::error!("ReconnectManager mutex poisoned in evict_expired");
            return;
        };
        self.evict_expired_inner(&mut map);
    }

    fn evict_expired_inner(&self, map: &mut HashMap<String, TokenEntry>) {
        map.retain(|_, entry| entry.created_at.elapsed() <= TOKEN_TTL);
    }

    /// Returns the number of active (non-expired) reconnect tokens.
    pub fn active_count(&self) -> usize {
        let Ok(map) = self.tokens.lock() else {
            return 0;
        };
        map.values()
            .filter(|e| e.created_at.elapsed() <= TOKEN_TTL)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_and_consume_token() {
        let mgr = ReconnectManager::new();

        let state = ReconnectState {
            account_id: "acc-1".into(),
            character_id: "char-1".into(),
            character_name: "TestPlayer".into(),
            entity_id: 42,
            map_id: 1,
        };

        let token = mgr.issue_token(state.clone());
        assert!(!token.is_empty());

        let restored = mgr.consume_token(&token);
        assert!(restored.is_some());

        let restored = restored.unwrap();
        assert_eq!(restored.account_id, "acc-1");
        assert_eq!(restored.entity_id, 42);
    }

    #[test]
    fn consume_token_is_single_use() {
        let mgr = ReconnectManager::new();

        let state = ReconnectState {
            account_id: "acc-1".into(),
            character_id: "char-1".into(),
            character_name: "Test".into(),
            entity_id: 1,
            map_id: 1,
        };

        let token = mgr.issue_token(state);
        assert!(mgr.consume_token(&token).is_some());
        assert!(mgr.consume_token(&token).is_none());
    }

    #[test]
    fn invalid_token_returns_none() {
        let mgr = ReconnectManager::new();
        assert!(mgr.consume_token("nonexistent").is_none());
    }
}
