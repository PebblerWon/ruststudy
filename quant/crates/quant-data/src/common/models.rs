use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    /// 开盘时间
    pub open_time: i64,

    /// 开盘价
    pub open: f64,

    /// 最高价
    pub high: f64,

    /// 最低价
    pub low: f64,

    /// 收盘时间
    pub close_time: i64,

    /// 收盘价
    pub close: f64,

    /// 成交量
    pub volume: f64,

    /// 成交额
    pub quote_volume: f64,

    /// 成交笔数
    pub trades_count: u32,

    /// 是否收盘
    pub is_closed: bool,
}

impl Kline {
    pub fn open_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self.open_time).expect("invalid open_time timestamp")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Interval {
    M1,  // 1 分钟
    M5,  // 5 分钟
    M15, // 15分钟
    M30, // 30 分钟
    H1,  // 1 小时
    H4,  // 4 小时
    D1,  // 1 天
    W1,  // 1 周
}

impl Interval {
    pub fn as_str(&self) -> &'static str {
        match self {
            Interval::M1 => "1m",
            Interval::M5 => "5m",
            Interval::M15 => "15m",
            Interval::M30 => "30m",
            Interval::H1 => "1h",
            Interval::H4 => "4h",
            Interval::D1 => "1d",
            Interval::W1 => "1w",
        }
    }

    pub fn as_millis(&self) -> i64 {
        match self {
            Interval::M1 => 60_000,
            Interval::M5 => 300_000,
            Interval::M15 => 900_000,
            Interval::M30 => 1_800_000,
            Interval::H1 => 3_600_000,
            Interval::H4 => 14_400_000,
            Interval::D1 => 86_400_000,
            Interval::W1 => 604_800_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    /// 交易对名称，如 "BTCUSDT"
    pub symbol: String,

    /// 交易对状态（"TRADING" 表示正常交易）
    pub status: String,

    /// 基础资产精度（如 BTC 的数量小数位数）
    pub base_asset_precision: u8,

    /// 报价资产精度（如 USDT 的价格小数位数）
    pub quote_asset_precision: u8,
}
