# Phase 4：实时层技术实现

## 4.1 模块概述

### 本阶段目标

接入 Binance WebSocket 实时行情，驱动 UI 实时更新：
- 建立 WebSocket 连接到 Binance 实时行情流
- 实时 K 线推送（Kline Stream）
- 24hr Ticker 推送（价格/涨跌幅/成交量）
- 断线重连与错误处理
- 并发任务管理与优雅关闭

### 覆盖的 Rust 知识点

| 知识点 | 在实时层的应用场景 |
|--------|-------------------|
| tokio-tungstenite | WebSocket 客户端连接、消息收发 |
| mpsc channel | 后台任务 → UI 线程的数据传递 |
| Arc<Mutex<>> | 共享状态（订阅列表、连接状态） |
| 并发 task 管理 | tokio::spawn + JoinHandle + CancellationToken |
| serde 流式解析 | WebSocket JSON 消息实时反序列化 |
| 指数退避算法 | 断线重连间隔控制 |

### 与 Phase 3 的衔接

```
Phase 3 (ui)                    Phase 4 (realtime)
┌──────────────┐                ┌──────────────┐
│  QuantApp    │                │ WsClient     │
│  AppState    │◀── mpsc ──────│ (tokio task) │
│  update()    │  try_recv()    │              │
│  ctx.request │                │ Binance WS   │
│  _repaint()  │                │ wss://...    │
└──────────────┘                └──────────────┘
```

- **UI 层**持有 `mpsc::Receiver`，在 `update()` 中 `try_recv()` 消费数据
- **实时层**后台任务持续接收 WebSocket 消息，通过 `mpsc::Sender` 推送
- `ctx.request_repaint()` 唤醒 UI 线程，确保实时数据及时显示

---

## 4.2 WebSocket 客户端

### quant-data/Cargo.toml 补充依赖（Phase 4）

```toml
# quant-data/Cargo.toml 追加

[dependencies]
# 继承 workspace 依赖
tokio-tungstenite = { workspace = true }
futures-util = { workspace = true }
tokio-util = "0.7"          # CancellationToken
anyhow = { workspace = true }
```

### WsClient 结构体

```rust
// src/realtime/ws_client.rs
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct WsClient {
    /// WebSocket 连接地址
    url: String,
    /// 当前订阅的交易对列表
    subscriptions: Arc<Mutex<Vec<String>>>,
    /// 连接状态
    connected: Arc<std::sync::atomic::AtomicBool>,
}

impl WsClient {
    pub fn new() -> Self {
        Self {
            url: "wss://stream.binance.com:9443/ws".to_string(),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            connected: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 建立 WebSocket 连接并开始接收消息
    /// 
    /// 返回 JoinHandle，调用方可通过 abort() 停止任务
    pub async fn connect(
        &self,
        kline_tx: mpsc::UnboundedSender<WsKline>,
        ticker_tx: mpsc::UnboundedSender<WsTicker>,
        cancel: tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<tokio::task::JoinHandle<()>> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        let (mut write, mut read) = ws_stream.split();

        self.connected.store(true, std::sync::atomic::Ordering::SeqCst);

        // 发送订阅消息
        let subs = self.subscriptions.lock().await.clone();
        if !subs.is_empty() {
            let subscribe_msg = serde_json::json!({
                "method": "SUBSCRIBE",
                "params": subs,
                "id": 1
            });
            write.send(Message::Text(subscribe_msg.to_string().into())).await?;
        }

        // 启动消息接收循环
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 接收 WebSocket 消息
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                Self::handle_message(&text, &kline_tx, &ticker_tx);
                            }
                            Some(Ok(Message::Ping(data))) => {
                                // 响应 ping 保持连接
                                let _ = write.send(Message::Pong(data)).await;
                            }
                            Some(Ok(Message::Close(_))) | None => {
                                // 连接关闭
                                break;
                            }
                            _ => {} // 忽略其他消息类型
                        }
                    }
                    // 取消信号
                    _ = cancel.cancelled() => {
                        tracing::info!("WebSocket 任务收到取消信号，正在关闭...");
                        break;
                    }
                }
            }
        });

        Ok(handle)
    }

    /// 解析 WebSocket 消息并分发到对应 channel
    fn handle_message(
        text: &str,
        kline_tx: &mpsc::UnboundedSender<WsKline>,
        ticker_tx: &mpsc::UnboundedSender<WsTicker>,
    ) {
        // 尝试解析为 Kline 消息
        if let Ok(kline) = serde_json::from_str::<WsKline>(text) {
            let _ = kline_tx.send(kline);
            return;
        }
        // 尝试解析为 Ticker 消息
        if let Ok(ticker) = serde_json::from_str::<WsTicker>(text) {
            let _ = ticker_tx.send(ticker);
            return;
        }
        // 订阅确认等消息忽略
        tracing::debug!("未识别的 WS 消息: {}", text);
    }
}
```

### 设计要点

- `connect_async` 建立异步 WebSocket 连接，返回 `SplitStream`/`SplitSink`
- `tokio::select!` 同时监听消息流和取消信号，实现优雅关闭
- Ping/Pong 心跳响应保持连接活跃
- 消息解析采用"尝试匹配"策略：先尝试 Kline，再尝试 Ticker，忽略订阅确认消息

---

## 4.3 实时数据模型

### WebSocket 消息结构体

```rust
// src/realtime/handler.rs
use serde::Deserialize;

/// WebSocket K 线推送格式
/// 对应 stream: <symbol>@kline_<interval>
#[derive(Debug, Clone, Deserialize)]
pub struct WsKline {
    /// 消息事件类型: "kline"
    #[serde(rename = "e")]
    pub event_type: String,
    /// 事件时间 (ms timestamp)
    #[serde(rename = "E")]
    pub event_time: i64,
    /// 交易对符号
    #[serde(rename = "s")]
    pub symbol: String,
    /// K 线数据
    #[serde(rename = "k")]
    pub kline: WsKlineData,
}

/// K 线详细数据（嵌套在 WsKline.k 字段中）
#[derive(Debug, Clone, Deserialize)]
pub struct WsKlineData {
    /// K 线开始时间
    #[serde(rename = "t")]
    pub open_time: i64,
    /// K 线结束时间
    #[serde(rename = "T")]
    pub close_time: i64,
    /// 交易对符号
    #[serde(rename = "s")]
    pub symbol: String,
    /// K 线周期
    #[serde(rename = "i")]
    pub interval: String,
    /// 开盘价
    #[serde(rename = "o")]
    pub open: String,
    /// 收盘价
    #[serde(rename = "c")]
    pub close: String,
    /// 最高价
    #[serde(rename = "h")]
    pub high: String,
    /// 最低价
    #[serde(rename = "l")]
    pub low: String,
    /// 成交量
    #[serde(rename = "v")]
    pub volume: String,
    /// 成交额
    #[serde(rename = "q")]
    pub quote_volume: String,
    /// 成交笔数
    #[serde(rename = "n")]
    pub trades_count: u32,
    /// 是否为已关闭的 K 线
    #[serde(rename = "x")]
    pub is_closed: bool,
}

/// WebSocket 24hr Ticker 推送格式
/// 对应 stream: <symbol>@ticker
#[derive(Debug, Clone, Deserialize)]
pub struct WsTicker {
    /// 消息事件类型: "24hrTicker"
    #[serde(rename = "e")]
    pub event_type: String,
    /// 事件时间
    #[serde(rename = "E")]
    pub event_time: i64,
    /// 交易对符号
    #[serde(rename = "s")]
    pub symbol: String,
    /// 24h 价格变化
    #[serde(rename = "p")]
    pub price_change: String,
    /// 24h 价格变化百分比
    #[serde(rename = "P")]
    pub price_change_percent: String,
    /// 最新价格
    #[serde(rename = "c")]
    pub last_price: String,
    /// 24h 成交量
    #[serde(rename = "v")]
    pub volume: String,
    /// 24h 成交额
    #[serde(rename = "q")]
    pub quote_volume: String,
    /// 24h 最高价
    #[serde(rename = "h")]
    pub high: String,
    /// 24h 最低价
    #[serde(rename = "l")]
    pub low: String,
}

/// 统一 WebSocket 消息枚举
#[derive(Debug, Clone)]
pub enum WsMessage {
    Kline(WsKline),
    Ticker(WsTicker),
}

// === 类型转换 ===

impl WsKline {
    /// 转换为 common::Kline 模型
    pub fn to_kline(&self) -> Option<crate::common::Kline> {
        let k = &self.kline;
        Some(crate::common::Kline {
            open_time: k.open_time,
            open: k.open.parse().ok()?,
            high: k.high.parse().ok()?,
            low: k.low.parse().ok()?,
            close: k.close.parse().ok()?,
            volume: k.volume.parse().ok()?,
            close_time: k.close_time,
            quote_volume: k.quote_volume.parse().ok()?,
            trades_count: k.trades_count,
            is_closed: k.is_closed,
        })
    }
}

impl WsTicker {
    /// 转换为 common::Ticker 模型
    pub fn to_ticker(&self) -> Option<crate::common::Ticker> {
        Some(crate::common::Ticker {
            symbol: self.symbol.clone(),
            price: self.last_price.parse().ok()?,
            volume_24h: self.volume.parse().ok()?,
            price_change_pct: self.price_change_percent.parse().ok()?,
            high_24h: self.high.parse().ok()?,
            low_24h: self.low.parse().ok()?,
            update_time: self.event_time,
        })
    }
}
```

### Binance WebSocket 消息示例

```json
// K 线消息 (btcusdt@kline_1m)
{
  "e": "kline",
  "E": 1672515782136,
  "s": "BTCUSDT",
  "k": {
    "t": 1672515780000,
    "T": 1672515839999,
    "s": "BTCUSDT",
    "i": "1m",
    "o": "16650.01",
    "c": "16650.50",
    "h": "16651.00",
    "l": "16649.50",
    "v": "1.234",
    "n": 42,
    "x": false
  }
}

// Ticker 消息 (btcusdt@ticker)
{
  "e": "24hrTicker",
  "E": 1672515782136,
  "s": "BTCUSDT",
  "p": "250.50",
  "P": "1.53",
  "c": "16650.50",
  "v": "12345.67",
  "q": "205123456.78"
}
```

### 设计要点

- Binance 价格字段均为 `String` 类型（避免浮点精度问题），解析为 `f64` 时使用 `parse().ok()?`
- `#[serde(rename = "e")]` 映射 Binance 的短字段名到 Rust 语义化字段名
- `WsKlineData` 嵌套在 `WsKline.k` 中，对应 Binance 的嵌套 JSON 结构
- `is_closed` 字段区分"进行中"K 线和"已关闭"K 线，UI 可据此决定是否更新最后一根蜡烛

---

## 4.4 数据通道架构

### 通道设计

```rust
// src/realtime/mod.rs
use tokio::sync::mpsc;

/// 实时层通道集合
pub struct RealtimeChannels {
    /// K 线实时数据通道
    pub kline_tx: mpsc::UnboundedSender<WsKline>,
    pub kline_rx: mpsc::UnboundedReceiver<WsKline>,
    /// Ticker 数据通道
    pub ticker_tx: mpsc::UnboundedSender<WsTicker>,
    pub ticker_rx: mpsc::UnboundedReceiver<WsTicker>,
}

impl RealtimeChannels {
    pub fn new() -> Self {
        // 使用 unbounded channel：WebSocket 推送频率可控，不会积压
        let (kline_tx, kline_rx) = mpsc::unbounded_channel();
        let (ticker_tx, ticker_rx) = mpsc::unbounded_channel();
        Self {
            kline_tx, kline_rx,
            ticker_tx, ticker_rx,
        }
    }
}
```

### 通道容量选择

| 通道类型 | 选择 | 理由 |
|---------|------|------|
| `kline_tx/rx` | `unbounded` | K 线推送频率低（每分钟 1 次/交易对），不会积压 |
| `ticker_tx/rx` | `unbounded` | Ticker 推送频率适中（每秒数次），UI 消费速度足够 |

**为什么不用 `bounded`？**
- `bounded` 在满时会阻塞发送方，导致 WebSocket 消息堆积在 tokio-tungstenite 内部
- `unbounded` 允许无限缓冲，但需确保 UI 端及时消费（`try_recv()` 循环）
- 如果未来发现内存占用过高，可切换为 `bounded(1024)` 并丢弃旧数据

### UI 侧消费代码

```rust
// 在 QuantApp::update() 中
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // 非阻塞读取所有待处理 K 线消息
    while let Ok(ws_kline) = self.kline_rx.try_recv() {
        if let Some(kline) = ws_kline.to_kline() {
            self.update_kline(kline);
        }
    }

    // 非阻塞读取所有待处理 Ticker 消息
    while let Ok(ws_ticker) = self.ticker_rx.try_recv() {
        if let Some(ticker) = ws_ticker.to_ticker() {
            self.update_ticker(ticker);
        }
    }

    // ... UI 渲染代码 ...
}
```

---

## 4.5 后台任务管理

### RealtimeService 结构体

```rust
// src/realtime/mod.rs
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct RealtimeService {
    /// WebSocket 客户端
    client: WsClient,
    /// 取消令牌（用于优雅关闭）
    cancel_token: CancellationToken,
    /// 后台任务句柄
    handles: Vec<JoinHandle<()>>,
    /// 通道集合
    channels: RealtimeChannels,
}

impl RealtimeService {
    pub fn new() -> Self {
        Self {
            client: WsClient::new(),
            cancel_token: CancellationToken::new(),
            handles: Vec::new(),
            channels: RealtimeChannels::new(),
        }
    }

    /// 启动实时数据服务
    pub async fn start(&mut self) -> anyhow::Result<()> {
        let handle = self.client
            .connect(
                self.channels.kline_tx.clone(),
                self.channels.ticker_tx.clone(),
                self.cancel_token.clone(),
            )
            .await?;
        self.handles.push(handle);
        tracing::info!("实时数据服务已启动");
        Ok(())
    }

    /// 动态订阅新的交易对
    pub async fn subscribe(&mut self, symbol: &str, interval: &str) -> anyhow::Result<()> {
        let stream_name = format!("{}@kline_{}", symbol.to_lowercase(), interval);
        let mut subs = self.client.subscriptions.lock().await;
        if !subs.contains(&stream_name) {
            subs.push(stream_name);
        }
        // TODO: 发送 SUBSCRIBE 消息到已建立的 WebSocket 连接
        Ok(())
    }

    /// 取消订阅交易对
    pub async fn unsubscribe(&mut self, symbol: &str, interval: &str) -> anyhow::Result<()> {
        let stream_name = format!("{}@kline_{}", symbol.to_lowercase(), interval);
        let mut subs = self.client.subscriptions.lock().await;
        subs.retain(|s| s != &stream_name);
        // TODO: 发送 UNSUBSCRIBE 消息
        Ok(())
    }

    /// 优雅关闭所有后台任务
    pub async fn stop(&mut self) {
        tracing::info!("正在关闭实时数据服务...");
        // 1. 触发取消信号
        self.cancel_token.cancel();

        // 2. 等待所有任务完成
        for handle in self.handles.drain(..) {
            let _ = handle.await;
        }
        tracing::info!("实时数据服务已关闭");
    }

    /// 获取 Ticker 接收端（传递给 UI）
    pub fn take_ticker_rx(&mut self) -> Option<mpsc::UnboundedReceiver<WsTicker>> {
        // 注意：Receiver 不能被 clone，只能转移所有权
        // 实际设计中应在 start() 时将 rx 传递给 UI
        None // placeholder
    }
}
```

### 设计要点

- `CancellationToken` 实现优雅关闭：UI 关闭时调用 `stop()`，所有后台任务收到取消信号后退出
- `subscriptions` 使用 `Arc<Mutex<Vec<String>>>` 共享，支持运行时动态订阅/取消订阅
- `JoinHandle` 存储在 `handles` 中，`stop()` 时 `await` 等待所有任务完成
- 通道 `tx` 端可 `clone()` 传递给多个任务，`rx` 端不可 clone

---

## 4.6 UI 集成

### 在 update() 中消费实时数据

```rust
// src/ui/app.rs
impl QuantApp {
    /// 处理实时 K 线更新
    fn update_kline(&mut self, kline: Kline) {
        let klines = &mut self.state.current_klines;

        if let Some(last) = klines.last_mut() {
            if last.open_time == kline.open_time {
                // 更新最后一根蜡烛（未关闭的 K 线）
                *last = kline;
            } else if kline.open_time > last.open_time {
                // 新 K 线，追加到末尾
                klines.push(kline);
                // 保持最大缓存数量
                if klines.len() > 10000 {
                    klines.remove(0);
                }
            }
        } else {
            klines.push(kline);
        }

        // 重新计算指标
        self.state.recompute_indicators();
        self.state.last_update_time = Some(kline.close_time);
    }

    /// 处理实时 Ticker 更新
    fn update_ticker(&mut self, ticker: Ticker) {
        // 价格闪烁效果：比较新旧价格
        if let Some(ref old_ticker) = self.state.latest_ticker {
            if ticker.symbol == old_ticker.symbol {
                self.price_flash_color = if ticker.price > old_ticker.price {
                    Some(Color32::from_rgb(0, 200, 83))  // 涨：绿色闪烁
                } else if ticker.price < old_ticker.price {
                    Some(Color32::from_rgb(234, 57, 67))  // 跌：红色闪烁
                } else {
                    None
                };
                // 闪烁效果持续 500ms
                if self.price_flash_color.is_some() {
                    self.flash_timer = Some(std::time::Instant::now());
                }
            }
        }

        self.state.latest_ticker = Some(ticker);
        self.state.is_connected = true;
    }
}

impl eframe::App for QuantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. 消费实时数据
        while let Ok(ws_kline) = self.kline_rx.try_recv() {
            if let Some(kline) = ws_kline.to_kline() {
                self.update_kline(kline);
            }
        }
        while let Ok(ws_ticker) = self.ticker_rx.try_recv() {
            if let Some(ticker) = ws_ticker.to_ticker() {
                self.update_ticker(ticker);
            }
        }

        // 2. 检查闪烁效果是否过期
        if let Some(flash_start) = self.flash_timer {
            if flash_start.elapsed().as_millis() > 500 {
                self.price_flash_color = None;
                self.flash_timer = None;
            }
        }

        // 3. UI 渲染（Panel 布局...）
        // ...
    }
}
```

### 数据更新策略

| 数据类型 | 策略 | 说明 |
|---------|------|------|
| K 线（未关闭） | 增量更新最后一根 | `is_closed == false` 时更新最后一根蜡烛 |
| K 线（已关闭） | 追加新蜡烛 | `is_closed == true` 时追加，保持历史完整 |
| Ticker | 全量替换 | 每次推送覆盖 `latest_ticker` |
| 指标 | 增量重算 | 仅在 K 线变化时重新计算，避免每帧重算 |

### 设计要点

- `try_recv()` 非阻塞消费，不阻塞 UI 线程
- K 线更新区分"进行中"和"已关闭"，避免重复追加
- 价格闪烁效果：比较新旧价格，涨绿闪/跌红闪，持续 500ms 后消失
- 指标仅在 K 线变化时重算，避免每帧重复计算

---

## 4.7 并发模型

### 任务拓扑

```
┌─────────────────────────────────────────────────────────────┐
│                    tokio runtime                             │
│                                                             │
│  ┌─────────────────┐    ┌──────────────────────────────┐   │
│  │  egui 主线程     │    │  ws_kline task               │   │
│  │  (UI 渲染)       │◀───│  (K 线 WebSocket)            │   │
│  │                 │    │  tokio-tungstenite            │   │
│  │  update() {     │    │  btcusdt@kline_1m             │   │
│  │    try_recv()   │    └──────────────────────────────┘   │
│  │    render()     │                                        │
│  │  }              │    ┌──────────────────────────────┐   │
│  │                 │    │  ws_ticker task               │   │
│  │  kline_rx ──────│◀───│  (Ticker WebSocket)          │   │
│  │  ticker_rx ─────│◀───│  btcusdt@ticker               │   │
│  └─────────────────┘    └──────────────────────────────┘   │
│                                                             │
│  通信方式：mpsc::UnboundedSender → UI 线程 try_recv()        │
│  关闭方式：CancellationToken → 所有 task 收到取消信号         │
└─────────────────────────────────────────────────────────────┘
```

### 任务间通信矩阵

| 发送方 | 接收方 | 通道类型 | 数据 |
|--------|--------|---------|------|
| `ws_kline` task | UI 主线程 | `mpsc::UnboundedSender<WsKline>` | 实时 K 线 |
| `ws_ticker` task | UI 主线程 | `mpsc::UnboundedSender<WsTicker>` | 实时 Ticker |
| UI 主线程 | `ws_*` tasks | `CancellationToken` | 关闭信号 |

### 优雅关闭流程

```
1. 用户关闭窗口
       │
       ▼
2. eframe 触发 Drop（或显式调用）
       │
       ▼
3. cancel_token.cancel()
       │
       ▼
4. 所有 ws_* task 的 tokio::select! 命中 cancelled() 分支
       │
       ▼
5. WebSocket 连接自动关闭（Drop WebSocket 对象）
       │
       ▼
6. handle.await 等待所有 task 退出
       │
       ▼
7. 主线程退出
```

### 设计要点

- egui 运行在主线程，不可异步；后台任务通过 channel 与 UI 通信
- `CancellationToken` 比 `mpsc` 关闭信号更优雅：支持多个 task 监听同一 token
- 不使用 `Drop` trait 关闭 WebSocket（`drop` 无法 `await`），改用显式 `stop()` 方法
- 任务拓扑简单：1 个 K 线 task + 1 个 Ticker task，不需要复杂的 task 编排

---

## 4.8 错误处理与重连

### 断线检测与指数退避重连

```rust
// src/realtime/ws_client.rs

/// 重连配置
pub struct ReconnectConfig {
    /// 初始重连间隔（毫秒）
    pub initial_delay_ms: u64,
    /// 最大重连间隔（毫秒）
    pub max_delay_ms: u64,
    /// 退避乘数
    pub multiplier: f64,
    /// 最大重连次数（0 = 无限）
    pub max_retries: u32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,   // 1 秒
            max_delay_ms: 60_000,     // 60 秒
            multiplier: 2.0,          // 指数退避
            max_retries: 0,           // 无限重试
        }
    }
}

impl WsClient {
    /// 带自动重连的连接循环
    pub async fn connect_with_reconnect(
        &self,
        kline_tx: mpsc::UnboundedSender<WsKline>,
        ticker_tx: mpsc::UnboundedSender<WsTicker>,
        cancel: CancellationToken,
        config: ReconnectConfig,
    ) {
        let mut current_delay = config.initial_delay_ms;
        let mut retry_count = 0u32;

        loop {
            // 检查取消信号
            if cancel.is_cancelled() {
                break;
            }

            tracing::info!("正在连接 WebSocket: {}", self.url);

            match self.connect_once(&kline_tx, &ticker_tx, cancel.clone()).await {
                Ok(()) => {
                    // 正常退出（收到取消信号）
                    break;
                }
                Err(e) => {
                    retry_count += 1;
                    if config.max_retries > 0 && retry_count > config.max_retries {
                        tracing::error!("达到最大重连次数 ({}), 停止重连", config.max_retries);
                        break;
                    }

                    tracing::warn!(
                        "WebSocket 连接断开: {}，{}ms 后第 {} 次重连...",
                        e, current_delay, retry_count
                    );

                    // 指数退避等待
                    tokio::select! {
                        _ = tokio::time::sleep(std::time::Duration::from_millis(current_delay)) => {}
                        _ = cancel.cancelled() => break,
                    }

                    // 计算下次重连间隔
                    current_delay = ((current_delay as f64) * config.multiplier)
                        .min(config.max_delay_ms as f64) as u64;
                }
            }
        }
    }

    /// 单次连接（断开后返回 Err）
    async fn connect_once(
        &self,
        kline_tx: &mpsc::UnboundedSender<WsKline>,
        ticker_tx: &mpsc::UnboundedSender<WsTicker>,
        cancel: CancellationToken,
    ) -> anyhow::Result<()> {
        // ... 同 4.2 节 connect() 实现 ...
        Ok(())
    }
}
```

### 错误通知到 UI

```rust
// 在 AppState 中添加连接状态
pub struct AppState {
    // ...
    pub ws_status: WsStatus,
}

pub enum WsStatus {
    Connected,
    Reconnecting { retry_count: u32, next_retry_ms: u64 },
    Disconnected { reason: String },
}

// 在状态栏显示
impl QuantApp {
    fn status_bar_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            match &self.state.ws_status {
                WsStatus::Connected => {
                    ui.label(RichText::new("● 已连接").color(Color32::GREEN));
                }
                WsStatus::Reconnecting { retry_count, next_retry_ms } => {
                    ui.label(RichText::new(format!(
                        "● 重连中... (第 {} 次, {}s 后)",
                        retry_count, next_retry_ms / 1000
                    )).color(Color32::YELLOW));
                }
                WsStatus::Disconnected { reason } => {
                    ui.label(RichText::new(format!("● 断开: {}", reason)).color(Color32::RED));
                }
            }
        });
    }
}
```

### 设计要点

- 指数退避：`1s → 2s → 4s → 8s → ... → 60s`（上限），避免频繁重连导致服务器压力
- 重连等待期间也监听 `cancel` 信号，确保关闭时能立即退出
- `WsStatus` 枚举在 UI 状态栏实时展示连接状态，用户可感知网络状况
- 重连成功后需重新发送 `SUBSCRIBE` 消息（新连接无订阅状态）

---

## 4.9 测试策略

### 单元测试：消息解析

```rust
// src/realtime/handler.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ws_kline() {
        let json = r#"{
            "e": "kline",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "k": {
                "t": 1672515780000,
                "T": 1672515839999,
                "s": "BTCUSDT",
                "i": "1m",
                "o": "16650.01",
                "c": "16650.50",
                "h": "16651.00",
                "l": "16649.50",
                "v": "1.234",
                "n": 42,
                "x": false
            }
        }"#;

        let kline: WsKline = serde_json::from_str(json).unwrap();
        assert_eq!(kline.symbol, "BTCUSDT");
        assert_eq!(kline.kline.open, "16650.01");
        assert!(!kline.kline.is_closed);

        let converted = kline.to_kline().unwrap();
        assert!((converted.open - 16650.01).abs() < 0.01);
    }

    #[test]
    fn test_parse_ws_ticker() {
        let json = r#"{
            "e": "24hrTicker",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "p": "250.50",
            "P": "1.53",
            "c": "16650.50",
            "v": "12345.67",
            "q": "205123456.78"
        }"#;

        let ticker: WsTicker = serde_json::from_str(json).unwrap();
        assert_eq!(ticker.symbol, "BTCUSDT");
        assert_eq!(ticker.last_price, "16650.50");
    }
}
```

### 单元测试：通道通信

```rust
#[tokio::test]
async fn test_channel_communication() {
    let (tx, mut rx) = mpsc::unbounded_channel::<WsTicker>();

    // 模拟发送 Ticker
    let ticker = WsTicker {
        event_type: "24hrTicker".into(),
        event_time: 1672515782136,
        symbol: "BTCUSDT".into(),
        price_change: "250.50".into(),
        price_change_percent: "1.53".into(),
        last_price: "16650.50".into(),
        volume: "12345.67".into(),
        quote_volume: "205123456.78".into(),
    };

    tx.send(ticker).unwrap();

    // 验证接收
    let received = rx.recv().await.unwrap();
    assert_eq!(received.symbol, "BTCUSDT");
}
```

### 集成测试：Mock WebSocket 服务器

```rust
// tests/realtime_tests.rs
use tokio_tungstenite::accept_async;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_ws_connect_and_receive() {
    // 1. 启动 mock WebSocket 服务器
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let ws_stream = accept_async(stream).await.unwrap();
        let (mut write, mut read) = ws_stream.split();

        // 发送 mock K 线消息
        let msg = serde_json::json!({
            "e": "kline",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "k": {
                "t": 1672515780000, "T": 1672515839999,
                "s": "BTCUSDT", "i": "1m",
                "o": "16650.01", "c": "16650.50",
                "h": "16651.00", "l": "16649.50",
                "v": "1.234", "n": 42, "x": false
            }
        });
        use tokio_tungstenite::tungstenite::Message;
        write.send(Message::Text(msg.to_string().into())).await.unwrap();
    });

    // 2. 客户端连接到 mock 服务器
    // ... 验证接收到 mock 消息 ...

    server_handle.await.unwrap();
}
```

### 手动验收清单

| 验收项 | 预期结果 |
|--------|---------|
| 启动应用后连接状态 | 状态栏显示"● 已连接"（绿色） |
| 切换交易对 | 新交易对 K 线加载，Ticker 更新 |
| 实时 K 线更新 | 最后一根蜡烛实时变化（未关闭时） |
| 价格闪烁效果 | 价格上涨时绿色闪烁，下跌时红色闪烁 |
| 断网后重连 | 状态栏显示"重连中..."，恢复后自动重连 |
| 关闭窗口 | 应用正常退出，无 panic 或资源泄漏 |

### 设计要点

- 单元测试覆盖 JSON 解析和类型转换，确保 Binance 消息格式变更时快速发现
- 集成测试使用 `TcpListener` 启动 mock 服务器，避免依赖真实网络
- 手动验收是 GUI 测试的核心，自动化测试无法完全替代视觉验证
- `#[tokio::test]` 提供异步测试环境，`mpsc` 通道测试验证数据流正确性
