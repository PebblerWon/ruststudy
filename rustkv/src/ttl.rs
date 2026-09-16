use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::mpsc;

use crate::models::Entry;

pub struct TtlManager {
    shutdown: mpsc::Sender<()>,
}

impl TtlManager {
    pub fn spawn(store: Arc<Mutex<HashMap<String, Entry>>>, check_interval: Duration) -> Self {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(check_interval)=>{
                        let Ok(mut store) = store.lock() else {break};

                        store.retain(|_, entry|{
                            !entry.is_expired()
                        });
                    }

                    _= shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });
        Self {
            shutdown: shutdown_tx,
        }
    }

    pub async fn shutdown(self) {
        let _ = self.shutdown.send(()).await;
    }
}
