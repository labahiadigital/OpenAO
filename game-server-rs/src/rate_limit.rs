use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;

/// Sliding-window rate limiter per connection.
///
/// Tracks the last `window_size` packet timestamps and rejects packets
/// that would exceed `max_packets_per_window`.
pub struct RateLimiter {
    timestamps: Vec<Instant>,
    max_packets: usize,
    window: std::time::Duration,
}

impl RateLimiter {
    pub fn new(max_packets_per_second: usize) -> Self {
        Self {
            timestamps: Vec::with_capacity(max_packets_per_second * 2),
            max_packets: max_packets_per_second,
            window: std::time::Duration::from_secs(1),
        }
    }

    /// Returns `true` if the packet should be allowed, `false` if rate-limited.
    pub fn check(&mut self) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window;
        self.timestamps.retain(|&t| t > cutoff);

        if self.timestamps.len() >= self.max_packets {
            return false;
        }

        self.timestamps.push(now);
        true
    }

    /// Returns the number of packets in the current window.
    #[allow(dead_code)]
    pub fn current_count(&self) -> usize {
        let now = Instant::now();
        let cutoff = now - self.window;
        self.timestamps.iter().filter(|&&t| t > cutoff).count()
    }
}

/// Per-command rate limiter for expensive operations.
/// Supports multiple named commands, each with its own cooldown.
pub struct CommandRateLimiter {
    last_at: HashMap<&'static str, Instant>,
    cooldown: std::time::Duration,
}

impl CommandRateLimiter {
    pub fn new(cooldown_ms: u64) -> Self {
        Self {
            last_at: HashMap::new(),
            cooldown: std::time::Duration::from_millis(cooldown_ms),
        }
    }

    /// Returns `true` if the named command should be allowed.
    pub fn check(&mut self, cmd: &'static str) -> bool {
        let now = Instant::now();
        if let Some(&last) = self.last_at.get(cmd)
            && now.duration_since(last) < self.cooldown
        {
            return false;
        }
        self.last_at.insert(cmd, now);
        true
    }
}

/// Global rate limiter by IP address.
/// Limits the total number of new connections per IP within a sliding window.
/// Thread-safe via DashMap — shared across all accept tasks.
pub struct IpRateLimiter {
    buckets: Arc<DashMap<IpAddr, Vec<Instant>>>,
    max_connections: usize,
    window: std::time::Duration,
}

impl IpRateLimiter {
    pub fn new(max_connections_per_window: usize, window_secs: u64) -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
            max_connections: max_connections_per_window,
            window: std::time::Duration::from_secs(window_secs),
        }
    }

    /// Returns `true` if a new connection from this IP should be allowed.
    pub fn check(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window;

        let mut entry = self.buckets.entry(ip).or_default();
        entry.retain(|&t| t > cutoff);

        if entry.len() >= self.max_connections {
            return false;
        }

        entry.push(now);
        true
    }

    /// Remove stale entries to prevent unbounded memory growth.
    #[allow(dead_code)]
    pub fn evict_stale(&self) {
        let now = Instant::now();
        let cutoff = now - self.window;
        self.buckets.retain(|_, timestamps| {
            timestamps.retain(|&t| t > cutoff);
            !timestamps.is_empty()
        });
    }

    #[allow(dead_code)]
    pub fn active_ips(&self) -> usize {
        self.buckets.len()
    }
}

impl Clone for IpRateLimiter {
    fn clone(&self) -> Self {
        Self {
            buckets: Arc::clone(&self.buckets),
            max_connections: self.max_connections,
            window: self.window,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_allows_within_limit() {
        let mut limiter = RateLimiter::new(10);
        for _ in 0..10 {
            assert!(limiter.check());
        }
    }

    #[test]
    fn rate_limiter_blocks_over_limit() {
        let mut limiter = RateLimiter::new(5);
        for _ in 0..5 {
            assert!(limiter.check());
        }
        assert!(!limiter.check());
    }

    #[test]
    fn command_rate_limiter_enforces_cooldown() {
        let mut limiter = CommandRateLimiter::new(1000);
        assert!(limiter.check("market"));
        assert!(!limiter.check("market"));
        // Different command is independent
        assert!(limiter.check("craft"));
    }

    #[test]
    fn ip_rate_limiter_allows_within_limit() {
        let limiter = IpRateLimiter::new(3, 60);
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
        assert!(!limiter.check(ip));
    }

    #[test]
    fn ip_rate_limiter_independent_ips() {
        let limiter = IpRateLimiter::new(2, 60);
        let ip1: IpAddr = "10.0.0.1".parse().unwrap();
        let ip2: IpAddr = "10.0.0.2".parse().unwrap();
        assert!(limiter.check(ip1));
        assert!(limiter.check(ip1));
        assert!(!limiter.check(ip1));
        assert!(limiter.check(ip2));
        assert!(limiter.check(ip2));
        assert!(!limiter.check(ip2));
    }

    #[test]
    fn ip_rate_limiter_evict_stale() {
        let limiter = IpRateLimiter::new(100, 60);
        let ip: IpAddr = "172.16.0.1".parse().unwrap();
        limiter.check(ip);
        assert_eq!(limiter.active_ips(), 1);
        limiter.evict_stale();
        assert_eq!(limiter.active_ips(), 1);
    }
}
