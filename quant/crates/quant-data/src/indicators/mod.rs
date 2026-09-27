mod ema;
mod sma;
#[derive(Debug, Clone)]

/// 指标计算结果
/// 支持单值指标（SMA/EMA/RSI）和多值指标（MACD/布林带）
pub enum IndicatorOutput {
    /// 单值序列（每个时间点对应一个 f64）
    /// 长度与输入相同，前 period-1 个值为 f64::NAN
    Single(Vec<f64>),

    /// 多值序列（每个时间点对应多个命名值）
    /// 例如 MACD → {"macd_line", "signal_line", "histogram"}
    /// 例如 Bollinger → {"upper", "middle", "lower"}
    Multi {
        names: Vec<String>,
        /// 每个 name 对应一个 Vec<f64>，长度与输入相同
        values: Vec<Vec<f64>>,
    },
}

/// 技术指标 trait — 所有指标的公共接口
/// Send + Sync 约束确保指标可在多线程环境安全使用（如回测、实时计算）
pub trait Indicator: Send + Sync {
    /// 指标名称（用于日志和 UI 展示）
    fn name(&self) -> &str;

    /// 计算指标值
    ///
    /// # 参数
    /// - `input`: 输入数据序列（通常为收盘价）
    ///
    /// # 返回
    /// - `Ok(IndicatorOutput)`: 计算成功
    /// - `Err(IndicatorError)`: 输入数据不足等错误
    fn compute(&self, input: &[f64]) -> Result<IndicatorOutput, IndicatorError>;
}

#[derive(Debug, thiserror::Error)]
pub enum IndicatorError {
    #[error("数据不足: 需要至少 {required} 个数据点，实际 {actual} 个")]
    InsufficientData { required: usize, actual: usize },
    #[error("无效参数:{0}")]
    InvalidParam(String),
}
