# 详情页：secid 参数化通用方案（指数先行，T4.6）

> 对应 DEV_PLAN **T4.6**；分时图部分落地 **T4.1**（trends2 当日分时 + 均价线 + 昨收基准）。
> **通用化修订（2026-11-20）**：详情页参数为完整 **secid**（"1.000001"/"0.600519"），指数与个股共用同一页面骨架、模型与数据链路；差异只在入口侧 secid 来源（指数查表 / 个股规则函数）与个股扩展字段渲染。本期 MVP 只落地指数入口，个股接入路径见 §4.10（通道已留，接入约半天）。文件名沿用 `04_index_detail.md`，避免引用链接失效。
> 前置阅读：[01_data_layer.md §1.4.3 / §1.5.4](01_data_layer.md)（INDEX_SECIDS 与 trends2 实测契约）、[03_integration.md §3.3](03_integration.md)（DataBridge 桥接模式）。
> 本文涉及的 gpui-kit 0.8.0 chart API 均已对照 cargo registry 的 gpui-component 0.8.0 源码核对（§4.6），实现时如遇 API 不符以实际版本为准并回写本文。

## 4.0 功能概述与范围

一句话：**详情页按 secid 参数化，指数与个股共用页面骨架与数据链路；本期 MVP：总览页指数卡片可点击，进入该指数的详情页（行情头 + 当日分时图），返回按钮 / 侧边栏切换退出详情页。**

### 4.0.1 页面原型

（以指数为例；个股详情仅多一行换手/PE/PB/市值指标，见 §4.10.2 第 3 步）

```
┌──────────────────────────────────────────────────────────┐
│ ← 返回   上证指数  000001                                 │
│ 3869.23   +12.45  +0.32%                                 │
├──────────────────────────────────────────────────────────┤
│ 今开 3855.10   最高 3875.66   最低 3848.02   昨收 3856.78 │
│ 成交额 5612.34亿   成交量 4.8亿手                         │
├──────────────────────────────────────────────────────────┤
│ 3880.00 ─┐                                    ┌───────   │ ← y 刻度（价格）
│          │ \      ____价格线（红/绿随昨收）    /          │
│ 3856.78 ─┼───·────·─────·─────·─────·───（昨收虚线）      │
│          │    \___/ 均价线（橙黄）        \___/           │
│ 3830.00 ─┴─────────────────────────────────────────      │
│          09:30          11:30|13:00           15:00      │ ← x 标签 3 档
└──────────────────────────────────────────────────────────┘
```

### 4.0.2 范围界定

| 做（MVP）                                                                                                                                       | 不做（记录为后续扩展）                                                                        |
| ----------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| **通用骨架**：`DetailTarget`（Index+Stock 双变体）/ `DetailQuote`（个股扩展字段 Option）/ `quote_detail` / `quote_timeline` 全部按 secid 参数化 | 个股详情入口落地（通道已留，见 §4.10；榜单入口还需先补 StockBoard 的领涨股 code/market 字段） |
| 6 张指数卡片全部可点（上证指数为验收主案例）                                                                                                    | 个股专属功能：日 K / 多周期 / 资金流 / F10                                                    |
| 行情头：价格/涨跌/开高低收/成交额/成交量                                                                                                        | 分时成交量副图（量柱）                                                                        |
| 当日分时图：价格线 + 均价线 + 昨收虚线 + 固定 242 槽位 + hover tooltip                                                                          | 5 日分时（ndays>1 数据跨日不连续，见 §4.8 坑 6）                                              |
| 轮询实时刷新（盘中 5s / 收盘 60s，随页销毁）                                                                                                    | y 轴涨跌幅双刻度（东财右轴样式）                                                              |
| 交易时段感知、退避重试（复用 spawn_polling 现成骨架）                                                                                           | 北交所个股 secid 规则扩展（现规则拒绝 8/4 开头）                                              |

### 4.0.3 涉及文件总表

| 文件                                            | 动作     | 内容                                                              |
| ----------------------------------------------- | -------- | ----------------------------------------------------------------- |
| `crates/stock-data/src/data/normalize.rs`       | 增量     | `shou_to_wanshou()`                                               |
| `crates/stock-data/src/common/models.rs`        | 增量     | `DetailQuote` 模型                                                |
| `crates/stock-data/src/data/eastmoney_types.rs` | 增量     | `parse_quote_detail_row()`                                        |
| `crates/stock-data/src/data/eastmoney.rs`       | 增量     | `index_secid()` / `quote_detail()` / `quote_timeline()`           |
| `crates/stock-data/src/lib.rs`                  | 增量     | 出口 `index_secid` / `INDEX_SECIDS`                               |
| `crates/stock-app/src/bridge.rs`                | 增量     | `start_polling_quote_detail()` / `start_polling_quote_timeline()` |
| `crates/stock-app/src/ui/polling_util.rs`       | **新建** | 跨面板泛型轮询启动模板（从 OverviewPanel 抽取）                   |
| `crates/stock-app/src/ui/detail_panel.rs`       | **新建** | 详情页面板 + 分时图                                               |
| `crates/stock-app/src/ui/app.rs`                | 增量     | `DetailTarget` / `active_detail` / observe / 渲染分流             |
| `crates/stock-app/src/ui/overview_panel.rs`     | 增量     | 卡片 `secid` / `on_click` / `cursor_pointer`                      |
| `crates/stock-app/src/ui/theme.rs`              | 增量     | 均价线语义色常量                                                  |
| `crates/stock-app/src/ui/mod.rs`                | 增量     | `pub mod polling_util; pub mod detail_panel;`                     |

---

## 4.1 导航模型：app_state 单一事实源

### 4.1.1 决策

详情页导航走 **`StockAppState` 共享实体 + observe 消费**，与现有 `all_a_codes` → `handle_app_state_change` 的模式完全同构，不引入新的通信机制。

备选方案对比（为什么不选）：

| 方案                              | 做法                                                                           | 结论                                                                                      |
| --------------------------------- | ------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| **A. app_state 请求字段**（采用） | 入口点击写 `active_detail`（DetailTarget::Index/Stock），StockApp observe 消费 | ✅ 与既有模式同构；指数与个股入口共用同一通道，将来榜单点击领涨股打开个股详情页零协议改动 |
| B. OverviewPanel 构造传入回调闭包 | 闭包持有 StockApp 的 WeakEntity                                                | ❌ `StockApp::new` 里先建 panel 后有 self，构造顺序死锁                                   |
| C. 详情页加为侧边栏菜单项         | `AppPanel::Detail(...)`                                                        | ❌ 详情页不是导航目标，是叠加页面；菜单里不该出现                                         |
| D. OverviewPanel 内部自切换       | panel 自己 render 详情页                                                       | ❌ 导航职责散落到面板；侧边栏切换无法感知，轮询清理要跨两层协调                           |

### 4.1.2 状态与类型（app.rs）

```rust
// app.rs

/// 详情页目标——**secid 参数化的通用导航协议**。
/// 全链路只携带完整 secid（"1.000001"/"0.600519"），任何环节禁止从 code 反推市场
/// （000001 沪是上证指数、深是平安银行，§4.8 坑 1）。
/// MVP 仅落地 Index 入口；Stock 变体骨架同步就位，接入路径见 §4.10。
#[derive(Clone, PartialEq)]
pub enum DetailTarget {
    Index {
        secid: String, // 完整 secid："1.000001"——经 index_secid() 查表获得
        code: String,  // 展示用 "000001"
        name: String,  // 展示用 "上证指数"
    },
    Stock {
        secid: String, // 完整 secid："0.600519"——经 secid() 规则函数或 f13.f12 拼接
        code: String,
        name: String,
    },
}

impl DetailTarget {
    /// 用访问器代替解构：三个字段所有变体同名，match 穷尽收敛在这一处；
    /// 将来加变体（如 ETF）面板/桥接层零改动
    pub fn secid(&self) -> &str {
        match self {
            Self::Index { secid, .. } | Self::Stock { secid, .. } => secid,
        }
    }
    pub fn code(&self) -> &str {
        match self {
            Self::Index { code, .. } | Self::Stock { code, .. } => code,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Index { name, .. } | Self::Stock { name, .. } => name,
        }
    }
}

pub struct StockAppState {
    pub all_a_codes: Option<Arc<Vec<String>>>,
    pub active_detail: Option<DetailTarget>, // 新增：Some = 详情页打开
}

pub struct StockApp {
    // ...现有字段不动...
    detail: Option<Entity<DetailPanel>>, // 新增：已创建的详情页 Entity 缓存
}
```

### 4.1.3 状态时序

```
打开：OverviewPanel 卡片 on_click
      └─▶ app_state.active_detail = Some(Index{..}) + notify
            └─▶ StockApp::handle_detail_change（observe 触发）
                  ├─ 与当前 detail.target 相同 → 忽略（防重复创建）
                  └─ 不同/未打开 → stop 旧轮询 → cx.new(DetailPanel) → 缓存
                        └─▶ render_main_content：detail 优先渲染

返回：DetailPanel "← 返回" on_click
      └─▶ app_state.active_detail = None + notify
            └─▶ StockApp::handle_detail_change
                  └─▶ close_detail()：stop_polling() → Entity 释放

切换详情：详情页打开时点击另一张指数卡片
      └─▶ 同上，target 不同 → 旧面板 stop + 替换

侧边栏切换：菜单 on_click
      └─▶ active_key = key；app_state.active_detail = None（observe 完成清理）
```

### 4.1.4 StockApp 增量代码

```rust
// app.rs — StockApp::new() 末尾追加
cx.observe(&app_state, Self::handle_detail_change).detach();

// 详情页请求消费：Some → 创建/替换；None → 关闭
fn handle_detail_change(&mut self, _e: Entity<StockAppState>, cx: &mut Context<Self>) {
    let target = self.app_state.read(cx).active_detail.clone();
    match target {
        None => self.close_detail(cx),
        Some(t) => {
            // 与当前详情页相同则忽略；不同则先停旧轮询再重建
            let stale = self.detail.as_ref().map_or(true, |p| p.read(cx).target() != &t);
            if stale {
                self.close_detail(cx);
                let bridge = self._bridge.clone();
                let app_state = self.app_state.clone();
                self.detail = Some(cx.new(|cx| {
                    DetailPanel::new(bridge, t, app_state, cx)
                }));
            }
        }
    }
    cx.notify();
}

/// 关闭详情页。⚠️ PollingHandle drop 不会停止任务（01 §1.6.3 设计如此），
/// 必须显式 stop，否则返回总览后每 5s 仍在打 quote_detail + trends2
fn close_detail(&mut self, cx: &mut Context<Self>) {
    if let Some(p) = self.detail.take() {
        p.read(cx).stop_polling();
    }
}
```

### 4.1.5 渲染分流与侧边栏清理（app.rs）

```rust
// render_main_content 内：详情页存在时整体替换菜单面板内容
let content_elm = if let Some(detail) = &self.detail {
    detail.clone().into_any_element()
} else if let Some(panel) = content_panel {
    match panel {
        AppPanel::Overview(v) => v.clone().into_any_element(),
        AppPanel::Ranklist(v) => v.clone().into_any_element(),
    }
} else {
    div().child("开发中...").into_any_element()
};

// render_sidebar 的菜单 on_click 追加：显式导航 = 关闭详情页
.on_click(cx.listener(move |this, _, _, cx| {
    this.active_key = key.clone();
    // 经 app_state 单一事实源，observe 负责轮询清理；不要直接动 this.detail
    this.app_state.update(cx, |s, cx| {
        s.active_detail = None;
        cx.notify();
    });
    cx.notify();
}))
```

> ⚠️ 关闭后回到的是**当前菜单面板**（现状默认 `active_key = "ranklist"`），不要把"返回"写死成渲染总览。

---

## 4.2 数据层增量（stock-data）

### 4.2.1 index_secid —— 代码查表（eastmoney.rs）

**为什么必须查表而不能复用 `secid()`**：`secid()` 是个股市场规则（00/30 开头 → 深市 `0.`）。指数代码 `000001` 会被映射成 `0.000001`——深市**平安银行**；`399001` 的 39 前缀直接被拒。指数与个股是两套 secid 体系，01 §1.4.3 已明确"指数走显式 secid 常量，不经 secid() 规则函数"。

> **个股入口不走这张表**：`secid()` 规则函数对个股完全可靠（60/68→`1.`、00/30→`0.`——它的缺陷只在指数代码撞车）；数据源自带 f13（market）+ f12（code）时优先 `f13.f12` 拼接，连规则都不用猜。两套入口各自可靠，详情页内部零反推（§4.10.1）。

```rust
// eastmoney.rs — INDEX_SECIDS 旁

/// 指数代码 → 完整 secid。查表而非猜市场：
/// "000001" 在沪是上证指数、在深是平安银行，按代码前缀猜必错
pub fn index_secid(code: &str) -> Option<&'static str> {
    INDEX_SECIDS
        .iter()
        .copied()
        .find(|s| s.split_once('.').map(|(_, c)| c == code).unwrap_or(false))
}

#[cfg(test)]
mod index_secid_tests {
    use super::*;

    #[test]
    fn lookup_rules() {
        assert_eq!(index_secid("000001"), Some("1.000001")); // 上证指数
        assert_eq!(index_secid("399001"), Some("0.399001")); // 深证成指
        assert_eq!(index_secid("600519"), None);             // 个股不在表内
        assert_eq!(index_secid(""), None);
    }
}
```

`lib.rs` 出口补一行（03 §3.2 出口收口纪律）：

```rust
pub use data::eastmoney::{index_secid, INDEX_SECIDS, EastmoneyClient, spawn_polling, PollingHandle};
```

### 4.2.2 DetailQuote 模型（models.rs）

详情页行情头需要的字段比 `IndexQuote`（只有价格/涨跌/成交额）多开高低收与成交量。新建通用模型 `DetailQuote` 而不是复用 `Quote`——`Quote` 是个股全字段模型，指数没有市值/换手率（f20/f21/f8 返 `"-"`），复用会带进一堆恒 0 字段误导渲染；反向同理，个股扩展字段在 `DetailQuote` 里全部 `Option` 化，指数详情自然为 `None`，一套模型两种证券。

需要补一个解析 helper（eastmoney_types.rs 现有 helper 只有 `f64_field`/`str_field`/`take_data`）：

```rust
/// `"-"`（停牌/指数无此字段）→ None，数字 → Some；个股扩展字段专用
pub fn opt_f64_field(row: &Value, key: &str) -> Option<f64> {
    row[key].as_f64()
}
```

```rust
// models.rs 追加

/// 详情页快照（行情头）——指数/个股通用，按 secid 参数化
/// ⚠️ 单位契约：成交额亿元、成交量万手（出口归一，UI 禁止再换算）
/// 个股扩展字段全部 Option：指数接口返 "-" → None → UI 不渲染该行（无需类型判断）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DetailQuote {
    pub code: String,    // "000001"（指数）/ "600519"（个股）
    pub name: String,    // "上证指数" / "贵州茅台"
    pub price: f64,
    pub change: f64,
    pub change_pct: f64, // %
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub pre_close: f64,
    pub amount: f64,     // 亿元
    pub volume: f64,     // 万手（个股极小成交量可能 <0.1，MVP 统一 {:.2} 万手，不做动态单位）
    // ---- 个股扩展（指数为 None；字段号见 01 §1.5 ulist 契约）----
    pub turnover_rate: Option<f64>, // 换手率 %（f8）
    pub pe: Option<f64>,            // 市盈率·动（f9）
    pub pb: Option<f64>,            // 市净率（f23）
    pub total_mv: Option<f64>,      // 总市值 亿元（f20，元→亿）
    pub circ_mv: Option<f64>,       // 流通市值 亿元（f21，元→亿）
}
```

### 4.2.3 normalize.rs 增量

```rust
/// 手 → 万手（指数成交量展示口径）
pub fn shou_to_wanshou(v: f64) -> f64 { v / 1e4 }
```

### 4.2.4 parse_quote_detail_row（eastmoney_types.rs）

```rust
/// ulist 行 → DetailQuote（出口单位：amount 亿元、volume 万手；个股扩展字段 "-" → None）
pub fn parse_quote_detail_row(row: &Value) -> DetailQuote {
    DetailQuote {
        code: str_field(row, "f12").to_string(),
        name: str_field(row, "f14").to_string(),
        price: f64_field(row, "f2"),
        change: f64_field(row, "f4"),
        change_pct: f64_field(row, "f3"),
        open: f64_field(row, "f17"),
        high: f64_field(row, "f15"),
        low: f64_field(row, "f16"),
        pre_close: f64_field(row, "f18"),
        amount: norm::yuan_to_yi(f64_field(row, "f6")),     // 元 → 亿元
        volume: norm::shou_to_wanshou(f64_field(row, "f5")), // 手 → 万手
        // 个股扩展："-" → None（指数）；数值 → Some（个股）
        turnover_rate: opt_f64_field(row, "f8"),
        pe: opt_f64_field(row, "f9"),
        pb: opt_f64_field(row, "f23"),
        total_mv: opt_f64_field(row, "f20").map(norm::yuan_to_yi), // 元 → 亿元
        circ_mv: opt_f64_field(row, "f21").map(norm::yuan_to_yi),
    }
}
```

### 4.2.5 客户端方法（eastmoney.rs）

两个新方法都**接收完整 secid**，把"指数不走 secid()"的约束压在签名上——调用方传代码也过不了编译期检查不了，但方法名与文档注释双保险。方法本身对指数/个股通用（同一 ulist / trends2 接口），差异只在**入口侧 secid 的来源**（指数查表 / 个股规则函数，§4.10.1），不在数据层：

```rust
impl EastmoneyClient {
    /// 详情页快照（ulist.np，单 secid，指数/个股通用）。
    /// ⚠️ secid 必须是完整形态：指数经 index_secid() 查表获得；
    /// 指数严禁走 secid() 规则函数（000001 会映射成平安银行 0.000001）；
    /// 个股经 secid() 规则函数是安全的（§4.10.1）
    pub async fn quote_detail(&self, secid: &str) -> Result<DetailQuote> {
        let url = format!(
            "{}/ulist.np/get?fltt=2&invt=2&secids={secid}\
             &fields=f2,f3,f4,f5,f6,f8,f9,f12,f14,f15,f16,f17,f18,f20,f21,f23",
            self.base_q,
        );
        let v = self.get_json(&url).await?;
        let data = raw::take_data(&v, "ulist-quote-detail")?;
        let rows = data["diff"].as_array().unwrap_or(&vec![]);
        rows.first()
            .map(raw::parse_quote_detail_row)
            .ok_or_else(|| DataError::Api(format!("quote_detail 空数据: {secid}")))
    }

    /// 当日分时（trends2，ndays 固定 1；指数/个股通用，f51-f58 字段契约一致；
    /// 接口契约见 01 §1.5.4，2026-10-23 实测）
    pub async fn quote_timeline(&self, secid: &str) -> Result<Timeline> {
        let url = format!(
            "{}/stock/trends2/get?secid={secid}&fields1=f1,f2,f3,f7,f8\
             &fields2=f51,f52,f53,f54,f55,f56,f57,f58&iscr=0&ndays=1",
            self.base_his,
        );
        let v = self.get_json(&url).await?;
        let data = raw::take_data(&v, "trends2-quote")?;
        Ok(Timeline {
            code: raw::str_field(data, "code").to_string(),
            pre_close: raw::f64_field(data, "preClose"),
            points: data["trends"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|l| l.as_str())
                .filter_map(raw::parse_timeline_line) // 已存在，eastmoney_types.rs
                .collect(),
        })
    }
}
```

> 复用说明：`TimeLine`/`TimelinePoint` 模型与 `parse_timeline_line` 解析函数已存在于当前代码（models.rs / eastmoney_types.rs），本任务**只补 client 方法**，不动解析层。
>
> **缓存决策：两个方法都不接 TTL 缓存。** 唯一消费方是详情页轮询（spawn_polling 自带 5s/60s 节流与退避），无第二共享场景；01 §1.6.2 的 ulist 5s TTL 是为多面板共享设计，对单消费者是纯开销。trends2 逐分钟追加，TTL 反而制造 5s 数据回退窗口。

---

## 4.3 桥接增量（bridge.rs）

照 `start_polling_index_quotes` 的既有模板抄，泛型 `spawn_polling(fetch, on_data)` 骨架不动：

```rust
// bridge.rs 追加两个方法（注释模板与现有方法一致）

/// 启动轮询：指数详情快照（详情页行情头）
pub fn start_polling_quote_detail(
    &self,
    secid: String,
    tx: async_channel::Sender<stock_data::DetailQuote>,
) -> PollingHandle {
    let client = self.client.clone();
    self.rt.block_on(async move {
        spawn_polling(
            move || {
                let client = client.clone();
                let secid = secid.clone();
                async move { client.quote_detail(&secid).await }
            },
            move |data| {
                let _ = tx.try_send(data);
            },
        )
    })
}

/// 启动轮询：指数当日分时
pub fn start_polling_quote_timeline(
    &self,
    secid: String,
    tx: async_channel::Sender<stock_data::Timeline>,
) -> PollingHandle {
    let client = self.client.clone();
    self.rt.block_on(async move {
        spawn_polling(
            move || {
                let client = client.clone();
                let secid = secid.clone();
                async move { client.quote_timeline(&secid).await }
            },
            move |data| {
                let _ = tx.try_send(data);
            },
        )
    })
}
```

> fetch 闭包每 tick 克隆 Arc/String 是 spawn_polling 泛化签名的既有约定（03 §3.3.2：fetch 必须 `Fn`，捕获物只能 clone）。

---

## 4.4 UI 层增量（stock-app）

### 4.4.1 轮询模板抽取：ui/polling_util.rs（新建）

OverviewPanel 的私有 `spawn_polling_loop` 已在 2026-11 的重构中泛型化，详情页要用同一模板——**抽成跨面板公共函数**，OverviewPanel 同步迁移（删私有版，改 use，调用点不动，泛型自动推断）：

```rust
// ui/polling_util.rs
use std::sync::Arc;

use crate::bridge::DataBridge;
use async_channel::Sender;
use stock_data::PollingHandle;

/// 轮询启动模板：get_fn 发起轮询（tokio 侧生产），update 在 Entity 内回填状态（UI 侧消费）。
/// 原 OverviewPanel 私有方法的跨面板泛型版，行为完全一致。
pub fn spawn_polling_loop<V, T, G, F>(
    bridge: Arc<DataBridge>,
    cx: &mut Context<V>,
    get_fn: G,
    update: F,
) -> PollingHandle
where
    V: 'static,
    T: Send + 'static,
    G: Fn(Arc<DataBridge>, Sender<T>) -> PollingHandle + 'static,
    F: Fn(&mut V, T) + 'static,
{
    let (tx, rx) = async_channel::unbounded::<T>();
    cx.spawn(async move |this, cx| {
        while let Ok(data) = rx.recv().await {
            let _ = this.update(cx, |s, cx| {
                update(s, data);
                cx.notify();
            });
        }
    })
    .detach();
    get_fn(bridge, tx)
}
```

> 若泛型版遇到编译问题（`Context<V>` 约束差异），退路是复制一份面板私有版——模板本身 25 行，可接受的重复；先抽公共，收敛失败再降级。

### 4.4.2 OverviewPanel：卡片可点击（overview_panel.rs）

四处增量：

```rust
// ① use 追加（when_some 需要 FluentBuilder，app.rs 已有同款导入先例）
use gpui_kit::prelude::FluentBuilder as _;
use crate::ui::app::{DetailTarget, StockAppState};   // StockAppState 已有，补 DetailTarget
use stock_data::index_secid;

// ② IndexSnapshot 增加 secid：None = 表外指数 → 卡片不可点
pub struct IndexSnapshot {
    pub code: String,
    pub secid: Option<String>, // 新增：完整 secid（"1.000001"）；查表失败为 None
    pub name: String,
    // ...其余字段不动...
}

// update_snapshots 映射处同步补：
let new_snapshots: Vec<IndexSnapshot> = new_snapshots
    .into_iter()
    .map(|quote| IndexSnapshot {
        secid: index_secid(&quote.code).map(String::from), // 新增
        code: quote.code,
        // ...
    })
    .collect();

// ③ card() 的 hover 样式加指针光标
.hover(|style| style.shadow_md().cursor_pointer())
// 若 cursor_pointer 方法在当前版本不存在，用 style.cursor(CursorStyle::PointingHand)

// ④ render_indicator_cards → render_snapshot_card 传 cx（listener 需要），
//    卡片追加 on_click。只有 secid 命中查表的卡片才挂点击事件
fn render_indicator_cards(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = theme_palette(cx);
    h_flex().gap_3().w_full().justify_between().children(
        self.snapshots
            .iter()
            .enumerate()
            .map(|(ix, snap)| self.render_snapshot_card(ix, snap, cx)), // 签名加 cx
    )
}

fn render_snapshot_card(&self, ix: usize, snap: &IndexSnapshot, cx: &mut Context<Self>) -> impl IntoElement {
    // ...现有着色/字段渲染不动...
    let secid = snap.secid.clone();
    let code = snap.code.clone();
    let name = snap.name.clone();
    self.card(("index-card", ix))
        // when_some：secid 为 None（表外指数）时不挂 on_click，卡片退化为纯展示
        .when_some(secid.clone(), |el, secid| {
            el.on_click(cx.listener(move |this, _, _, cx| {
                this.app_state.update(cx, |s, cx| {
                    s.active_detail = Some(DetailTarget::Index {
                        secid: secid.clone(),
                        code: code.clone(),
                        name: name.clone(),
                    });
                    cx.notify();
                });
            }))
        })
        // ...child(...) 系列不动...
}
```

> 卡片已是 `Stateful<Div>`（`.id()` + `.hover()`，group_hover 陷阱的修复产物），`on_click` 直接可用，无需再碰元素状态问题。

### 4.4.3 theme.rs：均价线语义色

```rust
// theme.rs 追加（仿照 UP_COLOR 的定义方式，02 §2.2 先例）
/// 均价线 —— 橙黄，与红涨/绿跌语义色区分
pub const AVG_PRICE_COLOR: Rgba = rgb(0xE6A23C);
```

### 4.4.4 DetailPanel（ui/detail_panel.rs，新建）

```rust
use std::sync::Arc;

use crate::{
    bridge::DataBridge,
    ui::{
        app::{DetailTarget, StockAppState},
        polling_util::spawn_polling_loop,
        theme::{theme_palette, AVG_PRICE_COLOR},
    },
};
use gpui_kit::{
    base::{h_flex, v_flex},
    component::chart::LineChart,
    *,
};
use stock_data::{DetailQuote, PollingHandle, Timeline};

/// 分时图固定槽位：09:30-11:30 共 121 点 + 13:00-15:00 共 121 点
const TIMELINE_SLOTS: usize = 242;

pub struct DetailPanel {
    target: DetailTarget,
    app_state: Entity<StockAppState>,
    /// 持有 bridge：保住 tokio runtime 生命周期（同 OverviewPanel._bridge）
    _bridge: Arc<DataBridge>,
    polling_handles: Vec<PollingHandle>,
    detail: Option<DetailQuote>,
    timeline: Option<Timeline>,
}

impl DetailPanel {
    pub fn new(
        bridge: Arc<DataBridge>,
        target: DetailTarget,
        app_state: Entity<StockAppState>,
        cx: &mut Context<Self>,
    ) -> Self {
        // 访问器替代解构：Index/Stock 变体通用（解构会把面板绑死在指数变体上，
        // 加 Stock 变体后 match 不穷尽直接编译失败）
        let secid = target.secid().to_string();

        // 直接接轮询，不写"先一次性请求再开轮询"的双通道：
        // spawn_polling 循环是先 fetch 后 sleep，启动即有首帧（polling.rs 既有行为）
        let handle_detail = spawn_polling_loop(
            bridge.clone(),
            cx,
            move |bridge, tx| bridge.start_polling_quote_detail(secid.clone(), tx),
            |s: &mut Self, data| s.detail = Some(data),
        );
        let handle_timeline = spawn_polling_loop(
            bridge.clone(),
            cx,
            move |bridge, tx| bridge.start_polling_quote_timeline(secid.clone(), tx),
            |s: &mut Self, data| s.timeline = Some(data),
        );

        Self {
            target,
            app_state,
            _bridge: bridge,
            polling_handles: vec![handle_detail, handle_timeline],
            detail: None,
            timeline: None,
        }
    }

    pub fn target(&self) -> &DetailTarget {
        &self.target
    }

    /// 关闭/切换详情页前由 StockApp 调用（handle_detail_change → close_detail）。
    /// PollingHandle drop 不停任务，此方法就是防僵尸轮询的唯一闸门
    pub fn stop_polling(&self) {
        for h in &self.polling_handles {
            h.stop();
        }
    }

    fn go_back(&mut self, cx: &mut Context<Self>) {
        // 单向数据流：只写 app_state，StockApp 的 observe 负责实体释放与轮询停止
        self.app_state.update(cx, |s, cx| {
            s.active_detail = None;
            cx.notify();
        });
    }
}
```

### 4.4.5 详情页渲染骨架

```rust
impl Render for DetailPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_full()
            .v_flex()
            .gap_3()
            .p_2()
            .child(self.render_header(cx))
            .child(self.render_quote_grid(cx))
            .child(self.render_timeline_chart(cx))
    }
}

/// 头部：返回按钮 + 名称/代码 + 价格大字 + 涨跌额/幅（相对昨收着色）
fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = theme_palette(cx);
    let (code, name) = (self.target.code().to_string(), self.target.name().to_string());
    let d = self.detail.as_ref();
    let color = d.map_or(theme.text_secondary, |x| {
        if x.price >= x.pre_close { theme.color_rise } else { theme.color_fall }
    });

    h_flex().items_center().gap_3()
        .child(
            // 自绘返回按钮：与总览卡片同款 stateful div + on_click 模式
            div().id("detail-back").px_2().py_1().rounded_sm()
                .hover(|s| s.cursor_pointer().bg(theme.border_primary))
                .child("← 返回")
                .on_click(cx.listener(|this, _, _, cx| this.go_back(cx))),
        )
        .child(div().text_lg().font_bold().child(name))
        .child(div().text_sm().text_color(theme.text_tertiary).child(code))
        .child(div().text_2xl().text_color(color)
            .child(d.map_or("--".into(), |x| format!("{:.2}", x.price))))
        .child(div().text_sm().text_color(color)
            .child(d.map_or("--".into(), |x| {
                format!("{:+.2}  {:+.2}%", x.change, x.change_pct)
            })))
}

/// 指标行：今开 / 最高 / 最低 / 昨收 / 成交额(亿) / 成交量(万手) 六格
/// 每格 v_flex：label（text_xs，text_tertiary）+ value（text_sm，相对昨收着色）
/// 个股详情在本行之下追加一行：换手率 / PE / PB / 总市值 / 流通市值 ——
/// 用 when_some 逐格挂载，Option 为 None 的格子不渲染；指数扩展字段全为 None，
/// 这行自然消失，无需 match DetailTarget 判断类型（§4.10.2 第 3 步）
fn render_quote_grid(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = theme_palette(cx);
    let d = self.detail.as_ref();
    // 每项颜色按其自身相对昨收：>= 昨收红、< 昨收绿；d 为 None 时一律 "--"
    // ⚠️ 显示文案直接用已归一字段：amount 亿元、volume 万手，禁止 UI 再换算
    // 结构：h_flex().justify_between() + 六个 v_flex 格子（实现略，照头部着色规则）
    // ...（照 4.0.1 原型实现，约 60 行）
    // 个股扩展行（指数为 None 时整行不渲染，when_some 逐格挂载）：
    //   换手率 {:.2}% / PE {:.1} / PB {:.2} / 总市值 {:.0}亿 / 流通市值 {:.0}亿
    # unimplemented_placeholder()
}

/// 分时图：双 LineChart 绝对定位叠层（均价层在下不抢交互，价格层在上带轴与 tooltip）
fn render_timeline_chart(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = theme_palette(cx);
    let Some(t) = self.timeline.as_ref()
        .filter(|t| !t.points.is_empty() && t.pre_close > 0.0)
    else {
        // 空态：首帧到达前（数百 ms）/ 接口异常退避期 / pre_close 异常
        return div().flex_1().min_h(px(320.))
            .items_center().justify_center()
            .text_color(theme.text_tertiary)
            .child("暂无分时数据")
            .into_any_element();
    };

    // 价格线颜色随现价相对昨收（东财惯例）
    let last = t.points.last().map_or(t.pre_close, |p| p.price);
    let price_color = if last >= t.pre_close { theme.color_rise } else { theme.color_fall };
    let (lo, hi) = y_domain(t);
    let pc = t.pre_close;
    let (rise, fall) = (theme.color_rise, theme.color_fall);

    // 均价层：无轴/无网格/无交互（不抢 hover），纯线。
    // ⚠️ 与价格层必须同 y_domain、同 y_axis + 同 y_tick_format——
    // 两层各自测量的刻度 gutter 等宽，绘图区左缘对齐，两条线的 x 才对得上
    let avg = LineChart::new(t.points.iter().cloned())
        .id("tl-avg")
        .x(|p| p.time.clone())          // X = "HH:MM"，ScalePoint 按槽位布局
        .y(|p| p.avg_price)
        .point_count(TIMELINE_SLOTS)    // 固定 242 槽位：盘中数据从首槽连续填充，右端留白不拉伸
        .linear()                       // 分时用折线，不开平滑曲线
        .y_domain(lo, hi)
        .y_axis(true)
        .y_tick_format(|v| format!("{v:.2}")) // 与价格层同刻度文案 = 同 gutter 宽度
        .stroke(AVG_PRICE_COLOR)
        .interactive(false)
        .x_axis(false)
        .grid(false);

    let price = LineChart::new(t.points.iter().cloned())
        .id("tl-price")
        .x(|p| p.time.clone())
        .y(|p| p.price)
        .point_count(TIMELINE_SLOTS)
        .linear()
        .y_domain(lo, hi)
        .y_axis(true)
        .y_tick_format(|v| format!("{v:.2}"))
        .stroke(price_color)
        .reference_line(pc)             // 昨收虚线（组件内置样式，域外自动不画）
        .x_tick_count(3)                // 标签铺满 242 槽位取 3 档：09:30 / 11:30|13:00 / 15:00；
                                        // 未到的槽位标签不画（组件 point_count 语义）
        .name("价格")
        .tooltip_value(|_, v| format!("{v:.2}"))
        .tooltip_value_color(move |_, v| if v >= pc { rise } else { fall });

    div().flex_1().min_h(px(320.)).relative()
        .child(div().absolute().inset_0().child(avg))
        .child(div().absolute().inset_0().child(price))
        .into_any_element()
}

/// y 域对称于昨收（东财分时惯例）：上下振幅取两侧较大者，再留 5% 余量。
/// 最小振幅兜底 0.05% 防止横盘日 lo==hi（y_domain 相等时 LineChart 整图不画）
fn y_domain(t: &Timeline) -> (f64, f64) {
    let pc = t.pre_close;
    let (mut lo, mut hi) = (pc, pc);
    for p in &t.points {
        lo = lo.min(p.price).min(p.avg_price);
        hi = hi.max(p.price).max(p.avg_price);
    }
    let spread = (hi - pc).max(pc - lo).max(pc * 0.0005) * 1.05;
    (pc - spread, pc + spread)
}
```

> `y_domain` 是纯函数，放面板底部 `#[cfg(test)]` 里直测（横盘/涨停/跌停三组数据），不建窗口。

---

## 4.5 导航链路数据流总图

```
OverviewPanel（总览）                StockAppState（共享 Entity）
  卡片 on_click ──── 写请求 ────────▶  active_detail: Option<DetailTarget>
       ▲                                    │ observe
       │                              StockApp::handle_detail_change
  render_main_content                       │
  （detail 优先渲染）        ┌── Some(新 target) ─▶ close 旧 + cx.new(DetailPanel)
                             └── None ───────────▶ close_detail(): stop_polling + 释放
                                          │
                    DetailPanel ◀────┘
                      │ spawn_polling_loop ×2
                      ▼
             bridge.start_polling_quote_detail / quote_timeline
                      │ spawn_polling（tokio，盘中 5s/收盘 60s/退避）
                      ▼
             EastmoneyClient.quote_detail / quote_timeline
                      │ async_channel（tokio → gpui）
                      ▼
             DetailPanel 状态回填 + cx.notify → 重渲染
```

---

## 4.6 gpui-kit 0.8.0 分时图 API 核对表（2026-11-19 对照 registry 源码）

| API                                                       | 行为（gpui-component 0.8.0 `src/chart/line_chart.rs`）                                                                                           |
| --------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `.point_count(n)`                                         | x 轴按 n 个等距槽位布局，数据从首槽**连续**填充，未到的槽位留空——源码文档原话即 _"as an intraday chart does before the close"_，为本功能量身定做 |
| `.reference_line(v)`                                      | 在 y=v 画横跨绘图区的虚线——原话即 _"such as a previous close"_                                                                                   |
| `.y_domain(min, max)`                                     | 钉死 y 轴（默认自动缩放会把分时压扁）；min==max 时整图不画                                                                                       |
| `.y_axis(true)` + `.y_tick_format(f)`                     | 左侧 gutter 显示刻度（默认 5 档，`AxisLabelPlacement::Outside` 自动测宽）                                                                        |
| `.x_tick_count(n)`                                        | 标签均匀铺满**全部槽位**；落在数据之外（未交易时段）的槽不画                                                                                     |
| `.linear()`                                               | 折线（默认 curve 之外另有 natural/step_after）                                                                                                   |
| `.interactive(false)`                                     | 去掉 hitbox：不响应 hover、不出 tooltip——叠层底层专用                                                                                            |
| `.name()` / `.tooltip_value()` / `.tooltip_value_color()` | hover 十字线 tooltip 的行名/文案/着色                                                                                                            |
| `.id(...)`                                                | 默认取构造点 caller_id，显式命名防两图撞 id（同层兄弟节点）                                                                                      |
| `.appear_key(k)`                                          | **不要用**：轮询每次更新数据都会重放入场动画，图会每 5s 闪一次；不设 key 时数据自动原地重绘                                                      |

导入路径：`use gpui_kit::component::chart::LineChart;`（gpui-kit 0.8.0 将 gpui-component 转发为 `gpui_kit::component`）。`X` 泛型约束 `PartialEq + Into<SharedString>`，`String` 满足；`Y` 需实现 `PlotValue`，`f64` 满足。

---

## 4.7 测试方案

### 4.7.1 stock-data 单测（普通 `#[test]`）

| 对象                     | 断言要点                                                                                                    |
| ------------------------ | ----------------------------------------------------------------------------------------------------------- |
| `index_secid`            | 000001→1.000001、399001→0.399001、个股代码→None（§4.2.1 已给代码）                                          |
| `shou_to_wanshou`        | 4.8e8 手 → 4.8e4 万手、0 值                                                                                 |
| `parse_quote_detail_row` | fixture 行 → 字段映射、amount 元→亿、volume 手→万手；个股 fixture 扩展字段 Some、指数 fixture 扩展字段 None |
| `opt_f64_field`          | 数字→Some、`"-"`→None                                                                                       |
| `y_domain`（stock-app）  | 横盘（spread 兜底）、涨停单边、昨收居中对称                                                                 |

### 4.7.2 wiremock 集成测试（`stock-data/tests/http_tests.rs` 追加）

fixture 新增 `tests/fixtures/eastmoney/quote_detail.json`（指数 ulist 单行，f6/f5 用真实量级）与 `quote_detail_stock.json`（个股单行，含 f8/f9/f20/f21/f23 扩展字段），trends2 复用 §1.7.2 规划的 `trends.json`：

```rust
#[tokio::test]
async fn quote_detail_parses_units() {
    let server = MockServer::start().await;
    let fixture = std::fs::read_to_string("tests/fixtures/eastmoney/quote_detail.json").unwrap();
    Mock::given(method("GET"))
        .and(path("/api/qt/ulist.np/get"))
        .and(query_param("secids", "1.000001"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
        .mount(&server)
        .await;
    let client = EastmoneyClient::with_base_url(
        format!("{}/api/qt", server.uri()),  // base_q
        "http://unused".into(),              // base_his
        "http://unused".into(),              // base_dc
    ).unwrap();
    let d = client.quote_detail("1.000001").await.unwrap();
    assert_eq!(d.code, "000001");
    assert!(d.amount > 1000.0 && d.volume > 10000.0); // 亿元/万手量级
}

#[tokio::test]
async fn quote_detail_parses_stock_fields() {
    // 同款模板：query_param("secids", "0.600519") + fixture quote_detail_stock.json
    //（f8/f9/f23/f20/f21 用真实量级）
    // 断言：turnover_rate/pe/pb/total_mv/circ_mv 全为 Some，且 total_mv 单位为亿元
    //（f20 原始值元 → 亿元，量级差 8 个数量级）
}

#[tokio::test]
async fn quote_timeline_parses_pre_close_and_points() {
    // 同款模板：path("/api/qt/stock/trends2/get") + query_param("secid", "1.000001")，
    // with_base_url 第二参指向 mock；断言 pre_close 与 points 非空
}
```

### 4.7.3 真实接口 smoke test（`#[ignore]`，手动跑）

```rust
#[tokio::test]
#[ignore = "需要真实网络"]
async fn smoke_quote_detail_and_timeline() {
    let client = EastmoneyClient::new().unwrap();
    let d = client.quote_detail("1.000001").await.unwrap();
    assert!(d.price > 0.0 && d.pre_close > 0.0 && d.amount > 0.0);
    let t = client.quote_timeline("1.000001").await.unwrap();
    assert!(t.pre_close > 0.0 && t.points.len() > 100);
    assert_eq!(t.points[0].time, "09:30"); // 首点即开盘分钟

    // 个股：同链路 + 扩展字段
    let s = client.quote_detail("0.600519").await.unwrap(); // 贵州茅台
    assert!(s.turnover_rate.is_some() && s.total_mv.is_some());
    let _ = client.quote_timeline("0.600519").await.unwrap();
}
```

### 4.7.4 手动验收清单（对应 DEV_PLAN T4.6 验收）

- [ ] 盘中：总览点击上证指数卡片 → 详情页 1s 内出现行情头与分时图（数据与东财网页对账：价格/涨跌幅/成交额）
- [ ] 分时图 5s 推进：右端加点，图形不拉伸（固定 242 槽位生效，图形比例恒定）
- [ ] 均价线与价格线 x 对齐（同一时刻竖直对应，无整体错位）
- [ ] 昨收虚线位置与东财网页一致；y 域上下对称于昨收
- [ ] hover 图表出十字线 + tooltip，tooltip 值红绿着色
- [ ] "← 返回" → 回总览；**返回后观察日志/RUST_LOG=info 确认 quote_detail 与 trends2 请求停止**（轮询已 stop）
- [ ] 详情页打开时点侧边栏"榜单" → 详情页关闭、轮询停止、榜单正常渲染
- [ ] 详情页打开时点击"深证成指"卡片 → 面板切换为深成详情，旧轮询停止（不叠加）
- [ ] 收盘时段打开详情页：分时图完整（~242 点），状态轮询 60s 降频
- [ ] 断网 30s 不 panic，详情页保持旧数据；恢复后自动续刷
- [ ] `cargo test -p stock-data -p stock-app` 全绿

---

## 4.8 踩坑清单（实现前通读）

1. **指数 secid 严禁走 `secid()`，个股入口不受此限**：个股规则会把 000001 映射成平安银行 `0.000001`，指数入口唯一来源是 `index_secid()` 查表；个股入口走 `secid()` 规则函数是安全的（缺陷只在指数代码撞车），数据源自带 f13+f12 时优先拼接（§4.2.1 / §4.10.1）。全链路禁止任何环节从 code 反推市场。
2. **PollingHandle drop 不停任务**：详情页关闭/替换前必须显式 `stop_polling()`（§4.1.4 close_detail）。漏掉的后果：返回总览后每 5s 仍打 quote_detail + trends2；把 6 张卡各点一遍 = 12 个僵尸轮询任务，限速桶被拖慢，总览刷新跟着变卡。
3. **双图表叠层必须同 y_domain + 同 y_axis + 同 y_tick_format**：两层各自测量刻度 gutter，文案不同 → gutter 宽度不同 → 绘图区左缘错位 → 两条线 x 对不上。这是叠层方案唯一的对齐契约。
4. **y_domain 相等时 LineChart 整图不画**：横盘日/异常数据（pre_close=0）必须走 `y_domain()` 的最小振幅兜底 + 空态占位，否则详情页白屏一块。
5. **`point_count` 要求数据从首槽连续**：trends2 ndays=1 天然连续可直接用；将来做 5 日分时（ndays>1）跨日不连续，不能直接套 point_count，需另做槽位映射（已列入不做范围）。
6. **不要设 `appear_key`**：轮询每 5s 更新数据会重放入场动画；不设 key 时数据原地重绘。
7. **`render_snapshot_card` 需要加 `cx` 参数**：现有签名 `(ix, snap)` 没有 cx，挂 listener 会编译失败；`render_indicator_cards` 调用点同步改。
8. **单位契约**：DetailQuote 出口即亿元/万手，UI 直接 format，禁止再换算（03 §3.2 纪律；01 §1.5.6 坑 3 的 `{:.2}` 截断字符串前科仍在）。
9. **直接接轮询，不写双通道**：spawn_polling 先 fetch 后 sleep，启动即有首帧；"一次性请求预热 + 轮询"是两套代码加一段回填竞态，纯负资产。
10. **详情页关闭后回到当前菜单面板**（现状默认 ranklist），渲染分流不要写死"返回=总览"。
11. **DetailTarget 用访问器不用解构**：面板/桥接层若写 `let DetailTarget::Index { .. } = ...` 解构，加 Stock 变体后 match 不穷尽直接编译失败，且把面板绑死在指数变体上——统一走 `target.secid()/code()/name()`（§4.1.2）。
12. **榜单入口的领涨股没有 code**：`StockBoard` 现只存领涨股名字符串（leading_stock），要做"点击领涨股进详情页"必须先在 clist 解析层补 code/market 字段（§4.10.1），名字字符串拼不出 secid。
13. **菜单面板轮询全程活跃是既定行为，不是漏了清理**：OverviewPanel / RanklistPanel 的 Entity 在 `StockApp::new` 一次创建后常驻（`menus` 数组持有），切菜单 / 打开详情页只换渲染分支，面板轮询照跑——隐藏面板数据持续更新，切回无首帧等待。全应用峰值 8 个轮询（总览 4 + 榜单 2 + 详情页 2），盘中 5s 一轮 ≈ 1.6 req/s，贴着 500ms 全局限速桶（2 req/s）的 80%，仍在容量内。MVP 不做菜单切换的 stop/resume（对称复杂度翻倍，收益只是省后台请求）；若将来面板数量增长使限速桶吃紧，再统一实现 visibility-driven polling（StockApp 按 active_key/active_detail 显式停启隐藏面板），不要让每个面板自己管理启停状态。

---

## 4.9 实现顺序（每步可验证）

| 步  | 内容                                                                                                                                        | 验证                                                        |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| 1   | stock-data：normalize → opt_f64_field → DetailQuote 模型 → parse_quote_detail_row → index_secid → quote_detail/quote_timeline → lib.rs 出口 | `cargo test -p stock-data`（含 wiremock + smoke --ignored） |
| 2   | bridge.rs 两个 start_polling                                                                                                                | `cargo check -p stock-app`                                  |
| 3   | polling_util.rs 抽取 + OverviewPanel 迁移（删私有版）                                                                                       | `cargo check`；跑应用确认总览轮询行为不变                   |
| 4   | app.rs 导航（DetailTarget 双变体 + 访问器 / active_detail / observe / 渲染分流 / 侧边栏清理）+ OverviewPanel 卡片点击 + cursor_pointer      | 跑应用：点击卡片出现空白详情页占位（面板可先返回空 div）    |
| 5   | DetailPanel 骨架（无图表）：头部 + 指标行                                                                                                   | 跑应用：行情头数据与东财对账                                |
| 6   | 分时图（双 LineChart 叠层 + y_domain + 空态）                                                                                               | 跑应用：图形/对齐/昨收线/tooltip；盘中观察 5s 推进          |
| 7   | 测试补齐 + §4.7.4 全量手动验收                                                                                                              | `cargo test -p stock-data -p stock-app` 全绿                |

> 实现完成后回写：本文 §4.6 核对表若与实际版本 API 有出入，按实际为准修订；DEV_PLAN 进度表 T4.6 状态同步更新。

---

## 4.10 个股详情接入路径（本期不实现，通道已留）

> 通用骨架在 MVP 各步已就位：`DetailTarget::Stock` 变体（§4.1.2）、`DetailQuote` 个股扩展字段（§4.2.2）、`quote_detail`/`quote_timeline` 本身就是按 secid 通用的（§4.2.5）。个股接入 = **加一个入口 + 显示扩展行**，预计半天。

### 4.10.1 入口场景与 secid 来源

| 入口       | 触发点                           | secid 来源                                | 前置数据工作                                                                                                             |
| ---------- | -------------------------------- | ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| 榜单领涨股 | RanklistPanel "领涨股"单元格点击 | `f13.f12` 拼接（最稳）                    | ⚠️ `StockBoard` 现只存领涨股名字符串，需在 clist 解析层补 `leading_stock_code`/`leading_stock_market` 字段（§4.8 坑 12） |
| 自选列表行 | WatchlistPanel（未实现，T2.3）   | 行点击时 `secid()` 规则函数（对个股可靠） | watchlist.json 建议直接存 secid，免规则推导                                                                              |
| 搜索结果   | suggest 搜索（T4.5）             | suggest 返回自带 market/code → 拼接       | 无                                                                                                                       |

### 4.10.2 接入步骤

1. 入口侧构造 `DetailTarget::Stock { secid, code, name }` 写 `app_state.active_detail`——照抄 §4.4.2 ④ 的 on_click 模式，只换枚举变体
2. **无需改动**：DetailPanel / bridge 两个 start_polling / client 两方法 / 导航 observe——全部按 secid 工作（访问器保证面板不关心变体）
3. 行情头个股扩展行自动出现（DetailQuote 扩展字段有值 → when_some 渲染，§4.4.5 render_quote_grid 注释）
4. 个股专属功能（日 K CandlestickChart / 资金流 / F10）在 match `DetailTarget::Stock` 分派下加 Tab 或区块——枚举变体的意义就在这里

### 4.10.3 注意

- **北交所**：`secid()` 现规则拒绝 8/4/92 开头（T1.3 设计如此），个股入口 MVP 限沪深；要支持需扩展规则函数，且北交所个股存在 f2 返回特殊标记（`s:2048`）的形态，实现时先 curl 验证容错
- **trends2 个股契约**：f51-f58 字段与指数完全一致（同一接口），若实测有出入以 curl 为准回写本文
- **个股停牌**：quote_detail 的 volume=0 / 分时 points 为空 → 详情页复用现有空态占位，无需新逻辑
