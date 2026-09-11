use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use std::time::Duration;

use crate::error::KvError;
use crate::models::{Entry, Value};
use crate::ttl::TtlManager;
use crate::wal_tokio::{AsyncWal, WalOp};
use dirs::home_dir;

pub struct Config {
    pub data_dir: PathBuf,
    pub wal_enabled: bool,
    pub ttl_check_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data_dir: home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".rustkv"),
            wal_enabled: true,
            ttl_check_interval: Duration::from_secs(1),
        }
    }
}

pub struct Engine {
    store: Arc<Mutex<HashMap<String, Entry>>>,
    // Phase 后续（TTL 后台清理、快照/恢复）将使用配置；当前暂未被读取
    #[allow(dead_code)]
    config: Arc<Config>,
    wal: Option<AsyncWal>,
    ttl: TtlManager,
}

impl Engine {
    pub async fn new(config: Config) -> Result<Self, KvError> {
        let wal = if config.wal_enabled {
            std::fs::create_dir_all(&config.data_dir)?;
            Some(AsyncWal::new(config.data_dir.join("wal.log")).await)
        } else {
            None
        };
        let store = Arc::new(Mutex::new(HashMap::new()));
        let ttl = TtlManager::spawn(Arc::clone(&store), config.ttl_check_interval);
        Ok(Engine {
            store,
            config: Arc::new(config),
            wal,
            ttl,
        })
    }

    pub async fn put(&self, key: &str, value: Value, ttl: Option<Duration>) -> Result<(), KvError> {
        if let Some(w) = &self.wal {
            let op = WalOp::Put {
                key: key.to_string(),
                value: value.clone(),
                ttl_secs: ttl.map(|d| d.as_secs()),
            };
            w.append(op).await?;
        }
        let entry = Entry::new(value, ttl);
        let mut store = self.store.lock()?;
        store.insert(key.to_string(), entry);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Value>, KvError> {
        let store = self.store.lock()?;
        let res = store.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.value.clone())
            }
        });
        Ok(res)
    }
    pub async fn del(&self, key: &str) -> Result<bool, KvError> {
        if let Some(w) = &self.wal {
            let op = WalOp::Del {
                key: key.to_string(),
            };
            w.append(op).await?;
        }
        let mut store = self.store.lock()?;

        Ok(store.remove(key).is_some())
    }

    pub fn len(&self) -> Result<usize, KvError> {
        let store = self.store.lock()?;

        Ok(store.len())
    }

    pub fn keys(&self, pattern: &str) -> Result<Vec<String>, KvError> {
        let store = self.store.lock()?;
        let filtered_keys = store
            .keys()
            .filter(|k| k.starts_with(pattern))
            .cloned()
            .collect();
        Ok(filtered_keys)
    }
    pub async fn concurrent_put(
        engine: &Engine,
        entries: Vec<(String, Value)>,
    ) -> Result<Vec<tokio::task::JoinHandle<()>>, KvError> {
        let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::with_capacity(entries.len());
        for (key, value) in entries {
            if let Some(wal) = &engine.wal {
                wal.append(WalOp::Put {
                    key: key.clone(),
                    value: value.clone(),
                    ttl_secs: None,
                })
                .await?;
            }
            let store = Arc::clone(&engine.store);
            let handle = tokio::task::spawn(async move {
                let mut s = store.lock().unwrap();
                s.insert(key, Entry::new(value, None));
            });
            handles.push(handle);
        }
        Ok(handles)
    }

    pub async fn close(self) {
        if let Some(wal) = self.wal {
            wal.close().await;
        }
        self.ttl.shutdown().await;
    }
}

#[cfg(test)]
pub mod tests {
    use crate::engine::{Config, Engine};
    use crate::models::Value;
    use std::time::Duration;

    #[tokio::test]
    async fn test_engine() {
        let temp_dir = std::env::temp_dir().join("rustkv.test");
        let config = Config {
            data_dir: temp_dir.clone(),
            wal_enabled: true,
            ttl_check_interval: Duration::from_secs(1),
        };
        let engine = Engine::new(config).await.unwrap();
        engine
            .put("name", Value::from("RustKV"), None)
            .await
            .unwrap();
        assert_eq!(
            engine.get("name").unwrap(),
            Some(Value::String("RustKV".into()))
        );
        engine.put("count", Value::from(42i64), None).await.unwrap();
        engine.del("name").await.unwrap();
        assert_eq!(engine.len().unwrap(), 1);
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_concurrent_put_with_wal() {
        let temp_dir = std::env::temp_dir().join("rustkv.test.concurrent");
        let config = Config {
            data_dir: temp_dir.clone(),
            wal_enabled: true,
            ttl_check_interval: Duration::from_secs(1),
        };
        let engine = Engine::new(config).await.unwrap();
        let entries: Vec<(String, Value)> = (0..10)
            .map(|i| (format!("key{i}"), Value::from(i as i64)))
            .collect();
        let handles = Engine::concurrent_put(&engine, entries).await.unwrap();
        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(engine.len().unwrap(), 10);
        engine.close().await;
        // WAL 已记录（主线程串行 append），文件应存在且非空
        let wal_path = temp_dir.join("wal.log");
        assert!(wal_path.exists());
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
