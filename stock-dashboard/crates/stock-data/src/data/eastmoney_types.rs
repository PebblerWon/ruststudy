use crate::common::error::{DataError, Result as DataErrorResult};
use crate::common::models::*;
use serde::Deserialize;
use serde_json::Value;

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
