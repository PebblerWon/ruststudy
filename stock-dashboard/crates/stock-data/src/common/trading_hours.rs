use chrono::{Datelike, FixedOffset, NaiveTime, TimeZone, Timelike, Weekday};

const SHANGHAI: FixedOffset = FixedOffset::east_opt(8 * 3600).unwrap();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradingPhase {
    PreOpen,    // 开盘前（含 09:25-09:30 连续竞价前的静默段）
    Auction,    // 09:15-09:25 集合竞价
    Morning,    // 09:30-11:30
    LunchBreak, // 11:30-13:00
    Afternoon,  // 13:00-15:00
    Closed,     // 收盘 / 周末
}

/// 当前上海时间(时间+星期)
pub fn now_shanghai() -> (NaiveTime, Weekday) {
    let t = chrono::Utc::now().with_timezone(&SHANGHAI);
    (t.time(), t.weekday())
}

///
pub fn now_ms_shanghai() -> i64 {
    chrono::Utc::now()
        .with_timezone(&SHANGHAI)
        .timestamp_millis()
}

pub fn phase(now: NaiveTime, weekday: Weekday) -> TradingPhase {
    use TradingPhase::*;
    if matches!(weekday, Weekday::Sat | Weekday::Sun) {
        return Closed;
    }
    let (h, m) = (now.hour(), now.minute());
    let mins = h * 60 + m;
    match mins {
        0..=554 => PreOpen,      // < 09:15
        555..=565 => Auction,    // 09:15-09:25
        566..=569 => PreOpen,    // 09:25-09:30
        570..=690 => Morning,    // 09:30-11:30
        691..=779 => LunchBreak, // 11:30-13:00
        780..=900 => Afternoon,  // 13:00-15:00
        _ => Closed,
    }
}

impl TradingPhase {
    pub fn poll_interval_secs(self) -> u64 {
        match self {
            TradingPhase::Morning | TradingPhase::Afternoon | TradingPhase::Auction => 5,
            _ => 60,
        }
    }
}
