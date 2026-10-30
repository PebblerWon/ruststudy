# Phase 2.5-3：stock-data ↔ stock-app 融合技术实现

> 前置阅读：[01_data_layer.md](01_data_layer.md)（客户端与轮询 API）、[02_ui_layer.md](02_ui_layer.md)（面板与状态）。
> 本文回答一个核心问题：**async 的 stock-data 如何安全地跑在 gpui 应用里**。

## 3.1 融合要解决的三个问题

| #   | 问题                   | 为什么难                                                                                                                                                                 | 解法（见）   |
| --- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------ |
| 1   | **双 runtime 桥接**    | gpui 自带 smol 系异步 executor；stock-data 是 tokio + reqwest，reqwest 内部依赖 tokio reactor/timer，**直接在 gpui executor 上 `.await` 会 panic（no reactor running）** | §3.3         |
| 2   | **数据流单向且可追溯** | 轮询是 tokio 侧推、UI 是 gpui 侧拉，两侧线程模型不同，不能从 tokio 线程直接碰 Entity                                                                                     | §3.3.4、§3.6 |
| 3   | **限速桶不失效**       | TokenBucket 是 `Arc` 内部 Mutex 共享的——如果 UI 和轮询各建一个 client，限速形同虚设                                                                                      | §3.8         |

## 3.2 Crate 边界与依赖

```
stock-app (bin)  ──依赖──▶  stock-data (lib)
     │                            │
     │ UI / 状态 / 桥接            │ HTTP 客户端 / 解析 / 缓存 / 轮询
     └── 只允许调用 lib.rs 出口 ──▶└── 不 import 任何 gpui 依赖（保持可独立测试）
```

**出口收口**（stock-data/src/lib.rs，当前缺轮询出口且有残留 `fn main()`，顺手清理）：

```rust
// stock-data/src/lib.rs
pub mod common;
pub mod data;

pub use common::{error::DataError, models::*};
pub use data::eastmoney::EastmoneyClient;
pub use data::polling::{spawn_polling, PollingHandle};
```

**调用纪律：**

- stock-app **只 import `stock_data::*` 的公开项**，不深入 `stock_data::data::eastmoney_types` 等内部模块
- 数据模型（Quote/Kline/...）可以穿透到 UI（就是渲染输入）；HTTP 细节、字段映射、解析函数绝不穿透
- stock-data 不感知 UI：不出现 `gpui`/`Entity`/`cx` 字样；跨界通信只通过**函数参数与回调**（如 `spawn_polling` 的 `fetch` 闭包与 `on_data` 回调）

## 3.3 双 runtime 桥接（核心）

### 3.3.1 为什么不能直接 await

```rust
// ❌ 02_ui_layer.md §2.7 的写法在真实 reqwest 下不可行：
cx.spawn(async move |this, cx| {
    let q = client.quotes(&codes).await;   // panic: "must be called from the context of a Tokio 1.x runtime"
    ...
});
```

原因：`cx.spawn` 的 future 由 **gpui 自己的 executor**（smol 系）驱动，线程上没有 tokio runtime 上下文，而 reqwest 的连接池/IO/timer 都挂在 tokio reactor 上。

### 3.3.2 方案：DataBridge（stock-app/src/bridge.rs，新模块）

在 UI 进程内持有一个**独立 tokio Runtime**，两类调用分别走两条桥：

- **一次性请求**（首次加载、切周期、切品种）：gpui 后台线程上 `rt.block_on(...)`——请求亚秒级，短暂占用一个后台线程可接受
- **持续轮询**（Phase 3）：轮询任务整体跑在 tokio runtime 里，通过 **runtime 无关的 async-channel** 把结果推给 UI

```rust
// stock-app/src/bridge.rs
use std::sync::Arc;
use stock_data::{EastmoneyClient, PollingHandle, spawn_polling};

pub struct DataBridge {
    rt: tokio::runtime::Runtime,
    pub client: Arc<EastmoneyClient>,
}

impl DataBridge {
    pub fn new() -> anyhow::Result<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2) // 只跑 HTTP 轮询，2 线程足够
            .enable_all()
            .build()?;
        let client = Arc::new(EastmoneyClient::new()?);
        Ok(Self { rt, client })
    }

    /// 一次性请求：在 gpui 后台线程阻塞驱动 tokio future。
    /// ⚠️ 只允许在 cx.background_spawn 里调用，严禁在 UI（foreground）线程调用。
    pub fn block_on<F: Future>(&self, fut: F) -> F::Output {
        self.rt.block_on(fut)
    }

    /// 启动行情轮询：任务跑在 tokio runtime 内，结果经 tx 推给 UI。
    /// spawn_polling 已泛化（见下方签名说明）：fetch 闭包决定轮询什么，本方法只是行情场景的封装。
    pub fn start_polling_quotes(
        &self,
        codes: Arc<Vec<String>>,                       // 自选列表，轮询期间可通过换 Arc 指向更新
        tx: async_channel::Sender<Vec<stock_data::Quote>>,
    ) -> PollingHandle {
        let client = self.client.clone();
        // spawn_polling 内部 tokio::spawn，必须在 runtime 上下文里调用
        self.rt.block_on(async move {
            spawn_polling(
                move || {                              // fetch：Fn() -> Fut，每个 tick 调一次
                    let client = client.clone();       // Arc::clone 进 future，保证 'static
                    let codes = codes.clone();
                    async move { client.quotes(&codes).await }
                },
                move |quotes| {                        // on_data：成功后怎么消费
                    let _ = tx.try_send(quotes);       // 通道满就丢弃本轮，UI 不欠债
                },
            )
        })
    }
}
```

> 注意 `start_polling_quotes` 只在启动时调用一次（短暂 `block_on` 无妨）；`spawn_polling` 的退避逻辑（1s→30s 封顶）在 stock-data 内部，UI 不感知。
>
> **`spawn_polling` 泛化签名（2026-10-30 定稿）**：轮询框架不再硬编码 `client.quotes`，改为调用者注入"取数策略"。稳定版 Rust 没有 async 闭包，惯用模式是 `Fn() -> Fut`：
>
> ```rust
> pub fn spawn_polling<T, Fut, F, G>(fetch: G, on_data: F) -> PollingHandle
> where
>     G: Fn() -> Fut + Send + 'static,                       // 每 tick 调一次，产出新 future
>     Fut: Future<Output = Result<T, DataError>> + Send + 'static,
>     F: Fn(T) + Send + 'static,                             // 成功后怎么消费（如 tx.try_send）
>     T: Send + 'static,
> ```
>
> 三个关键点：
>
> - fetch 必须是 `Fn`（循环多次调用），所以内部不能 move 捕获物，只能 clone（`Arc::clone` 纳秒级）；
> - 返回的 Fut 必须 `'static`（`tokio::spawn` 要求）——async move 块内自持 Arc 即可满足；
> - 限速/退避/时段感知/watch 停止是骨架，留在 polling.rs，换数据源不碰；换数据源 = 换一个 fetch 闭包（klines/index_quotes/...）。

### 3.3.3 一次性请求模式（面板首次加载 / 切换）

以 K 线面板为例（模式对总览/自选同样适用）：

```rust
fn load_klines(&mut self, cx: &mut Context<Self>) {
    let Some(code) = self.selected.clone() else { return };
    self.load_state = LoadState::Loading;
    cx.notify();

    let bridge = self.bridge.clone();
    let interval = self.interval;
    let beg = self.beg; // chrono::NaiveDate

    // 1) 同步上下文里先把后台任务 spawn 出去（block_on 不占 UI 线程）
    let task = cx.background_spawn(async move {
        bridge.block_on(async {
            stock_data::klines(&code, interval, stock_data::Adjust::Qfq, beg).await
        })
    });

    // 2) 前台任务等结果、回填 Entity
    cx.spawn(async move |this, cx| {
        let result = task.await;
        let _ = this.update(cx, |s, cx| {
            match result {
                Ok(k) => { s.klines = k; s.load_state = LoadState::Ready; }
                Err(e) => { s.load_state = LoadState::Error(e.to_string()); }
            }
            cx.notify();
        });
    });
}
```

要点：

- `cx.background_spawn` 拿到的 `Task` 在 `cx.spawn` 的前台任务里 `.await`——**先 spawn 后 await**，不要在前台 future 里 block_on
- `this.update` 返回 `Result`（Entity 可能已释放），用 `let _ =` 吞掉即可；窗口关闭后任务自然终止
- 请求进行中再次触发（快速切周期）：旧 task 的回填会覆盖新数据。MVP 接受；如需严格，给请求加自增 `epoch: u64`，回填时校验 `if epoch != self.epoch { return }`

### 3.3.4 轮询模式（Phase 3）

跨 runtime 的通道必须**两个 executor 都能用**：`async-channel`（基于计数信号量，不绑定 runtime）。Cargo.toml 增加：

```toml
# stock-app/Cargo.toml
async-channel = "2"
```

UI 侧消费循环（`StockApp::new` 里启动一次）：

```rust
// StockApp::new 中：
let (tx, rx) = async_channel::unbounded::<Vec<Quote>>();
let watchlist = Arc::new(vec!["600519".into(), "000001".into()]);   // 实际从 watchlist.json 读
self.polling_handle = Some(self.bridge.start_polling_quotes(watchlist, tx));   // tokio 侧生产
cx.spawn(async move |this, cx| {
    while let Ok(quotes) = rx.recv().await {                 // gpui 侧消费
        let _ = this.update(cx, |s, cx| {
            s.apply_quotes(quotes);      // 分发给 OverviewPanel / 自选面板（见 §3.5）
            s.last_refresh = Some(chrono::Local::now());
            cx.notify();
        });
    }
}).detach();
```

要点：

- `rx.recv().await` 挂在 gpui executor 上，tokio 侧 `try_send` 后 gpui 立即被唤醒，无轮询空转
- 事件处理逻辑抽成 `fn apply_quotes(&mut self, quotes: Vec<Quote>)` 纯状态更新函数，可脱离 GUI 单测
- **三类数据合并进一个轮询任务**（指数→自选→快照顺序拉，共享同一个限速桶），不要每类各开一个任务——见 [01_data_layer.md §1.6](01_data_layer.md)

### 3.3.5 备选方案对比

| 方案                                | 做法                          | 优点                 | 缺点                                        | 结论          |
| ----------------------------------- | ----------------------------- | -------------------- | ------------------------------------------- | ------------- |
| **A. block_on 桥接**（§3.3.2）      | 后台线程 `rt.block_on`        | 零新依赖、代码量最小 | 占用一个后台线程（亚秒级）                  | ✅ 一次性请求 |
| **B. async-channel 推送**（§3.3.4） | tokio 任务 → 通道 → gpui 消费 | 事件驱动、及时唤醒   | 引一个 crate                                | ✅ 轮询       |
| C. UI 定时器拉取                    | gpui timer 每 1s `try_recv`   | 零新依赖             | 延迟粒度 1s、空转                           | ❌ 不采用     |
| D. 数据全同步（blocking reqwest）   | `reqwest::blocking`           | 无双 runtime 问题    | 轮询退避/timer 都得自己造；阻塞 UI 线程风险 | ❌ 不采用     |

## 3.4 启动时序（main.rs 改造）

```
main
 ├─ tracing_subscriber::fmt().with_env_filter(...).init()   // RUST_LOG=info
 ├─ DataBridge::new()? → Arc::new(...)                       // tokio Runtime + 全局唯一 client（main 里 ? 合法）
 ├─ application().run(|cx|
 │    ├─ open_window → StockApp::new(window, cx, bridge)
 │    │    ├─ 创建面板 Entity（全部 Loading 态）
 │    │    ├─ async_channel 建立轮询通道，start_polling_quotes(watchlist, tx) → PollingHandle 存 StockApp
 │    │    └─ cx.spawn 轮询消费循环
 │    └─ 各面板发起首次一次性请求（§3.3.3）
 └─ 窗口关闭 → 进程退出 → Runtime drop（兜底终止一切 tokio 任务）
```

对应到现有 `main.rs`：开头加 tracing 初始化；在 `application().run` **之前**构建 `DataBridge`，以参数传入 `StockApp::new`：

```rust
// main.rs（示意）
let bridge = Arc::new(DataBridge::new()?);   // main 可返回 Result，? 在这里合法
// ...
open_window(..., |window, cx| cx.new(|cx| StockApp::new(window, cx, bridge.clone())))
// ↑ bridge 后续不再用时可直接 move（open_window 的 build 闭包是 FnOnce，move 合法）；
//   写 .clone() 是防御性习惯：Arc::clone 仅原子计数 +1，将来 main 里再加多窗口/退出清理时不用改

// app.rs（示意）
pub fn new(_window: &mut Window, cx: &mut Context<Self>, bridge: Arc<DataBridge>) -> Self {
    // ...
    let polling_handle = bridge.start_polling_quotes(watchlist, tx);   // Arc 自动解引用为 &DataBridge
    // ...
}
```

> ⚠️ **`DataBridge::new()?` 不能写进 `StockApp::new` 里**（2026-10-30 实测踩坑，E0277）：
> `StockApp::new` 返回 `Self` 而非 `Result`，`?` 无法从非 Result 函数传播错误；且 gpui 的
> `cx.new(|cx| ...)` 闭包不可失败，View 构造函数被框架限定为返回 `Self`。
> 因此**一切可能失败的资源构建（Runtime、HTTP client、配置加载）必须在进入 gpui 之前完成**，
> 构建产物经参数传入。
>
> 同场景两个连带陷阱：
>
> - 不要写 `DataBridge::new().ok()` + 事后 `unwrap()`：`unwrap` 会把值从 `Option` 里 **move**
>   走，后续再用 `bridge` 就是 E0382；且 `DataBridge` 内含 `Runtime`（非 Clone），本体永远
>   不可 clone，跨任务/跨面板共享只允许 `Arc::clone`
> - 字段类型写 `bridge: Arc<DataBridge>`（见 §3.5），不要 `Option<DataBridge>` 或裸值——
>   §3.3.3 的 `self.bridge.clone()` 克隆的是 Arc 句柄，才能被 move 进 `background_spawn`

## 3.5 状态归属与共享

| 数据/对象                        | 归属            | 形式                                                              | 更新时机                   |
| -------------------------------- | --------------- | ----------------------------------------------------------------- | -------------------------- |
| `DataBridge`（Runtime + client） | `StockApp`      | `Arc<DataBridge>`                                                 | 启动创建，全程只读共享     |
| `PollingHandle`                  | `StockApp`      | `Option<PollingHandle>`                                           | 启动存入；退出时 `stop()`  |
| 轮询通道 rx                      | `StockApp`      | `async_channel::Receiver`                                         | 消费循环持有               |
| 指数 + 市场统计                  | `OverviewPanel` | `Vec<IndexQuote>` + `Option<MarketSnapshot>` + 自己的 `LoadState` | 首次请求 + 轮询            |
| 榜单数据                         | `RanklistPanel` | 同上                                                              | 首次请求 + 轮询（Phase 4） |
| 自选列表 codes                   | `StockApp`      | `Vec<String>`（watchlist.json）                                   | 启动加载、用户编辑时写盘   |

**更新分发**：`apply_quotes(quotes)` 收到全市场无关的自选快照后，按代码前缀拆分、调用对应面板 Entity 的更新方法（面板内部再 `cx.notify()`）；面板不自己发网络请求消费轮询，只有首次加载自己发一次性请求。

## 3.6 数据流总图

```
                       stock-app (UI 进程)
┌────────────────────────────────────────────────────────────────┐
│  gpui executor (foreground)      gpui thread pool (background) │
│  ┌──────────────┐  Task.await   ┌────────────────────────────┐ │
│  │ 面板 Entity   │◀─────────────│ background_spawn(          │ │
│  │  update+notify│              │   bridge.block_on(client   │ │
│  └──────▲───────┘              │   .klines(...)) )          │ │
│         │ apply_quotes         └──────────────┬─────────────┘ │
│  ┌──────┴──────────────┐                      │ block_on      │
│  │ cx.spawn 轮询消费    │                      ▼               │
│  │ rx.recv().await     │        ┌─────────────────────────┐   │
│  └──────▲──────────────┘        │   tokio Runtime (2 线程) │   │
│         │ try_send              │  ┌───────────────────┐  │   │
│  ┌──────┴──────────────┐        │  │ spawn_polling      │  │   │
│  │ async_channel (tx)──┼────────┼─▶│  时段感知+退避      │  │   │
│  └─────────────────────┘        │  └─────────┬─────────┘  │   │
│                                 └────────────┼────────────┘   │
└──────────────────────────────────────────────┼─────────────────┘
                                               ▼
                                    EastmoneyClient（Arc 共享，
                                    内部 TokenBucket + TtlCache）
                                               │
                                               ▼
                                   push2 / push2his.eastmoney.com
```

- **拉**（一次性）：面板 → `background_spawn` → `block_on` → tokio → HTTP → 回填 → `cx.notify()`
- **推**（轮询）：tokio `spawn_polling` → `async_channel` → gpui 消费循环 → Entity 更新 → `cx.notify()`

## 3.7 错误传递与状态栏

| 层         | 错误形态                 | 去向                                                                      |
| ---------- | ------------------------ | ------------------------------------------------------------------------- |
| stock-data | `DataError`（thiserror） | 原样上抛，`Display` 带上下文（如"时间解析失败 {line}"）                   |
| 一次性请求 | `Result<T, DataError>`   | 面板 `LoadState::Error(e.to_string())` + 重试按钮                         |
| 轮询       | 内部退避，不中断         | MVP：状态栏比较 `last_refresh` 与当前时段间隔，超过 3 个周期显示"重连中…" |

- UI 层**不再包装新错误类型**，直接 `to_string()` 展示；`InvalidCode`（北交所等）在输入侧就拦截，给行内提示而非 Error 态
- 停牌（`is_suspended`）不是错误，是正常数据，UI 按"停牌"渲染

## 3.8 限速与并发约束

1. **全局唯一 `Arc<EastmoneyClient>`**——TokenBucket 在 client 内部，两个 client = 两个桶 = 限速失效
2. 轮询任务内部顺序拉取（不并发发多个请求），一次性请求与轮询天然被同一个桶串行化
3. `with_base_url` 仅测试用；生产路径禁止绕过 client 直发 HTTP

## 3.9 退出与清理

- `PollingHandle.stop()`（watch 通知）→ 轮询任务下个 tick 前退出；**drop 不停任务**（[01 §1.6.3](01_data_layer.md) 设计如此）
- MVP 进程退出兜底：窗口关闭 → 进程结束 → `Runtime` drop → 所有 tokio 任务终止，不严格依赖手动 stop
- 若后续做多窗口/热重载，再在窗口 onClose 里显式 `stop()`

## 3.10 测试策略

| 内容                    | 位置                                                | 方式                                                                                                          |
| ----------------------- | --------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| HTTP/解析/缓存/轮询退避 | stock-data（已有规划，[01 §1.7](01_data_layer.md)） | wiremock + `with_base_url`                                                                                    |
| 轮询事件 → 状态更新     | stock-app                                           | `apply_quotes` / `apply_poll_event` 抽成纯函数，普通 `#[test]` 直测，不建窗口                                 |
| block_on 桥接           | stock-app                                           | 一个 `#[test]`：`DataBridge` 用 `with_base_url` 指向 wiremock，`rt.block_on(client.quotes(...))` 断言解析成功 |
| UI 渲染                 | —                                                   | 手动验收清单（[02 §2.8](02_ui_layer.md)）                                                                     |

## 3.11 落地顺序与现状对照

当前代码状态（2026-10-30）与落地顺序：

| 步骤 | 内容                                                                                             | 现状                                                                                                                                                         |
| ---- | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1    | stock-data 补 `klines()` / `timeline()`（[01 §1.5.6](01_data_layer.md)）                         | ◐ 解析函数已在 eastmoney_types.rs（`parse_kline_line`/`parse_timeline_line`），client 方法未加                                                               |
| 2    | stock-data 补 `polling.rs`（[01 §1.6.3](01_data_layer.md)）+ lib.rs 出口收口、删残留 `fn main()` | ◐ polling.rs 已实现（时段感知 + 退避 + watch stop），lib.rs 已导出；待泛化 `spawn_polling` 签名为 `fetch + on_data` 双回调（见 §3.3.2，2026-10-30 定稿）     |
| 3    | stock-app 新增 `bridge.rs`（§3.3.2）+ main.rs 加 tracing 初始化                                  | ✅ 已完成（2026-10-30）：tracing 已初始化；bridge 在 main.rs 构建、参数传入；**⚠️ 必须有窗口生命周期的对象持有 `Arc<DataBridge>`（见 §3.4.1 生命周期陷阱）** |
| 4    | 面板接入一次性请求（§3.3.3）：总览先行，验证融合通路                                             | ◐ OverviewPanel 轮询通道已跑通（2026-10-30 验证：能收到真实 8 指数并更新）；一次性请求待接                                                                   |
| 5    | Phase 3：轮询通道接入（§3.3.4）+ 状态栏                                                          | ◐ OverviewPanel 已接入 index_quotes 轮询；其余面板/状态栏待接                                                                                                |

**一致性备忘**：02_ui_layer.md §2.7 的轮询示例按本文 §3.3.4 修正（直接 `.await` client 方法在 gpui executor 上不可行）；该文档标注 gpui-kit 0.7.3，而 stock-app/Cargo.toml 已是 **0.8.0**，实现时以实际版本 API 为准并回写文档。

### 3.11.1 生命周期陷阱：DataBridge 必须被窗口持有（2026-10-30 实录）

**症状**：轮询任务启动后 2～4 秒全部静默，UI 永远收不到数据；reqwest 超时不触发、进程不退出、线程全在 Wait，看似“tokio runtime 被冻结”。

**根因**：gpui Windows 平台的 `run(on_finish_launching)`（gpui-pre-windows platform.rs）在**调用完启动闭包后立即丢弃闭包**，随后才进入消息循环。若 `Arc<DataBridge>` 只被 main 的启动闭包捕获、且 StockApp/面板都不存储它，则窗口打开完成后 Arc 计数归零 → `DataBridge` drop → 内含的 tokio `Runtime` drop → **所有 tokio 任务（轮询、定时器）被取消**。gpui 主循环不受影响，所以进程看起来一切正常。

**规则**：

- `Arc<DataBridge>` 必须由至少一个**窗口生命周期的对象**持有（现由 `StockApp._bridge` 与 `OverviewPanel._bridge` 同时持有，Runtime 活到窗口关闭）；
- 任何内嵌 `tokio::runtime::Runtime` 的结构体同理：谁 spawn 任务，谁就要保证该结构体活得比任务久；
- 诊断这类问题可给结构体加临时 `impl Drop` 打印，直接验证 drop 时机。

**融合完成验收：**

- [ ] 盘中启动：总览面板 3s 内显示 8 指数 + 市场统计（走一次性请求）
- [ ] 轮询接入后：自选价格 5s 级自动跳动，UI 不卡顿（数据在后台线程/tokio，UI 线程只渲染）
- [ ] 断网：状态栏进入"重连中"，恢复后自动续刷，全程无 panic
- [ ] `cargo test -p stock-data -p stock-app` 全绿；`cargo run -p stock-app` 关窗后进程正常退出
