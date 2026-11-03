use std::{any::Any, sync::mpsc::TrySendError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("网络请求失败：{0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON 解析失败: {0}")]
    Json(#[from] serde_json::Error),

    #[error("无效的股票代码: {0}")]
    InvalidCode(String),

    #[error("接口返回异常: {0}")]
    Api(String),

    #[error("进程异常: {0}")]
    ProcessError(String),
}

pub type Result<T> = std::result::Result<T, DataError>;
