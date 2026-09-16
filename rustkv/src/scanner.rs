use crate::{error::KvError, models::Entry, models::Value};
use std::collections::HashMap;
use std::{
    sync::{Arc, Mutex},
};

pub struct ScanIterator {
    items: Vec<(String, Value)>,
    index: usize,
}

impl ScanIterator {
    pub fn new(store: Arc<Mutex<HashMap<String, Entry>>>, prefix: &str) -> Result<Self, KvError> {
        let store = store.lock()?;

        let mut items: Vec<(String, Value)> = store
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .filter(|(_, v)| !v.is_expired())
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(Self { items, index: 0 })
    }
}

impl Iterator for ScanIterator {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.items.len() {
            let item = self.items[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
