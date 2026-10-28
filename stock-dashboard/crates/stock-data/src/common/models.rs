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
    pub total_mv: f64, // 亿元
    pub circ_mv: f64,  // 亿元
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
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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
