use crate::error::KvError;
use crate::models::Value;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalOp {
    Put {
        key: String,
        value: Value,
        ttl_secs: Option<u64>,
    },
    Del {
        key: String,
    },
}

pub struct Wal {
    sender: Option<Sender<WalOp>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Wal {
    pub fn new(path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel::<WalOp>();
        let handle = thread::spawn(move || {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .expect("打开wal文件失败");
            for op in rx {
                let line = serde_json::to_string(&op).expect("WAL 序列化失败");
                writeln!(file, "{line}").expect("WAL 写入失败");
            }
        });
        Self {
            sender: Some(tx),
            handle: Some(handle),
        }
    }
    pub fn append(&self, op: WalOp) -> Result<(), KvError> {
        self.sender
            .as_ref()
            .ok_or(KvError::WalClosed)?
            .send(op)
            .map_err(|_| KvError::WalClosed)
    }
}

impl Drop for Wal {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
