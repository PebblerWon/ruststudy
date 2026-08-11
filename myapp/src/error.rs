use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("任务不存在：{0}")]
    NotFound(String),

    #[error("标题不能为空")]
    EmptyTitle,

    #[error("标题长度不能超过 100 个字符")]
    TitleTooLong,

    #[error("日期格式错误，请使用 YYYY-MM-DD 格式")]
    InvalidDate,

    #[error("ID 匹配到多个任务：{0}")]
    AmbiguousId(String),

    #[error("数据文件读取失败：{0}")]
    StoreLoadError(#[from] std::io::Error),

    #[error("数据解析失败：{0}")]
    ParseError(#[from] serde_json::Error),

    #[error("不支持的导出格式：{0}")]
    UnsupportedFormat(String),

    #[error("无法获取父目录")]
    HomeDirNotFound,

    #[error("最多支持10条标签")]
    TooManyTags,
}
