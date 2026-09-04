use std::cell::Ref;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{cell::RefCell, rc::Rc};

use std::time::Duration;

use crate::models::{Entry, Value};
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
    store: RefCell<HashMap<String, Entry>>,
    config: Rc<Config>,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        Engine {
            store: RefCell::new(HashMap::new()),
            config: Rc::new(config),
        }
    }

    pub fn put(&self, key: &str, value: Value, ttl: Option<Duration>) {
        let entry = Entry::new(value, ttl);

        self.store.borrow_mut().insert(key.to_string(), entry);
    }

    pub fn del(&self, key: &str) -> bool {
        self.store.borrow_mut().remove(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.store.borrow().len()
    }

    pub fn keys(&self, pattern: &str) -> Vec<String> {
        let store = self.store.borrow();
        let filtered_keys = store
            .keys()
            .filter(|k| k.starts_with(pattern))
            .cloned()
            .collect();
        filtered_keys
    }
}
