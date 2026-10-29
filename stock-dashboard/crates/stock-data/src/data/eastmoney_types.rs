use crate::common::error::{DataError, Result as DataErrorResult};
use crate::common::models::*;
use crate::data::normalize as norm;
use chrono::{FixedOffset, NaiveDate, NaiveDateTime, TimeZone};
use serde::Deserialize;
use serde_json::Value;

const SHANGHAI: FixedOffset = FixedOffset::east_opt(8 * 3600).unwrap();

/// 通用响应壳：{"rc":0,"data":{...}}，data 可能为 null

pub fn take_data<'a>(v: &'a Value, api: &str) -> DataErrorResult<&'a Value> {
    v.get("data")
        .filter(|d| !d.is_null())
        .ok_or_else(|| DataError::Api(format!("{api}: data 为空")))
}

/// 提取数值字段：Number 直接取；String 尝试 parse（"-" 等失败归 0）；null 归 0
pub fn f64_field(v: &Value, key: &str) -> f64 {
    match v.get(key) {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// 是否为 "-"（停牌占位），用于打 is_suspended 标
pub fn is_dash(v: &Value, key: &str) -> bool {
    match v.get(key) {
        Some(Value::String(s)) => s == "-",
        _ => false,
    }
}

pub fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

pub fn int_field(v: &Value, key: &str) -> i64 {
    f64_field(v, key) as i64
}

fn f64_or_zero<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Value::deserialize(d)? {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    })
}

/// {"rc":0,"data":{"total":3488,"diff":[{"f2":3.03,"f3":10.18,"f5":505496,"f6":152967019.1,"f8":6.77, "f12":"002535","f13":0,"f14":"林州重机","f15":3.03,"f16":3.0,"f17":3.01,"f18":2.75,"f20":2429099714,"f21":2261402731}]}}

pub fn parse_quote_row(row: &Value) -> Quote {
    let suspended = is_dash(row, "f2");

    Quote {
        code: str_field(row, "f12").to_string(),
        name: str_field(row, "f14").to_string(),
        market: int_field(row, "f13") as u8,
        price: f64_field(row, "f2"),
        change: f64_field(row, "f4"),
        change_pct: f64_field(row, "f3"),
        volume: f64_field(row, "f5"),                    // 手
        amount: norm::yuan_to_wan(f64_field(row, "f6")), // 元 → 万元
        turnover_rate: f64_field(row, "f8"),
        open: f64_field(row, "f17"),
        high: f64_field(row, "f15"),
        low: f64_field(row, "f16"),
        pre_close: f64_field(row, "f18"),
        total_mv: norm::yuan_to_yi(f64_field(row, "f20")),
        circ_mv: norm::yuan_to_yi(f64_field(row, "f21")),
        is_suspended: suspended,
    }
}

/// 解析一行 kline CSV：
/// "2024-01-02,1685.00,1685.90,1698.00,1670.11,30726,5172858000.00,1.72,-0.33,-5.62,0.24"
/// 顺序（fields2）：f51 时间, f52 开, f53 收, f54 高, f55 低, f56 量(手), f57 额(元),
///                 f58 振幅, f59 涨跌幅, f60 涨跌额, f61 换手率
/// ⚠️ 收盘价在开盘价之后、最高价之前——不是 开/高/收/低！
pub fn parse_kline_line(line: &str, interval: Interval) -> DataErrorResult<Kline> {
    let p: Vec<&str> = line.split(',').collect();
    if p.len() < 7 {
        return Err(DataError::Api(format!("kline 字段数不足: {line}")));
    }
    // 时间解析：日线 "YYYY-MM-DD"（补 09:30 开盘），分钟线 "YYYY-MM-DD HH:MM"
    let (open_dt, close_dt) = if interval.minutes() > 0 {
        let t = NaiveDateTime::parse_from_str(p[0], "%Y-%m-%d %H:%M")
            .map_err(|e| DataError::Api(format!("时间解析失败 {line}: {e}")))?;

        let c = t + chrono::Duration::minutes(interval.minutes());
        (NaiveDateTime::from(t), NaiveDateTime::from(c))
    } else {
        // 日线
        let d = NaiveDate::parse_from_str(p[0], "%Y-%m-%d %H:%M")
            .map_err(|e| DataError::Api(format!("时间解析失败 {line}: {e}")))?;

        let open = d
            .and_hms_opt(9, 30, 0)
            .ok_or_else(|| DataError::Api(format!("无效日期 {line}")))?;
        let close = d
            .and_hms_opt(15, 0, 0)
            .ok_or_else(|| DataError::Api(format!("无效日期 {line}")))?;
        (open, close)
    };

    let open = p[1].parse().unwrap_or(0.0);
    let close = p[2].parse().unwrap_or(0.0);
    let high: f64 = p[3].parse().unwrap_or(0.0);
    let low: f64 = p[4].parse().unwrap_or(0.0);

    Ok(Kline {
        open_time: SHANGHAI
            .from_local_datetime(&open_dt)
            .unwrap()
            .timestamp_millis(),
        open,
        high,
        low,
        close,
        volume: p[5].parse::<f64>().unwrap_or(0.0) * 100.0, // 手 → 股
        close_time: SHANGHAI
            .from_local_datetime(&close_dt)
            .unwrap()
            .timestamp_millis(),
        quote_volume: p[6].parse().unwrap_or(0.0), // 元
        trades_count: 0,                           // 东财无此字段
        is_closed: true,
    })
}

pub fn mark_last_bar_open(klines: &mut [Kline], now_ms: i64) {
    if let Some(last) = klines.last_mut() {
        if now_ms < last.close_time {
            last.is_closed = false;
        }
    }
}

/// 一行分时 CSV："2026-10-23 09:31,1311.50,1321.00,1323.65,1310.00,986,129903055.00,1314.441"
/// 顺序（fields2）：f51 时间, f52 开, f53 现价, f54 高, f55 低, f56 量(手), f57 额(元), f58 均价
pub fn parse_timeline_line(line: &str) -> Option<TimelinePoint> {
    let p: Vec<&str> = line.split(',').collect();
    if p.len() < 8 {
        return None;
    }
    Some(TimelinePoint {
        time: p[0].split(' ').nth(1)?.to_string(),
        price: p[2].parse().ok()?,
        avg_price: p[7].parse().ok()?,
        volume: p[5].parse().unwrap_or(0.0),
        amount: norm::yuan_to_wan(p[6].parse().unwrap_or(0.0)),
    })
}
