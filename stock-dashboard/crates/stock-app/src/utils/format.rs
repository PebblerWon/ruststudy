pub fn format_wan_yi_with_decimals(value: f64, decimals: usize) -> String {
    let (num, unit) = if value >= 100_000_000.0 {
        (value / 100_000_000.0, "亿")
    } else if value >= 100_000.0 {
        (value / 100_000.0, "万")
    } else {
        (value, "")
    };
    let s = format!("{:.1$}", num, decimals);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    format!("{}{}", s, unit)
}
pub fn format_wan_yi(value: f64) -> String {
    format_wan_yi_with_decimals(value, 2)
}
