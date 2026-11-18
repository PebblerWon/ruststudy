use serde::{Deserialize, Serialize};

/// 个股实时快照（出口已归一：市值亿元、成交额万元、成交量手）

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Quote {
    pub code: String, // 6 位代码
    pub name: String,
    pub market: u8, // 0=深 1=沪
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,    // %
    pub volume: f64,        // 手
    pub amount: f64,        // 万元
    pub turnover_rate: f64, // %
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub pre_close: f64,
    pub total_mv: f64,      // 亿元
    pub circ_mv: f64,       // 亿元
    pub is_suspended: bool, // 是否停牌
}

/// 指数快照
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IndexQuote {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub amount: f64, // 亿元
}

/// K 线（与 quant 的 Kline 一致，可直接拷贝 models.rs）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Kline {
    pub open_time: i64, // Unix ms
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64, // 股（东财手 × 100）
    pub close_time: i64,
    pub quote_volume: f64, // 元
    pub trades_count: u32, // 东财无此字段，恒为 0
    pub is_closed: bool,
}

/// 分时点
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TimelinePoint {
    pub time: String, // "HH:MM"
    pub price: f64,
    pub avg_price: f64,
    pub volume: f64, // 手
    pub amount: f64, // 万元
}

/// 当日分时（含昨收基准，供涨跌幅着色）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeLine {
    pub code: String,
    pub pre_close: f64,
    pub points: Vec<TimelinePoint>,
}

/// 全市场统计（总览页）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MarketSnapshot {
    pub total: usize,
    pub up_count: usize,
    pub down_count: usize,
    pub flat_count: usize,
    pub limit_up: usize, // 涨跌幅 ≥ 9.9%（近似）
    pub limit_down: usize,
    pub total_amount: f64, // 亿元
}

/// K 线周期（东财无 4h，刻意不设 H4）
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum Interval {
    M1,
    M5,
    M15,
    M30,
    H1,
    D1,
    W1,
}

impl Interval {
    pub fn klt(self) -> u32 {
        match self {
            Self::M1 => 1,
            Self::M5 => 5,
            Self::M15 => 15,
            Self::M30 => 30,
            Self::H1 => 60,
            Self::D1 => 101,
            Self::W1 => 102,
        }
    }

    /// 分钟粒度时长；日线/周线返回 0（close_time 特殊处理）
    pub fn minutes(self) -> i64 {
        match self {
            Interval::M1 => 1,
            Interval::M5 => 5,
            Interval::M15 => 15,
            Interval::M30 => 30,
            Interval::H1 => 60,
            _ => 0,
        }
    }
}

/// 复权方式（东财 fqt 参数）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Adjust {
    None = 0,     // 不复权
    Forward = 1,  // 前复权（指标与展示默认用这个）
    Backward = 2, // 后复权
}

impl Adjust {
    pub fn fqt(self) -> u32 {
        self as u32
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MarketFundFlow {
    pub date: String,
    pub main_net_inflow: f64,
    pub small_net_inflow: f64,
    pub medium_net_inflow: f64,
    pub large_net_inflow: f64,
    pub super_large_net_inflow: f64,
    pub main_net_inflow_percent: f64,
    pub small_net_inflow_percent: f64,
    pub medium_net_inflow_percent: f64,
    pub large_net_inflow_percent: f64,
    pub super_large_net_inflow_percent: f64,
    pub sh_close: f64,
    pub sh_change_percent: f64,
    pub sz_close: f64,
    pub sz_change_percent: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutualType {
    Sg,   // 001 沪股通（北向）
    HkSg, // 002 港股通（沪）（南向）
    Sz,   // 003 深股通（北向）
    HkSz, // 004 港股通（深）（南向）
}

impl MutualType {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "001" => Some(Self::Sg),
            "002" => Some(Self::HkSg),
            "003" => Some(Self::Sz),
            "004" => Some(Self::HkSz),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Sg => "001",
            Self::HkSg => "002",
            Self::Sz => "003",
            Self::HkSz => "004",
        }
    }

    /// 是否北向（外资经沪深港通买 A 股）
    pub fn is_north(self) -> bool {
        matches!(self, Self::Sg | Self::Sz)
    }
}

/// 通道当日概览（RPT_MUTUAL_QUOTA，盘中实时）
/// ⚠️ 北向行金额字段（net_buy_amt/remain_amt）为 0.0 占位（2024-08 停披露），
///    仅涨跌家数与关联指数实时有效；南向行金额照常
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NorthboundSummary {
    pub channel: MutualType,
    pub name: String,       // "沪股通"（响应带前导空格，解析已 trim）
    pub direction: String,  // "北向" / "南向"
    pub board: String,      // "沪港通" / "深港通"
    pub up_count: u32,      // f104 标的池上涨家数（通道池口径，非全市场）
    pub down_count: u32,    // f105
    pub flat_count: u32,    // f106
    pub net_buy_amt: f64,   // 万元；北向恒 0.0 占位，南向有效
    pub remain_amt: f64,    // 万元；额度剩余，"额度充足"时为 0
    pub index_code: String, // 000001 / 399001 / HSI
    pub index_name: String,
    pub index_change_pct: f64,         // %，实时（INDEX_f3）
    pub trade_date: chrono::NaiveDate, // QUOTA 盘中即返回当天
}
/// 通道历史日线（RPT_MUTUAL_DEAL_HISTORY，T+1 出数）
/// ⚠️ 2024-08-19 起净买额类字段 null → 解析为 None；禁用 0.0 兜底
///    （null 是"停披露"的契约语义，与"真实 0"必须可区分）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NorthboundDaily {
    pub channel: MutualType,
    pub date: chrono::NaiveDate,
    pub deal_amt: f64,                // 万元，成交总额（新旧口径唯一全存金额）
    pub deal_num: u64,                // 成交笔数
    pub net_deal_amt: Option<f64>,    // 万元，净买额（2024-08-19 起 None）
    pub fund_inflow: Option<f64>,     // 万元，当日资金流入
    pub buy_amt: Option<f64>,         // 万元，买入成交额
    pub sell_amt: Option<f64>,        // 万元，卖出成交额
    pub quota_balance: Option<f64>,   // 万元，当日额度余额
    pub hold_market_cap: Option<f64>, // ⚠️ 元（唯一"元"单位），持股市值
    pub lead_stock_code: String,      // 北向 "603366.SH" / 南向 "02335.HK"
    pub lead_stock_name: String,
    pub lead_stock_pct: f64,   // %，领涨股涨跌幅
    pub index_close: f64,      // 关联指数收盘（沪：上证，深：深成，南向：恒指）
    pub index_change_pct: f64, // %
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StockBoard {
    pub name: String,
    pub code: String,
    pub price: f64,
    pub change_percent: f64,
    pub change: f64,
    pub total_market_cap: f64,
    pub rise_count: u32,
    pub fall_count: u32,
    pub leading_stock: String,
    pub leading_stock_change_percent: f64,
}
