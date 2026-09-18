use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuantError {
    /// 网络连接错误（DNS 解析失败、超时、连接重置等）
    #[error("网络错误: {0}")]
    Network(String),

    /// Binance API 返回错误（HTTP 非 2xx、频率限制等）
    #[error("API 错误: {0}")]
    Api(String),

    /// 数据解析错误（JSON 反序列化失败、字段类型不匹配等）
    #[error("解析错误: {0}")]
    Parse(String),

    /// 文件 IO 错误（缓存读写失败、目录创建失败等）
    #[error("IO 错误: {0}")]
    Io(String),

    /// polars 数据处理错误（DataFrame 操作失败、Parquet 读写异常等）
    #[error("数据处理错误: {0}")]
    DataProcess(String),
}

impl From<polars::error::PolarsError> for QuantError {
    fn from(err: polars::error::PolarsError) -> Self {
        QuantError::DataProcess(err.to_string())
    }
}
