use std::time::{Duration, Instant};

use super::value::Value;

#[derive(Debug, Clone)]
pub struct Entry {
    pub value: Value,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
}

impl Entry {
    pub fn new(value: Value, ttl: Option<Duration>) -> Self {
        let now = Instant::now();
        let expires_at = ttl.map(|d| now + d);
        Entry {
            value,
            created_at: now,
            expires_at: expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            None => false,
            Some(e) => e < Instant::now(),
        }
    }

    pub fn is_expired_at(&self, now: Instant) -> bool {
        self.expires_at.map_or(false, |d| d < now)
    }

    pub fn age(&self) -> Duration {
        let a = Instant::now() - self.created_at;
        a
    }
}
