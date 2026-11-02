use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub mod cache_prefix {
    pub const ALL_A_CODES: &str = "ALL_A_CODES";
}
#[derive(Debug)]
struct Entry {
    at: Instant,
    ttl: Duration,
    value: Arc<dyn Any + Send + Sync>,
}

#[derive(Debug)]
pub struct AnyTtlCach {
    entries: Mutex<HashMap<String, Entry>>,
}

impl AnyTtlCach {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get<V: Any + Send + Sync>(&self, key: &str) -> Option<Arc<V>> {
        let mut map = self.entries.lock().await;
        let entry = map.get(key)?;
        if entry.at.elapsed() >= entry.ttl {
            map.remove(key);
            return None;
        }
        entry.value.clone().downcast::<V>().ok()
    }

    pub async fn insert<V: Any + Send + Sync>(&self, key: &str, value: V, ttl: Duration) -> Arc<V> {
        let arc_val = Arc::new(value);
        // self.insert_arc(key, arc_val, ttl).await;
        self.entries.lock().await.insert(
            key.to_string(),
            Entry {
                at: Instant::now(),
                ttl,
                value: arc_val.clone(),
            },
        );
        arc_val.clone()
    }

    pub async fn insert_arc(&self, key: &str, value: Arc<dyn Any + Send + Sync>, ttl: Duration) {
        self.entries.lock().await.insert(
            key.to_string(),
            Entry {
                at: Instant::now(),
                ttl,
                value,
            },
        );
    }
    pub async fn remove(&self, key: &str) {
        self.entries.lock().await.remove(key);
    }
    pub async fn clear(&self) {
        self.entries.lock().await.clear();
    }
}
