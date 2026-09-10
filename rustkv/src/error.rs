use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvError {
    #[error("键不存在：{0}")]
    KeyNotFound(String),

    #[error("类型不匹配：{left} + {right}")]
    TypeMismatch {
        left: &'static str,
        right: &'static str,
    },

    #[error("WAL 已关闭")]
    WalClosed,

    #[error("存储锁中毒")]
    LockPoisoned,

    #[error("IO 错误：{0}")]
    IoError(#[from] std::io::Error),

    #[error("序列化错误：{0}")]
    SerializeError(#[from] serde_json::Error),

    #[error("数据损坏：{0}")]
    CorruptedData(String),
}

impl<T> From<std::sync::PoisonError<T>> for KvError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        KvError::LockPoisoned
    }
}
