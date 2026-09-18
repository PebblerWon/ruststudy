# Phase 5：回测层技术实现

## 5.1 模块概述

### quant-app/Cargo.toml 补充依赖（Phase 5）

```toml
# quant-app/Cargo.toml 追加

[dependencies]
# 继承 workspace 依赖
polars = { workspace = true }
thiserror = { workspace = true }
csv = "1"                  # CSV 导出
```

### 本阶段目标

实现向量化回测引擎，基于历史 K 线数据执行策略回测，计算绩效统计并生成报告数据。

### 覆盖的 Rust 知识点

| Rust 特性 | 应用场景 |
|-----------|---------|
| **polars 数据处理** | K 线 DataFrame 操作、列计算、过滤 |
| **文件 I/O** | 从 KlineStore 加载缓存数据、CSV 导出 |
| **统计计算** | 收益率、夏普比率、最大回撤等金融指标 |
| **迭代器链** | 逐 bar 遍历、信号序列处理、收益序列聚合 |
| **thiserror** | 回测层错误类型定义 |

### 与前序阶段的衔接

- **Phase 1（数据层）**：`KlineStore` 提供历史 K 线数据（polars DataFrame）
- **Phase 2（指标层）**：`Indicator` trait 的 `compute()` 为回测提供指标值序列
- **Phase 5 定位**：接收数据层和指标层的输出，驱动策略生成信号，模拟交易并统计绩效

---

## 5.2 回测引擎设计

### 向量化回测 vs 事件驱动回测

| 维度 | 向量化回测 | 事件驱动回测 |
|------|-----------|-------------|
| **核心思路** | 一次性对整个数据序列做批量计算 | 逐 bar 回调，模拟实时事件流 |
| **性能** | 高（利用 polars 列式运算） | 低（逐条处理，开销大） |
| **实现复杂度** | 低 | 高（需模拟订单簿、撮合引擎） |
| **适用场景** | 学习、快速验证策略 | 生产级交易系统 |
| **本项目选择** | ✅ | ❌ |

**选择向量化回测的理由**：
1. 学习项目，重点在理解回测原理和 polars 数据处理
2. 数据量有限（~10000 根 K 线），向量化性能绰绰有余
3. 实现简洁，避免事件驱动框架的过度工程化

### 回测流程

```
加载历史数据 (DataFrame)
      │
      ▼
计算指标序列 (Indicator::compute)
      │
      ▼
逐 bar 生成信号 (Strategy → Signal 序列)
      │
      ▼
模拟交易 (信号 → 开仓/平仓 → TradeRecord)
      │
      ▼
计算绩效 (收益率、夏普、回撤 → BacktestResult)
```

### 代码骨架

```rust
// backtest/engine.rs

use anyhow::Result;
use polars::prelude::*;
use quant_data::common::models::{Kline, Interval};
use quant_data::indicators::Indicator;
use crate::strategy::{Strategy, Signal};
use super::report::{BacktestResult, BacktestConfig, TradeRecord};

/// 向量化回测引擎
pub struct BacktestEngine {
    config: BacktestConfig,
}

impl BacktestEngine {
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// 执行回测
    /// 1. 从 KlineStore 加载历史数据
    /// 2. 计算策略所需的指标
    /// 3. 逐 bar 生成信号并模拟交易
    /// 4. 计算绩效统计
    pub fn run(
        &self,
        klines: &[Kline],
        strategy: &mut dyn Strategy,
        indicators: &[&dyn Indicator],
    ) -> Result<BacktestResult> {
        // 计算所有指标
        let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();
        let indicator_values = self.compute_indicators(&closes, indicators);

        // 逐 bar 生成信号并模拟交易
        let mut simulator = TradeSimulator::new(&self.config);
        for (i, kline) in klines.iter().enumerate() {
            let signal = strategy.on_kline(kline, &indicator_values);
            simulator.process_bar(i, kline, signal);
        }

        // 计算绩效
        simulator.build_result(klines)
    }

    /// 批量计算指标值
    fn compute_indicators(
        &self,
        closes: &[f64],
        indicators: &[&dyn Indicator],
    ) -> IndicatorValues {
        // 调用每个 indicator.compute(closes)，组装 IndicatorValues
        // ...
    }
}
```

### 设计要点

- `BacktestEngine` 不持有数据，数据通过参数传入，便于测试
- 策略通过 `&mut dyn Strategy` 传入，支持动态分发
- 指标计算与信号生成解耦：先批量算指标，再逐 bar 生成信号

---

## 5.3 回测配置

### BacktestConfig 结构体

```rust
// backtest/report.rs（配置部分）

use serde::{Serialize, Deserialize};

/// 回测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// 交易对（如 "BTCUSDT"）
    pub symbol: String,
    /// K 线周期
    pub interval: Interval,
    /// 回测起始时间 (Unix ms)
    pub start_time: u64,
    /// 回测结束时间 (Unix ms)
    pub end_time: u64,
    /// 初始资金 (USDT)
    pub initial_capital: f64,
    /// 手续费率（如 0.001 = 0.1%）
    pub commission_rate: f64,
    /// 滑点（如 0.0005 = 0.05%）
    pub slippage: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            symbol: "BTCUSDT".to_string(),
            interval: Interval::D1,
            start_time: 0,
            end_time: u64::MAX,
            initial_capital: 10_000.0,
            commission_rate: 0.001,
            slippage: 0.0005,
        }
    }
}
```

### 设计要点

- 实现 `Default` 提供合理默认值，UI 面板可直接使用
- `Serialize/Deserialize` 支持配置持久化（JSON 文件）
- `slippage` 模拟真实交易中的价格偏移，使回测更贴近实际

---

## 5.4 信号生成

### Signal 枚举

```rust
// strategy/mod.rs（在 Phase 6 完善，Phase 5 先定义）

/// 交易信号
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    /// 买入信号
    Buy,
    /// 卖出信号
    Sell,
    /// 持有（无操作）
    Hold,
}
```

### SignalGenerator trait

```rust
// 与 Strategy trait 的关系：
// Strategy trait 在 Phase 6 完善，Phase 5 回测层直接调用 Strategy::on_kline()
// 信号生成是策略层的职责，回测层只消费信号

// 示例：基于 SMA 交叉的信号生成（在 Phase 6 的 SmaCrossStrategy 中实现）
//
// fn on_kline(&mut self, kline: &Kline, indicators: &IndicatorValues) -> Signal {
//     let sma_fast = indicators.sma.as_ref()?[self.fast_idx];
//     let sma_slow = indicators.sma.as_ref()?[self.slow_idx];
//     if sma_fast > sma_slow && !self.was_above {
//         self.was_above = true;
//         Signal::Buy
//     } else if sma_fast < sma_slow && self.was_above {
//         self.was_above = false;
//         Signal::Sell
//     } else {
//         Signal::Hold
//     }
// }
```

### 设计要点

- `Signal` 放在 `strategy/mod.rs` 中，回测层和策略层共享
- Phase 5 先使用简化的信号逻辑做回测验证，Phase 6 完善策略框架
- 信号生成遵循 ARCHITECTURE.md 定义：策略接收 `IndicatorValues`，输出 `Signal`

---

## 5.5 模拟交易引擎

### 核心结构体

```rust
// backtest/engine.rs

/// 持仓信息
#[derive(Debug, Clone)]
pub struct Position {
    /// 持仓方向（Buy = 多头）
    pub side: Signal,
    /// 持仓数量
    pub quantity: f64,
    /// 入场价格
    pub entry_price: f64,
    /// 入场时间 (Unix ms)
    pub open_time: u64,
    /// 入场 bar 索引
    pub entry_index: usize,
}

/// 单笔交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    /// 交易编号
    pub trade_id: u32,
    /// 交易对
    pub symbol: String,
    /// 方向（Buy/Sell）
    pub side: Signal,
    /// 入场价格
    pub entry_price: f64,
    /// 出场价格
    pub exit_price: f64,
    /// 交易数量
    pub quantity: f64,
    /// 盈亏金额 (USDT)
    pub pnl: f64,
    /// 盈亏百分比
    pub pnl_pct: f64,
    /// 手续费
    pub commission: f64,
    /// 开仓时间
    pub open_time: u64,
    /// 平仓时间
    pub close_time: u64,
}

/// 模拟交易引擎
pub struct TradeSimulator<'a> {
    config: &'a BacktestConfig,
    /// 当前持仓
    position: Option<Position>,
    /// 当前现金余额
    cash: f64,
    /// 已完成交易记录
    trades: Vec<TradeRecord>,
    /// 权益曲线 (bar_index, equity)
    equity_curve: Vec<(usize, f64)>,
    /// 交易计数器
    trade_counter: u32,
}

impl<'a> TradeSimulator<'a> {
    pub fn new(config: &'a BacktestConfig) -> Self {
        Self {
            config,
            position: None,
            cash: config.initial_capital,
            trades: Vec::new(),
            equity_curve: Vec::new(),
            trade_counter: 0,
        }
    }

    /// 处理每根 K 线的信号
    pub fn process_bar(&mut self, index: usize, kline: &Kline, signal: Signal) {
        match signal {
            Signal::Buy if self.position.is_none() => {
                // 开多仓：用全部资金买入
                let price = self.apply_slippage(kline.close, Signal::Buy);
                let quantity = self.cash / price;
                let commission = self.cash * self.config.commission_rate;
                self.cash -= commission;
                self.position = Some(Position {
                    side: Signal::Buy,
                    quantity,
                    entry_price: price,
                    open_time: kline.open_time,
                    entry_index: index,
                });
            }
            Signal::Sell if self.position.is_some() => {
                // 平仓：卖出全部持仓
                let pos = self.position.take().unwrap();
                let price = self.apply_slippage(kline.close, Signal::Sell);
                let revenue = pos.quantity * price;
                let commission = revenue * self.config.commission_rate;
                let pnl = revenue - (pos.entry_price * pos.quantity) - commission;
                let pnl_pct = pnl / (pos.entry_price * pos.quantity);

                self.cash += revenue - commission;
                self.trade_counter += 1;
                self.trades.push(TradeRecord {
                    trade_id: self.trade_counter,
                    symbol: self.config.symbol.clone(),
                    side: pos.side,
                    entry_price: pos.entry_price,
                    exit_price: price,
                    quantity: pos.quantity,
                    pnl,
                    pnl_pct,
                    commission,
                    open_time: pos.open_time,
                    close_time: kline.open_time,
                });
            }
            _ => {} // Hold 或重复信号，不操作
        }

        // 记录当前权益
        let equity = self.current_equity(kline.close);
        self.equity_curve.push((index, equity));
    }

    /// 应用滑点：买入价格上浮，卖出价格下浮
    fn apply_slippage(&self, price: f64, side: Signal) -> f64 {
        match side {
            Signal::Buy => price * (1.0 + self.config.slippage),
            Signal::Sell => price * (1.0 - self.config.slippage),
            Signal::Hold => price,
        }
    }

    /// 计算当前总权益（现金 + 持仓市值）
    fn current_equity(&self, current_price: f64) -> f64 {
        let position_value = self.position.as_ref()
            .map(|p| p.quantity * current_price)
            .unwrap_or(0.0);
        self.cash + position_value
    }

    /// 构建回测结果
    pub fn build_result(self, klines: &[Kline]) -> Result<BacktestResult> {
        // 计算各项绩效指标（见 §5.6）
        // ...
    }
}
```

### 设计要点

- `TradeSimulator` 持有 `&BacktestConfig` 引用，避免拷贝
- 全仓模式（每次信号用全部资金开/平仓），简化实现
- 滑点模型简单：按比例偏移，买入上浮、卖出下浮
- 权益曲线在每个 bar 结束时记录，包含持仓浮动盈亏

---

## 5.6 绩效统计

### BacktestResult 结构体

```rust
// backtest/report.rs

/// 回测结果
#[derive(Debug, Clone)]
pub struct BacktestResult {
    /// 回测配置
    pub config: BacktestConfig,
    /// 策略名称
    pub strategy_name: String,
    /// 总收益率
    pub total_return: f64,
    /// 年化收益率
    pub annualized_return: f64,
    /// 基准收益率（买入持有）
    pub benchmark_return: f64,
    /// 夏普比率
    pub sharpe_ratio: f64,
    /// 最大回撤
    pub max_drawdown: f64,
    /// 收益波动率
    pub volatility: f64,
    /// 总交易次数
    pub total_trades: u32,
    /// 胜率
    pub win_rate: f64,
    /// 盈亏比
    pub profit_loss_ratio: f64,
    /// 最大连续亏损次数
    pub max_consecutive_losses: u32,
    /// 总手续费
    pub total_commission: f64,
    /// 逐笔交易记录
    pub trades: Vec<TradeRecord>,
    /// 权益曲线 (时间戳, 净值)
    pub equity_curve: Vec<(u64, f64)>,
}
```

### 核心统计指标算法

#### total_return — 总收益率

```rust
/// 总收益率 = (最终权益 - 初始资金) / 初始资金
fn calc_total_return(final_equity: f64, initial_capital: f64) -> f64 {
    (final_equity - initial_capital) / initial_capital
}
```

#### annualized_return — 年化收益率

```rust
/// 年化收益率 = (1 + 总收益率) ^ (365 / 回测天数) - 1
fn calc_annualized_return(total_return: f64, start_time: u64, end_time: u64) -> f64 {
    let days = (end_time - start_time) as f64 / (86_400_000.0); // ms → 天
    (1.0 + total_return).powf(365.0 / days) - 1.0
}
```

#### sharpe_ratio — 夏普比率

```rust
/// 夏普比率 = (年化收益率 - 无风险利率) / 年化波动率
/// risk_free_rate: 无风险年化利率（默认 0.0，可配置）
fn calc_sharpe_ratio(
    daily_returns: &[f64],
    risk_free_rate: f64,
) -> f64 {
    if daily_returns.is_empty() {
        return 0.0;
    }
    let mean_return = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
    let variance = daily_returns.iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>() / daily_returns.len() as f64;
    let daily_vol = variance.sqrt();
    if daily_vol == 0.0 {
        return 0.0;
    }
    // 年化：日均收益 × 365，日波动率 × √365
    let annualized_return = mean_return * 365.0;
    let annualized_vol = daily_vol * (365.0_f64).sqrt();
    (annualized_return - risk_free_rate) / annualized_vol
}
```

#### max_drawdown — 最大回撤

```rust
/// 最大回撤 = max((峰值 - 谷值) / 峰值)，遍历权益曲线
fn calc_max_drawdown(equity_curve: &[(u64, f64)]) -> f64 {
    let mut peak = 0.0_f64;
    let mut max_dd = 0.0_f64;
    for &(_, equity) in equity_curve {
        if equity > peak {
            peak = equity;
        }
        let drawdown = (peak - equity) / peak;
        if drawdown > max_dd {
            max_dd = drawdown;
        }
    }
    max_dd
}
```

#### win_rate — 胜率

```rust
/// 胜率 = 盈利交易数 / 总交易数
fn calc_win_rate(trades: &[TradeRecord]) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }
    let wins = trades.iter().filter(|t| t.pnl > 0.0).count();
    wins as f64 / trades.len() as f64
}
```

#### profit_loss_ratio — 盈亏比

```rust
/// 盈亏比 = 平均盈利 / 平均亏损（取绝对值）
fn calc_profit_loss_ratio(trades: &[TradeRecord]) -> f64 {
    let wins: Vec<f64> = trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).collect();
    let losses: Vec<f64> = trades.iter().filter(|t| t.pnl < 0.0).map(|t| t.pnl.abs()).collect();
    if wins.is_empty() || losses.is_empty() {
        return f64::INFINITY; // 无亏损或无盈利
    }
    let avg_win = wins.iter().sum::<f64>() / wins.len() as f64;
    let avg_loss = losses.iter().sum::<f64>() / losses.len() as f64;
    if avg_loss == 0.0 {
        f64::INFINITY
    } else {
        avg_win / avg_loss
    }
}
```

#### 收益曲线生成

```rust
/// 权益曲线已在 TradeSimulator 中逐 bar 生成
/// 转换为 (时间戳, 净值) 格式供 UI 绘制
fn build_equity_curve(
    raw_curve: &[(usize, f64)],
    klines: &[Kline],
) -> Vec<(u64, f64)> {
    raw_curve.iter()
        .map(|&(idx, equity)| (klines[idx].open_time, equity))
        .collect()
}
```

### 设计要点

- 所有统计函数为纯函数（`fn` 而非 `method`），便于单元测试
- 夏普比率使用日收益率序列计算后年化，而非直接用价格序列
- 最大回撤遍历一次 O(n)，无需额外数据结构
- 盈亏比处理了除零边界（无亏损返回 `INFINITY`）

---

## 5.7 回测报告

### BacktestReport 结构体

```rust
// backtest/report.rs

/// 回测报告（供 UI 层展示的数据）
pub struct BacktestReport {
    /// 回测结果
    pub result: BacktestResult,
}

impl BacktestReport {
    pub fn new(result: BacktestResult) -> Self {
        Self { result }
    }

    /// 获取统计指标摘要（供 UI 表格展示）
    pub fn summary(&self) -> Vec<(&str, String)> {
        vec![
            ("总收益率", format!("{:.2}%", self.result.total_return * 100.0)),
            ("年化收益率", format!("{:.2}%", self.result.annualized_return * 100.0)),
            ("夏普比率", format!("{:.4}", self.result.sharpe_ratio)),
            ("最大回撤", format!("{:.2}%", self.result.max_drawdown * 100.0)),
            ("胜率", format!("{:.1}%", self.result.win_rate * 100.0)),
            ("盈亏比", format!("{:.2}", self.result.profit_loss_ratio)),
            ("总交易次数", self.result.total_trades.to_string()),
            ("总手续费", format!("{:.2} USDT", self.result.total_commission)),
        ]
    }

    /// 导出交易记录为 CSV
    pub fn export_trades_csv(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let mut wtr = csv::Writer::from_path(path)?;
        // 写入表头
        wtr.write_record([
            "trade_id", "symbol", "side", "entry_price", "exit_price",
            "quantity", "pnl", "pnl_pct", "commission", "open_time", "close_time",
        ])?;
        // 写入每笔交易
        for trade in &self.result.trades {
            wtr.serialize(trade)?;
        }
        wtr.flush()?;
        Ok(())
    }
}
```

### 数据加载器

```rust
// backtest/data_loader.rs

use polars::prelude::*;
use quant_data::common::models::Kline;
use quant_data::data::kline_store::KlineStore;

/// 从 KlineStore 加载历史数据并转换为回测所需格式
pub struct DataLoader;

impl DataLoader {
    /// 加载指定时间范围的 K 线数据
    pub fn load(
        store: &KlineStore,
        symbol: &str,
        interval: Interval,
        start_time: u64,
        end_time: u64,
    ) -> anyhow::Result<Vec<Kline>> {
        let klines = store.load_klines(symbol, interval)?;
        // 按时间范围过滤
        let filtered: Vec<Kline> = klines.into_iter()
            .filter(|k| k.open_time >= start_time && k.open_time <= end_time)
            .collect();
        if filtered.is_empty() {
            anyhow::bail!("指定时间范围内无 K 线数据");
        }
        Ok(filtered)
    }

    /// 将 K 线数据转换为 polars DataFrame（用于高级分析）
    pub fn to_dataframe(klines: &[Kline]) -> DataFrame {
        let open_times: Vec<i64> = klines.iter().map(|k| k.open_time).collect();
        let opens: Vec<f64> = klines.iter().map(|k| k.open).collect();
        let highs: Vec<f64> = klines.iter().map(|k| k.high).collect();
        let lows: Vec<f64> = klines.iter().map(|k| k.low).collect();
        let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();
        let volumes: Vec<f64> = klines.iter().map(|k| k.volume).collect();

        DataFrame::new(vec![
            Column::new("open_time".into(), open_times),
            Column::new("open".into(), opens),
            Column::new("high".into(), highs),
            Column::new("low".into(), lows),
            Column::new("close".into(), closes),
            Column::new("volume".into(), volumes),
        ]).expect("DataFrame 创建失败")
    }
}
```

### 设计要点

- `BacktestReport` 是 UI 层的数据源，不包含渲染逻辑
- CSV 导出使用 `csv` crate（需添加到 Cargo.toml）
- `DataLoader` 桥接数据层和回测层，负责数据格式转换
- DataFrame 转换在需要高级分析时使用，基础回测直接用 `Vec<Kline>`

---

## 5.8 测试策略

### 单元测试

| 测试目标 | 测试方法 | 验证方式 |
|---------|---------|---------|
| `calc_total_return` | 已知初始资金和最终权益 | 断言收益率精确到 4 位小数 |
| `calc_sharpe_ratio` | 构造固定日收益序列 | 与手动计算结果对比 |
| `calc_max_drawdown` | 构造含明显峰谷的权益曲线 | 断言最大回撤值 |
| `calc_win_rate` | 已知胜负的交易序列 | 断言胜率百分比 |
| `calc_profit_loss_ratio` | 已知盈亏金额 | 断言盈亏比 |
| `TradeSimulator::process_bar` | 手动构造 K 线 + 信号序列 | 验证交易记录和权益曲线 |
| `apply_slippage` | 给定价格和滑点 | 验证买入上浮、卖出下浮 |

### 集成测试

```rust
// tests/backtest_tests.rs

#[test]
fn test_full_backtest_flow() {
    // 1. 构造 10 根简单 K 线数据
    // 2. 使用简单策略（如收盘价上涨则买入）
    // 3. 执行 BacktestEngine::run()
    // 4. 验证 BacktestResult 各字段合理
    // 5. 验证 equity_curve 非空
    // 6. 验证 trades 数量 > 0
}
```

### 验收标准

1. 所有统计指标函数通过已知数据的单元测试
2. 完整回测流程（加载数据 → 信号 → 交易 → 绩效）可通过集成测试
3. CSV 导出文件可被 Excel 正确打开，数据无丢失
4. 手续费和滑点计算正确（手动验证 1-2 笔交易）
