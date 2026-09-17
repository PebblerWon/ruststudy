# Phase 2：指标层技术实现

## 2.1 模块概述

### 目标

自行实现核心技术指标计算（SMA、EMA、RSI、MACD、布林带），通过 `Indicator` trait 统一抽象，支持对任意 K 线数据集批量计算。

### 覆盖的 Rust 知识点

| 知识点 | 实践场景 |
|--------|---------|
| **Trait** | `Indicator` trait 定义统一指标接口 |
| **泛型** | 支持 `&[f64]` 和 polars `Series` 两种输入类型 |
| **迭代器** | 滑动窗口计算、`Iterator` 链式处理 |
| **数值计算** | f64 精度控制、数值稳定性处理 |
| **thiserror** | 指标层新增错误类型（本阶段引入） |

### 与 Phase 1 的衔接

```
Phase 1 输出: Vec<Kline>  →  提取 closes: Vec<f64>  →  Phase 2 指标计算
                                                         ↓
                                                    IndicatorResult
                                                         ↓
                                              传递给 Phase 3 (UI) / Phase 5 (回测)
```

指标层是**纯计算模块**，不依赖网络或文件 IO，仅接收数值序列、返回计算结果。

---

## 2.2 指标 Trait 设计

### Indicator trait 定义

```rust
// src/indicators/mod.rs

/// 指标计算结果
/// 支持单值指标（SMA/EMA/RSI）和多值指标（MACD/布林带）
#[derive(Debug, Clone)]
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
pub trait Indicator {
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

/// 指标计算错误
#[derive(Debug, thiserror::Error)]
pub enum IndicatorError {
    /// 输入数据不足以完成计算
    #[error("数据不足: 需要至少 {required} 个数据点，实际 {actual} 个")]
    InsufficientData { required: usize, actual: usize },

    /// 参数无效（如 period <= 0）
    #[error("无效参数: {0}")]
    InvalidParam(String),
}
```

### 设计要点

- `IndicatorOutput` 用枚举区分单值/多值指标，UI 层根据变体选择渲染方式
- 输出序列长度与输入相同，不足 `period` 的位置填充 `f64::NAN`，便于与时间轴对齐
- `compute` 返回 `Result`，数据不足时返回明确错误而非 panic
- trait 不绑定具体数据类型，`&[f64]` 是最通用的输入形式

---

## 2.3 移动平均线（SMA / EMA）

### SMA — 简单移动平均线

```rust
// src/indicators/sma.rs

use crate::indicators::{Indicator, IndicatorError, IndicatorOutput};

/// 简单移动平均线
/// SMA = (P1 + P2 + ... + Pn) / n
pub struct Sma {
    /// 计算周期
    period: usize,
}

impl Sma {
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        if period == 0 {
            return Err(IndicatorError::InvalidParam(
                "SMA period 必须大于 0".into()
            ));
        }
        Ok(Self { period })
    }
}

impl Indicator for Sma {
    fn name(&self) -> &str {
        "SMA"
    }

    fn compute(&self, input: &[f64]) -> Result<IndicatorOutput, IndicatorError> {
        if input.len() < self.period {
            return Err(IndicatorError::InsufficientData {
                required: self.period,
                actual: input.len(),
            });
        }

        let mut result = Vec::with_capacity(input.len());

        // 前 period-1 个值填充 NAN
        result.extend(std::iter::repeat(f64::NAN).take(self.period - 1));

        // 使用滑动窗口计算 SMA
        // 维护窗口内元素之和，避免每次重新求和
        let mut window_sum: f64 = input[..self.period].iter().sum();
        result.push(window_sum / self.period as f64);

        for i in self.period..input.len() {
            window_sum += input[i] - input[i - self.period];
            result.push(window_sum / self.period as f64);
        }

        Ok(IndicatorOutput::Single(result))
    }
}
```

### EMA — 指数移动平均线

```rust
// src/indicators/ema.rs

use crate::indicators::{Indicator, IndicatorError, IndicatorOutput};

/// 指数移动平均线
/// EMA_t = price_t × k + EMA_{t-1} × (1 - k)
/// 其中 k = 2 / (period + 1)（平滑系数）
pub struct Ema {
    period: usize,
    /// 平滑系数 k = 2 / (period + 1)
    multiplier: f64,
}

impl Ema {
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        if period == 0 {
            return Err(IndicatorError::InvalidParam(
                "EMA period 必须大于 0".into()
            ));
        }
        Ok(Self {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
        })
    }
}

impl Indicator for Ema {
    fn name(&self) -> &str {
        "EMA"
    }

    fn compute(&self, input: &[f64]) -> Result<IndicatorOutput, IndicatorError> {
        if input.len() < self.period {
            return Err(IndicatorError::InsufficientData {
                required: self.period,
                actual: input.len(),
            });
        }

        let mut result = Vec::with_capacity(input.len());

        // 前 period-1 个值填充 NAN
        result.extend(std::iter::repeat(f64::NAN).take(self.period - 1));

        // 初始值：前 period 个值的 SMA
        let first_sma: f64 = input[..self.period].iter().sum::<f64>() / self.period as f64;
        result.push(first_sma);

        // 递推计算 EMA
        for i in self.period..input.len() {
            let prev = result[i - 1]; // 前一个 EMA 值
            let ema = input[i] * self.multiplier + prev * (1.0 - self.multiplier);
            result.push(ema);
        }

        Ok(IndicatorOutput::Single(result))
    }
}
```

### 算法说明

| 指标 | 算法 | 时间复杂度 | 边界处理 |
|------|------|-----------|---------|
| SMA | 滑动窗口求和 / period | O(n) | 窗口内不足 period 个值时填 NAN |
| EMA | 递推公式，初始值取 SMA | O(n) | 前 period-1 个值填 NAN，第 period 个取 SMA |

### 设计要点

- SMA 使用**滑动窗口和**优化，避免 O(n × period) 的重复求和
- EMA 的初始值使用前 period 个值的 SMA（业界标准做法）
- 平滑系数 `multiplier` 在构造时预计算，避免每次循环重复计算

---

## 2.4 RSI（相对强弱指数）

```rust
// src/indicators/rsi.rs

use crate::indicators::{Indicator, IndicatorError, IndicatorOutput};

/// 相对强弱指数（Relative Strength Index）
/// RSI = 100 - 100 / (1 + RS)
/// RS = 平均涨幅 / 平均跌幅（使用 Wilder 平滑）
pub struct Rsi {
    period: usize,
}

impl Rsi {
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        if period == 0 {
            return Err(IndicatorError::InvalidParam(
                "RSI period 必须大于 0".into()
            ));
        }
        Ok(Self { period })
    }
}

impl Indicator for Rsi {
    fn name(&self) -> &str {
        "RSI"
    }

    fn compute(&self, input: &[f64]) -> Result<IndicatorOutput, IndicatorError> {
        // 需要至少 period+1 个数据点（计算 period 个价格变化）
        if input.len() <= self.period {
            return Err(IndicatorError::InsufficientData {
                required: self.period + 1,
                actual: input.len(),
            });
        }

        let mut result = Vec::with_capacity(input.len());
        // 第一个值无法计算变化，前 period 个值填 NAN
        result.extend(std::iter::repeat(f64::NAN).take(self.period));

        // 计算价格变化
        let changes: Vec<f64> = input.windows(2)
            .map(|w| w[1] - w[0])
            .collect();

        // 初始平均涨幅/跌幅：前 period 个变化的简单平均
        let mut avg_gain: f64 = changes[..self.period]
            .iter()
            .filter(|&&c| c > 0.0)
            .sum::<f64>() / self.period as f64;
        let mut avg_loss: f64 = changes[..self.period]
            .iter()
            .filter(|&&c| c < 0.0)
            .map(|c| c.abs())
            .sum::<f64>() / self.period as f64;

        // 第一个 RSI 值
        result.push(Self::calc_rsi(avg_gain, avg_loss));

        // Wilder 平滑递推
        for i in self.period..changes.len() {
            let change = changes[i];
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { change.abs() } else { 0.0 };

            // Wilder 平滑：avg = (prev_avg × (period-1) + current) / period
            avg_gain = (avg_gain * (self.period as f64 - 1.0) + gain) / self.period as f64;
            avg_loss = (avg_loss * (self.period as f64 - 1.0) + loss) / self.period as f64;

            result.push(Self::calc_rsi(avg_gain, avg_loss));
        }

        Ok(IndicatorOutput::Single(result))
    }
}

impl Rsi {
    /// 根据平均涨幅和平均跌幅计算 RSI 值
    fn calc_rsi(avg_gain: f64, avg_loss: f64) -> f64 {
        // 全跌：avg_loss > 0, avg_gain == 0 → RSI = 0
        if avg_loss == 0.0 {
            return 100.0;
        }
        // 全涨：avg_gain > 0, avg_loss == 0 → 已在上面处理
        // 正常情况
        let rs = avg_gain / avg_loss;
        100.0 - 100.0 / (1.0 + rs)
    }
}
```

### 边界情况处理

| 场景 | 处理方式 |
|------|---------|
| 数据不足 `period+1` 个 | 返回 `InsufficientData` 错误 |
| 全涨（avg_loss = 0） | RSI = 100.0 |
| 全跌（avg_gain = 0） | RSI = 0.0 |
| 首个周期 | 使用简单平均，后续使用 Wilder 平滑 |

### 设计要点

- Wilder 平滑是 RSI 的标准算法（TradingView 等平台均使用），不同于简单 EMA 平滑
- 初始平均值使用 SMA，之后递推，确保与权威平台数值一致
- `calc_rsi` 独立为私有方法，处理除零边界

---

## 2.5 MACD（移动平均收敛散度）

```rust
// src/indicators/macd.rs

use crate::indicators::{Indicator, IndicatorError, IndicatorOutput};
use crate::indicators::ema::Ema;

/// MACD 指标
/// MACD Line = EMA(fast) - EMA(slow)
/// Signal Line = EMA(MACD Line, signal_period)
/// Histogram = MACD Line - Signal Line
pub struct Macd {
    fast_period: usize,    // 默认 12
    slow_period: usize,    // 默认 26
    signal_period: usize,  // 默认 9
}

impl Macd {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize)
        -> Result<Self, IndicatorError>
    {
        if fast_period >= slow_period {
            return Err(IndicatorError::InvalidParam(
                "fast_period 必须小于 slow_period".into()
            ));
        }
        Ok(Self { fast_period, slow_period, signal_period })
    }

    /// 使用默认参数创建（12, 26, 9）
    pub fn default_params() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        }
    }
}

impl Indicator for Macd {
    fn name(&self) -> &str {
        "MACD"
    }

    fn compute(&self, input: &[f64]) -> Result<IndicatorOutput, IndicatorError> {
        if input.len() < self.slow_period {
            return Err(IndicatorError::InsufficientData {
                required: self.slow_period,
                actual: input.len(),
            });
        }

        // 1. 计算快/慢 EMA
        let fast_ema = Ema::new(self.fast_period)?.compute(input)?;
        let slow_ema = Ema::new(self.slow_period)?.compute(input)?;

        // 2. 提取 EMA 值（跳过前 slow_period-1 个 NAN）
        let fast_values = Self::extract_values(&fast_ema);
        let slow_values = Self::extract_values(&slow_ema);

        // 3. 计算 MACD Line = fast_ema - slow_ema
        let macd_line: Vec<f64> = fast_values.iter()
            .zip(slow_values.iter())
            .map(|(f, s)| f - s)
            .collect();

        // 4. 计算 Signal Line = EMA(MACD Line)
        let signal_ema = Ema::new(self.signal_period)?;
        let signal_values = signal_ema.compute(&macd_line)?;
        let signal = Self::extract_values(&signal_values);

        // 5. 计算 Histogram = MACD - Signal
        let histogram: Vec<f64> = macd_line.iter()
            .zip(signal.iter())
            .map(|(m, s)| m - s)
            .collect();

        // 6. 构建完整输出（前 slow_period-1 个值填 NAN）
        let offset = self.slow_period - 1;
        let mut macd_full = vec![f64::NAN; offset];
        macd_full.extend(macd_line);

        let mut signal_full = vec![f64::NAN; offset];
        signal_full.extend(signal);

        let mut hist_full = vec![f64::NAN; offset];
        hist_full.extend(histogram);

        Ok(IndicatorOutput::Multi {
            names: vec![
                "macd_line".into(),
                "signal_line".into(),
                "histogram".into(),
            ],
            values: vec![macd_full, signal_full, hist_full],
        })
    }
}

impl Macd {
    /// 从 IndicatorOutput 中提取有效值（跳过 NAN）
    fn extract_values(output: &IndicatorOutput) -> Vec<f64> {
        match output {
            IndicatorOutput::Single(v) => {
                v.iter().copied().filter(|x| !x.is_nan()).collect()
            }
            _ => unreachable!("EMA 返回 Single 类型"),
        }
    }
}
```

### 设计要点

- MACD **组合使用** `Ema` 计算，而非重新实现 EMA 逻辑，体现代码复用
- 输出为 `IndicatorOutput::Multi`，包含三条线：macd_line、signal_line、histogram
- 前 `slow_period - 1` 个位置填 NAN，确保输出长度与输入对齐
- 默认参数 `(12, 26, 9)` 是业界标准

---

## 2.6 布林带（Bollinger Bands）

```rust
// src/indicators/bollinger.rs

use crate::indicators::{Indicator, IndicatorError, IndicatorOutput};

/// 布林带指标
/// Middle = SMA(period)
/// Upper = Middle + std_dev × σ
/// Lower = Middle - std_dev × σ
/// 其中 σ 为窗口内价格的标准差
pub struct BollingerBands {
    period: usize,       // 默认 20
    std_dev: f64,        // 默认 2.0
}

impl BollingerBands {
    pub fn new(period: usize, std_dev: f64) -> Result<Self, IndicatorError> {
        if period == 0 {
            return Err(IndicatorError::InvalidParam(
                "Bollinger period 必须大于 0".into()
            ));
        }
        Ok(Self { period, std_dev })
    }

    /// 使用默认参数创建（period=20, std_dev=2.0）
    pub fn default_params() -> Self {
        Self { period: 20, std_dev: 2.0 }
    }
}

impl Indicator for BollingerBands {
    fn name(&self) -> &str {
        "BollingerBands"
    }

    fn compute(&self, input: &[f64]) -> Result<IndicatorOutput, IndicatorError> {
        if input.len() < self.period {
            return Err(IndicatorError::InsufficientData {
                required: self.period,
                actual: input.len(),
            });
        }

        let mut upper = Vec::with_capacity(input.len());
        let mut middle = Vec::with_capacity(input.len());
        let mut lower = Vec::with_capacity(input.len());

        // 前 period-1 个值填 NAN
        let nan_prefix = vec![f64::NAN; self.period - 1];
        upper.extend(&nan_prefix);
        middle.extend(&nan_prefix);
        lower.extend(&nan_prefix);

        // 滑动窗口计算布林带
        for i in (self.period - 1)..input.len() {
            let window = &input[i - self.period + 1..=i];

            // 中轨 = SMA
            let sma: f64 = window.iter().sum::<f64>() / self.period as f64;

            // 标准差 σ = sqrt(Σ(xi - mean)² / n)
            let variance: f64 = window.iter()
                .map(|&x| (x - sma).powi(2))
                .sum::<f64>() / self.period as f64;
            let std = variance.sqrt();

            // 上轨 / 下轨
            upper.push(sma + self.std_dev * std);
            middle.push(sma);
            lower.push(sma - self.std_dev * std);
        }

        Ok(IndicatorOutput::Multi {
            names: vec!["upper".into(), "middle".into(), "lower".into()],
            values: vec![upper, middle, lower],
        })
    }
}
```

### 设计要点

- 标准差使用**总体标准差**（除以 n），与 TradingView 一致
- 滑动窗口同时计算 SMA 和标准差，避免重复遍历
- 输出三条线：upper、middle、lower，UI 层可绘制为通道带

---

## 2.7 指标计算管道

```rust
// src/indicators/mod.rs (扩展)

use crate::common::models::Kline;

/// 指标计算管道 — 管理多个指标实例，批量计算并收集结果
pub struct IndicatorPipeline {
    indicators: Vec<Box<dyn Indicator>>,
}

impl IndicatorPipeline {
    /// 创建空的管道
    pub fn new() -> Self {
        Self { indicators: Vec::new() }
    }

    /// 添加一个指标到管道
    pub fn add(mut self, indicator: Box<dyn Indicator>) -> Self {
        self.indicators.push(indicator);
        self
    }

    /// 从 K 线数据中提取收盘价，批量计算所有指标
    pub fn compute_all(&self, klines: &[Kline]) -> Vec<(String, Result<IndicatorOutput, IndicatorError>)> {
        let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();
        self.compute_from_closes(&closes)
    }

    /// 从收盘价序列批量计算所有指标
    pub fn compute_from_closes(
        &self,
        closes: &[f64],
    ) -> Vec<(String, Result<IndicatorOutput, IndicatorError>)> {
        self.indicators.iter()
            .map(|ind| {
                let result = ind.compute(closes);
                (ind.name().to_string(), result)
            })
            .collect()
    }
}
```

### 与 polars DataFrame 的集成

```rust
// 使用示例：将指标结果追加为 DataFrame 列
//
// let df = klines_to_dataframe(&klines)?;
// let pipeline = IndicatorPipeline::new()
//     .add(Box::new(Sma::new(20)?))
//     .add(Box::new(Ema::new(12)?))
//     .add(Box::new(Rsi::new(14)?));
//
// let results = pipeline.compute_all(&klines);
// for (name, result) in results {
//     match result? {
//         IndicatorOutput::Single(values) => {
//             df.with_column(Series::new(&name, &values))?;
//         }
//         IndicatorOutput::Multi { names, values } => {
//             for (col_name, col_values) in names.iter().zip(values.iter()) {
//                 df.with_column(Series::new(col_name, col_values))?;
//             }
//         }
//     }
// }
```

### 设计要点

- `IndicatorPipeline` 使用 `Box<dyn Indicator>` 实现**动态分发**，支持运行时组合任意指标
- Builder 模式（`add` 返回 `self`）支持链式调用
- `compute_all` 接收 `&[Kline]`，内部自动提取收盘价
- 返回 `Vec<(name, Result)>`，单个指标计算失败不影响其他指标

---

## 2.8 测试策略

### 单元测试

| 指标 | 测试用例 | 验证方法 |
|------|---------|---------|
| **SMA** | 输入 `[1,2,3,4,5]`, period=3 → `[NAN, NAN, 2.0, 3.0, 4.0]` | 精确匹配已知输出 |
| **EMA** | 输入 `[1,2,3,4,5]`, period=3 → 验证递推值 | 与 TradingView 数值对比，误差 < 0.01% |
| **RSI** | 14 个递增价格 → RSI = 100；14 个递减价格 → RSI = 0 | 边界值精确匹配 |
| **RSI** | 标准 14 周期输入 → 验证 Wilder 平滑结果 | 与 TradingView RSI 对比 |
| **MACD** | 26+ 个数据点 → 验证 macd_line、signal_line、histogram | 与 TradingView MACD 对比 |
| **Bollinger** | 20 个数据点 → 验证 upper、middle、lower | middle = SMA(20)，带宽 = 4σ |

### 边界测试

| 场景 | 预期行为 |
|------|---------|
| 空输入 `&[]` | 返回 `InsufficientData` 错误 |
| 数据不足 period | 返回 `InsufficientData` 错误 |
| period = 0 | 构造时返回 `InvalidParam` 错误 |
| 所有价格相同 | SMA/EMA 返回该价格，RSI 返回 50（avg_gain=avg_loss=0 的特殊处理） |
| 极大数据集（10000+） | 计算耗时 < 100ms（PRD 性能要求） |

### 性能基准

```rust
// 在 tests/indicator_tests.rs 中
#[test]
fn benchmark_large_dataset() {
    let data: Vec<f64> = (0..10000).map(|i| 100.0 + (i as f64).sin()).collect();
    let sma = Sma::new(20).unwrap();

    let start = std::time::Instant::now();
    sma.compute(&data).unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 100, "SMA 计算 10000 条数据应 < 100ms");
}
```

### 验收标准

1. ✅ 各指标计算结果与 TradingView 数值一致（误差 < 0.01%）
2. ✅ `Indicator` trait 抽象合理，新指标仅需实现 `compute` 方法
3. ✅ 边界情况（空数据、不足 period、极值）均有明确错误处理
4. ✅ 10000 条数据指标计算耗时 < 100ms
5. ✅ 所有单元测试和边界测试通过
