use crate::error::KvError;
use crate::models::Value;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{self, Sender};

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

pub struct AsyncWal {
    sender: Sender<WalOp>,
    handle: tokio::task::JoinHandle<()>,
}
impl AsyncWal {
    pub async fn new(path: PathBuf) -> Self {
        let (tx, mut rx) = mpsc::channel::<WalOp>(1024);
        let handle = tokio::spawn(async move {
            // 注意不能用 File::create —— 它会截断已有 WAL 历史；
            // 必须与同步版一致：create + append
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await
                .expect("打开 WAL 文件失败");

            while let Some(op) = rx.recv().await {
                let line = serde_json::to_string(&op).expect("WAL 序列化失败");

                file.write_all(format!("{line}\n").as_bytes())
                    .await
                    .expect("WAL 写入失败");

                file.flush().await.ok();
            }
        });
        Self { sender: tx, handle }
    }

    pub async fn append(&self, op: WalOp) -> Result<(), KvError> {
        self.sender.send(op).await.map_err(|_| KvError::WalClosed)
    }

    /// 优雅关闭：消费 self，先 drop sender 关闭 channel
    /// （recv 返回 None，后台循环结束），再等后台任务把剩余 op 排空落盘。
    /// 注意：不能用 Drop 实现同样的效果——Drop::drop 是同步上下文，
    /// 无法 await 后台任务；在 Drop 里创建 Future 不 await 等于什么都没做
    pub async fn close(self) {
        // ① 先关闭 channel（drop 最后一个 Sender）
        drop(self.sender);
        // ② 再等待后台任务退出：保证 buffer 中剩余 op 全部落盘
        let _ = self.handle.await;
    }
}
