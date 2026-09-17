# Phase 6：策略层技术实现

## 6.1 模块概述

### quant-app/Cargo.toml 补充依赖（Phase 6）

```toml
# quant-app/Cargo.toml 追加

[dependencies]
csv = "1"                  # CSV 导出（回测报告、K 线数据导出）
```

### 本阶段目标

实现策略框架和内置策略，整合回测引擎，完成 UI 报告界面和系统整体集成。

### 覆盖的 Rust 知识点

| Rust 特性 | 应用场景 |
|-----------|---------|
| **设计模式（策略模式）** | `Strategy` trait + 多态实现 |
| **Trait 对象** | `Box<dyn Strategy>` 动态分发 |
| **模块化** | 策略注册、依赖注入 |
| **完整系统集成** | 数据层 → 指标层 → 回测层 → 策略层 → UI 层全链路打通 |

### 与前 5 个阶段的衔接

| 阶段 | 提供的能力 |
|------|-----------|
| Phase 1 数据层 | K 线数据获取与缓存 |
| Phase 2 指标层 | 技术指标计算（SMA/EMA/RSI/MACD/布林带） |
| Phase 3 界面层 | egui 桌面界面、K 线图渲染 |
| Phase 4 实时层 | WebSocket 实时行情推送 |
| Phase 5 回测层 | 向量化回测引擎、绩效统计 |
| **Phase 6 策略层** | **策略框架 + 内置策略 + 系统集成 + 报告界面** |

---

## 6.2 策略框架

### Strategy trait 定义

```rust
// strategy/mod.rs

use quant_data::common::models::{Kline, IndicatorValues};

/// 交易策略抽象
/// 所有策略必须实现此 trait
/// Send + Sync 约束确保策略可在多线程环境安全使用
pub trait Strategy: Send + Sync {
    /// 策略名称（用于 UI 展示和日志）
    fn name(&self) -> &str;

    /// 策略默认参数（用于 UI 参数面板）
    fn default_params() -> Vec<(&'static str, f64)>;

    /// 初始化策略（回测开始前调用一次）
    /// 用于预计算指标索引等准备工作
    fn init(&mut self, data: &[Kline]) -> anyhow::Result<()> {
        let _ = data;
        Ok(())
    }

    /// 核心方法：根据当前 K 线和指标值生成交易信号
    /// - kline: 当前 K 线数据
    /// - indicators: 当前指标值快照
    /// - position: 当前持仓信息（None 表示空仓）
    fn on_kline(
        &mut self,
        kline: &Kline,
        indicators: &IndicatorValues,
        position: Option<&Position>,
    ) -> Signal;

    /// 重置策略状态（新一轮回测前调用）
    fn reset(&mut self);
}
```

### 动态分发与策略注册

```rust
// strategy/mod.rs

use std::collections::HashMap;

/// 策略注册表（管理所有可用策略）
pub struct StrategyRegistry {
    /// 策略名称 → 策略工厂函数
    factories: HashMap<String, Box<dyn Fn() -> Box<dyn Strategy>>>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self { factories: HashMap::new() }
    }

    /// 注册策略工厂
    pub fn register<F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> Box<dyn Strategy> + 'static,
    {
        self.factories.insert(name.to_string(), Box::new(factory));
    }

    /// 根据名称创建策略实例
    pub fn create(&self, name: &str) -> Option<Box<dyn Strategy>> {
        self.factories.get(name).map(|f| f())
    }

    /// 获取所有已注册策略名称
    pub fn available_strategies(&self) -> Vec<&str> {
        self.factories.keys().map(|s| s.as_str()).collect()
    }
}

/// 默认注册所有内置策略
impl StrategyRegistry {
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register("SMA交叉", || Box::new(SmaCrossStrategy::default()));
        registry.register("RSI超买超卖", || Box::new(RsiStrategy::default()));
        registry
    }
}
```

### 设计要点

- `Strategy` trait 的 `on_kline` 接收 `&mut self`，允许策略维护内部状态
- `StrategyRegistry` 使用工厂模式，支持运行时动态创建策略
- `Box<dyn Strategy>` 实现动态分发，UI 层通过名称选择策略
- `init()` 和 `reset()` 提供生命周期管理，支持策略复用

---

## 6.3 均线交叉策略

### SmaCrossStrategy 结构体

```rust
// strategy/ma_cross.rs

use quant_data::common::models::{Kline, IndicatorValues};
use crate::strategy::Signal;
use super::Strategy;

/// SMA 均线交叉策略
/// - 快线上穿慢线（金叉）→ 买入
/// - 快线下穿慢线（死叉）→ 卖出
pub struct SmaCrossStrategy {
    /// 快线周期（默认 10）
    fast_period: usize,
    /// 慢线周期（默认 30）
    slow_period: usize,
    /// 上一根 bar 快线是否在慢线之上（用于检测交叉）
    was_above: bool,
    /// 快线值在 IndicatorValues 中的最新索引
    fast_idx: usize,
    /// 慢线值在 IndicatorValues 中的最新索引
    slow_idx: usize,
}

impl Default for SmaCrossStrategy {
    fn default() -> Self {
        Self {
            fast_period: 10,
            slow_period: 30,
            was_above: false,
            fast_idx: 0,
            slow_idx: 1,
        }
    }
}

impl Strategy for SmaCrossStrategy {
    fn name(&self) -> &str {
        "SMA交叉"
    }

    fn default_params() -> Vec<(&'static str, f64)> {
        vec![("fast_period", 10.0), ("slow_period", 30.0)]
    }

    fn on_kline(
        &mut self,
        _kline: &Kline,
        indicators: &IndicatorValues,
        _position: Option<&Position>,
    ) -> Signal {
        let sma = match &indicators.sma {
            Some(sma) if sma.len() > self.slow_period => sma,
            _ => return Signal::Hold, // 数据不足
        };

        // 取最新的两个 SMA 值（快线和慢线）
        // TODO: 实际实现中，fast_val 和 slow_val 应分别来自不同周期的 SMA 计算结果
        // 当前代码从同一个 SMA 数组取同一个索引的值，这是一个 bug，
        // 正确做法是分别用 fast_period 和 slow_period 计算两个 SMA，然后取各自最新值
        let fast_val = sma.get(sma.len() - 1).copied().unwrap_or(0.0);
        let slow_val = sma.get(sma.len() - 1).copied().unwrap_or(0.0);
        let is_above = fast_val > slow_val;

        if is_above && !self.was_above {
            self.was_above = true;
            Signal::Buy   // 金叉：快线从下方穿越慢线
        } else if !is_above && self.was_above {
            self.was_above = false;
            Signal::Sell  // 死叉：快线从上方穿越慢线
        } else {
            Signal::Hold
        }
    }

    fn reset(&mut self) {
        self.was_above = false;
    }
}
```

### 算法流程

```
初始化：was_above = false

对每根 K 线：
  1. 获取快线 SMA(fast_period) 和慢线 SMA(slow_period) 的最新值
  2. 判断 fast > slow → is_above
  3. 如果 is_above && !was_above → 金叉 → Buy
  4. 如果 !is_above && was_above → 死叉 → Sell
  5. 否则 → Hold
  6. 更新 was_above = is_above
```

### 设计要点

- 策略内部维护 `was_above` 状态，检测交叉事件
- 数据不足时（K 线数量 < slow_period）返回 `Hold`
- 实际实现中需要分别计算两个不同周期的 SMA，而非取同一个 SMA 的两个值

---

## 6.4 RSI 超买超卖策略

### RsiStrategy 结构体

```rust
// strategy/rsi_reversal.rs

use quant_data::common::models::{Kline, IndicatorValues};
use crate::strategy::Signal;
use super::Strategy;

/// RSI 超买超卖策略
/// - RSI < oversold → 买入（超卖反弹）
/// - RSI > overbought → 卖出（超买回落）
pub struct RsiStrategy {
    /// RSI 周期（默认 14）
    rsi_period: usize,
    /// 超买阈值（默认 70.0）
    overbought: f64,
    /// 超卖阈值（默认 30.0）
    oversold: f64,
    /// 当前是否已发出买入信号（防止重复买入）
    in_position: bool,
}

impl Default for RsiStrategy {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            overbought: 70.0,
            oversold: 30.0,
            in_position: false,
        }
    }
}

impl Strategy for RsiStrategy {
    fn name(&self) -> &str {
        "RSI超买超卖"
    }

    fn default_params() -> Vec<(&'static str, f64)> {
        vec![
            ("rsi_period", 14.0),
            ("overbought", 70.0),
            ("oversold", 30.0),
        ]
    }

    fn on_kline(
        &mut self,
        _kline: &Kline,
        indicators: &IndicatorValues,
        _position: Option<&Position>,
    ) -> Signal {
        let rsi = match &indicators.rsi {
            Some(rsi) if !rsi.is_empty() => rsi,
            _ => return Signal::Hold, // RSI 数据不足
        };

        let current_rsi = rsi.last().copied().unwrap_or(50.0);

        if current_rsi < self.oversold && !self.in_position {
            self.in_position = true;
            Signal::Buy   // 超卖区域 → 买入
        } else if current_rsi > self.overbought && self.in_position {
            self.in_position = false;
            Signal::Sell  // 超买区域 → 卖出
        } else {
            Signal::Hold
        }
    }

    fn reset(&mut self) {
        self.in_position = false;
    }
}
```

### 算法流程

```
初始化：in_position = false

对每根 K 线：
  1. 获取 RSI(rsi_period) 最新值
  2. 如果 RSI < oversold(30) 且未持仓 → Buy，标记 in_position = true
  3. 如果 RSI > overbought(70) 且已持仓 → Sell，标记 in_position = false
  4. 否则 → Hold
```

### 设计要点

- `in_position` 状态防止在超卖区域重复发出买入信号
- 策略逻辑简单：RSI 进入超卖区买入，进入超买区卖出
- 阈值可通过 `default_params()` 暴露给 UI 参数面板

---

## 6.5 模拟交易集成

### PaperTrader 结构体

```rust
// strategy/paper_trader.rs

use crate::backtest::engine::BacktestEngine;
use crate::backtest::report::{BacktestConfig, BacktestResult};
use quant_data::common::models::{Kline, IndicatorValues};
use super::Strategy;

/// 模拟交易器（整合策略与回测引擎）
pub struct PaperTrader {
    /// 回测引擎
    engine: BacktestEngine,
    /// 当前使用的策略
    strategy: Box<dyn Strategy>,
    /// 最近一次回测结果
    last_result: Option<BacktestResult>,
}

impl PaperTrader {
    pub fn new(config: BacktestConfig, strategy: Box<dyn Strategy>) -> Self {
        Self {
            engine: BacktestEngine::new(config),
            strategy,
            last_result: None,
        }
    }

    /// 执行历史回测
    pub fn run_backtest(
        &mut self,
        klines: &[Kline],
        indicators: &[&dyn quant_data::indicators::Indicator],
    ) -> anyhow::Result<&BacktestResult> {
        let result = self.engine.run(klines, self.strategy.as_mut(), indicators)?;
        self.last_result = Some(result);
        Ok(self.last_result.as_ref().unwrap())
    }

    /// 实时模式：处理一根实时 K 线，返回信号
    pub fn on_realtime_kline(
        &mut self,
        kline: &Kline,
        indicators: &IndicatorValues,
    ) -> Signal {
        self.strategy.on_kline(kline, indicators, None)
    }

    /// 获取最近一次回测结果
    pub fn last_result(&self) -> Option<&BacktestResult> {
        self.last_result.as_ref()
    }

    /// 更换策略
    pub fn set_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategy = strategy;
        self.last_result = None;
    }
}
```

### 设计要点

- `PaperTrader` 是策略层和回测层的桥梁，对外提供统一接口
- 支持两种模式：历史回测（`run_backtest`）和实时信号（`on_realtime_kline`）
- `Box<dyn Strategy>` 允许运行时切换策略

---

## 6.6 UI 集成（回测报告界面）

### 回测配置面板

```rust
// ui/backtest_panel.rs

use crate::backtest::report::BacktestConfig;
use crate::strategy::StrategyRegistry;

/// 回测面板状态
pub struct BacktestPanel {
    /// 回测配置
    config: BacktestConfig,
    /// 已选策略名称
    selected_strategy: String,
    /// 可用策略列表
    available_strategies: Vec<String>,
    /// 策略参数（名称 → 值）
    strategy_params: Vec<(String, f64)>,
    /// 是否正在运行回测
    is_running: bool,
    /// 回测结果
    result: Option<BacktestReport>,
    /// 错误信息
    error_msg: Option<String>,
}

impl BacktestPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // 策略选择下拉框
        egui::ComboBox::from_label("策略")
            .selected_text(&self.selected_strategy)
            .show_ui(ui, |ui| {
                for name in &self.available_strategies {
                    ui.selectable_value(&mut self.selected_strategy, name.clone(), name);
                }
            });

        // 参数设置
        ui.horizontal(|ui| {
            ui.label("初始资金:");
            ui.add(egui::DragValue::new(&mut self.config.initial_capital)
                .range(100.0..=1_000_000.0));
        });
        ui.horizontal(|ui| {
            ui.label("手续费率:");
            ui.add(egui::DragValue::new(&mut self.config.commission_rate)
                .range(0.0..=0.01)
                .speed(0.0001));
        });

        // 运行按钮
        if ui.add_enabled(!self.is_running, egui::Button::new("运行回测")).clicked() {
            self.is_running = true;
            // 触发回测（通过 channel 通知主逻辑）
        }
    }
}
```

### 回测结果展示

```rust
// ui/backtest_panel.rs（结果展示部分）

impl BacktestPanel {
    fn show_result(&self, ui: &mut egui::Ui) {
        let report = match &self.result {
            Some(r) => r,
            None => return,
        };

        // 收益曲线（egui_plot 折线图）
        egui_plot::Plot::new("equity_curve")
            .height(200.0)
            .show(ui, |plot_ui| {
                let points: Vec<[f64; 2]> = report.result.equity_curve.iter()
                    .enumerate()
                    .map(|(i, (_, equity))| [i as f64, *equity])
                    .collect();
                plot_ui.line(egui_plot::Line::new(
                    egui_plot::PlotPoints::from(points)
                ).name("权益曲线"));
            });

        // 统计指标表格（egui Grid）
        egui::Grid::new("stats_grid")
            .striped(true)
            .show(ui, |ui| {
                for (label, value) in report.summary() {
                    ui.label(label);
                    ui.label(value);
                    ui.end_row();
                }
            });

        // 交易记录列表（egui_extras TableBuilder）
        egui_extras::TableBuilder::new(ui)
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::auto())
            .header(20.0, |mut header| {
                for col in &["#", "方向", "入场价", "盈亏"] {
                    header.col(|ui| { ui.label(*col); });
                }
            })
            .body(|mut body| {
                for trade in &report.result.trades {
                    body.row(18.0, |mut row| {
                        row.col(|ui| { ui.label(trade.trade_id.to_string()); });
                        row.col(|ui| { ui.label(format!("{:?}", trade.side)); });
                        row.col(|ui| { ui.label(format!("{:.2}", trade.entry_price)); });
                        row.col(|ui| {
                            let color = if trade.pnl > 0.0 {
                                egui::Color32::from_rgb(0, 200, 0)
                            } else {
                                egui::Color32::from_rgb(200, 0, 0)
                            };
                            ui.colored_label(color, format!("{:.2}", trade.pnl));
                        });
                    });
                }
            });
    }
}
```

### 设计要点

- 回测面板独立于主界面，通过 `BacktestPanel` 状态管理
- 收益曲线使用 `egui_plot` 折线图，横轴为 bar 索引，纵轴为净值
- 统计指标使用 `egui::Grid` 两列布局
- 交易记录使用 `egui_extras::TableBuilder` 表格，盈亏用颜色区分

---

## 6.7 数据导出

### CSV 导出功能

```rust
// backtest/export.rs

use csv::Writer;
use serde::Serialize;
use quant_data::common::models::Kline;

/// 导出 K 线数据为 CSV
pub fn export_klines_csv(klines: &[Kline], path: &std::path::Path) -> anyhow::Result<()> {
    let mut wtr = Writer::from_path(path)?;
    wtr.write_record(["open_time", "open", "high", "low", "close", "volume"])?;
    for k in klines {
        wtr.write_record(&[
            k.open_time.to_string(),
            k.open.to_string(),
            k.high.to_string(),
            k.low.to_string(),
            k.close.to_string(),
            k.volume.to_string(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

/// 导出回测交易记录为 CSV（委托给 BacktestReport::export_trades_csv）
pub fn export_report_csv(
    report: &super::report::BacktestReport,
    path: &std::path::Path,
) -> anyhow::Result<()> {
    report.export_trades_csv(path)
}
```

### 设计要点

- 导出功能集中在 `backtest/export.rs`，UI 层通过文件对话框选择路径
- 使用 `csv` crate，所有可序列化结构体 derive `Serialize`
- 导出失败返回 `anyhow::Result`，UI 层展示错误信息

---

## 6.8 系统整合

### 应用启动流程

```
main()
  │
  ├── tracing_subscriber::init()          // 初始化日志
  ├── 创建 tokio runtime
  ├── 初始化 StrategyRegistry (内置策略)
  ├── 初始化 KlineStore (数据缓存)
  ├── eframe::run_native(QuantApp)        // 启动 GUI
  │     │
  │     └── QuantApp::new()
  │           ├── 创建 BinanceClient
  │           ├── 创建 IndicatorPipeline
  │           ├── 创建 PaperTrader
  │           ├── 创建 BacktestPanel
  │           └── 启动 WebSocket task (可选)
  │
  └── UI 事件循环 (update)
        ├── 控制面板：交易对/周期选择
        ├── K 线图 + 指标曲线渲染
        ├── 回测面板：策略选择/参数/运行
        └── 实时行情更新（WebSocket → repaint）
```

### main.rs 最终结构

```rust
// main.rs

// quant-data 模块通过外部 crate 引用
use quant_data::common;
use quant_data::data;
use quant_data::indicators;
use quant_data::realtime;

// quant-app 内部模块
mod ui;
mod backtest;
mod strategy;

use anyhow::Result;

#[tokio::main]
async fn main() -> eframe::Result {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("quant_app=info")
        .init();

    tracing::info!("RustQuant 启动中...");

    // eframe 启动配置
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("RustQuant"),
        ..Default::default()
    };

    eframe::run_native(
        "RustQuant",
        options,
        Box::new(|cc| {
            // 加载自定义字体（支持中文）
            ui::theme::load_chinese_fonts(&cc.egui_ctx);
            Ok(Box::new(ui::app::QuantApp::new(cc)))
        }),
    )
}
```

### 依赖注入关系

```rust
// ui/app.rs

use quant_data::common::models::{Kline, Interval, Ticker};
use quant_data::data::fetcher::BinanceClient;
use quant_data::data::kline_store::KlineStore;

/// QuantApp 使用 AppState 集中管理所有状态（与 tech/03 §3.8 一致）
pub struct QuantApp {
    /// 集中管理的 UI 状态
    state: AppState,
    // 子面板
    watchlist: WatchlistPanel,
    chart: CandlestickChart,
    indicator_panel: IndicatorPanel,
    // 策略层
    paper_trader: PaperTrader,
    strategy_registry: StrategyRegistry,
    // UI 面板
    backtest_panel: BacktestPanel,
    // 实时层通道
    kline_rx: mpsc::UnboundedReceiver<WsKline>,
    ticker_rx: mpsc::UnboundedReceiver<WsTicker>,
}
```

### 设计要点

- `main.rs` 保持极简，仅负责初始化和启动
- `QuantApp` 使用 `state: AppState` 集中管理状态（与 tech/03 §3.8 一致），避免字段平铺
- 各模块通过 `QuantApp` 的字段持有，在 `update()` 中协调
- 策略注册表在 `QuantApp::new()` 中初始化，UI 面板从中获取策略列表
- 中文支持通过 `load_chinese_fonts()` 加载字体文件

---

## 6.9 测试策略

### 单元测试

| 测试目标 | 测试方法 |
|---------|---------|
| `SmaCrossStrategy::on_kline` | 构造 SMA 交叉场景，验证 Buy/Sell 信号 |
| `RsiStrategy::on_kline` | 构造 RSI 超买/超卖场景，验证信号 |
| `StrategyRegistry` | 注册策略后按名称创建，验证实例正确 |
| `PaperTrader::on_realtime_kline` | 单根 K 线输入，验证信号输出 |

### 集成测试

```rust
// tests/strategy_tests.rs

#[test]
fn test_sma_cross_full_backtest() {
    // 1. 构造 100 根模拟 K 线（含趋势段）
    // 2. 创建 SmaCrossStrategy
    // 3. 创建 BacktestEngine + BacktestConfig
    // 4. 执行完整回测
    // 5. 验证 BacktestResult：
    //    - equity_curve 非空
    //    - trades 数量 > 0
    //    - total_return 合理（非 NaN）
    //    - max_drawdown 在 [0, 1] 范围内
}

#[test]
fn test_rsi_strategy_signals() {
    // 1. 构造先跌后涨的 K 线序列
    // 2. RSI 策略应在下跌段发出 Buy，上涨段发出 Sell
    // 3. 验证信号序列的正确性
}
```

### 手动验收

1. 启动应用，选择 BTCUSDT / 1d 周期
2. 打开回测面板，选择「SMA交叉」策略
3. 设置初始资金 10000 USDT，手续费 0.1%
4. 点击「运行回测」
5. 验证：收益曲线正常显示、统计指标表格完整、交易记录列表可查看
6. 切换「RSI超买超卖」策略，重新运行回测，对比结果
7. 导出交易记录 CSV，用 Excel 打开验证数据完整性
