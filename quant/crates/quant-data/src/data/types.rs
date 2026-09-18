use crate::common::error::QuantError;
use crate::common::models::Kline;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawKlineResponse(Vec<RawKlineRow>);

#[derive(Debug, Deserialize)]
pub struct RawKlineRow(
    pub i64,    // 0: open_time
    pub String, // 1: open
    pub String, // 2: high
    pub String, // 3: low
    pub String, // 4: close
    pub String, // 5: volume
    pub i64,    // 6: close_time
    pub String, // 7: quote_volume
    pub u32,    // 8: trades_count
    pub String, // 9: taker_buy_volume（忽略）
    pub String, // 10: taker_buy_quote_volume（忽略）
    pub String, // 11: ignore（忽略）
);

impl RawKlineResponse {
    pub fn into_klines(self) -> Result<Vec<Kline>, QuantError> {
        self.0
            .into_iter()
            .map(|row| {
                Ok(Kline {
                    open_time: row.0,
                    open: row
                        .1
                        .parse()
                        .map_err(|_| QuantError::Parse("open price".into()))?,
                    high: row
                        .2
                        .parse()
                        .map_err(|_| QuantError::Parse("high price".into()))?,
                    low: row
                        .3
                        .parse()
                        .map_err(|_| QuantError::Parse("low price".into()))?,
                    close: row
                        .4
                        .parse()
                        .map_err(|_| QuantError::Parse("close price".into()))?,
                    volume: row
                        .5
                        .parse()
                        .map_err(|_| QuantError::Parse("Volume".into()))?,
                    close_time: row.6,
                    quote_volume: row
                        .7
                        .parse()
                        .map_err(|_| QuantError::Parse("quote_volume".into()))?,
                    trades_count: row.8,
                    is_closed: true,
                })
            })
            .collect()
    }
}
