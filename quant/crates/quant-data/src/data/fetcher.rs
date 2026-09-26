use std::format;

use crate::common::error::QuantError;
use crate::common::models::{ExchangeInfo, Interval, Kline};
use crate::data::types::{RawExchangeInfoResponse, RawKlineResponse};
use reqwest::Client;

pub struct BinanceClient {
    client: Client,
    base_url: String,
}

impl BinanceClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    /// 获取历史 K 线数据
    ///
    /// # 参数
    /// - `symbol`: 交易对名称，如 "BTCUSDT"
    /// - `interval`: K 线周期
    /// - `start_time`: 起始时间（Unix 毫秒时间戳），可选
    /// - `limit`: 返回数量上限（Binance 最大 1000）
    pub async fn fetch_klines(
        &self,
        symbol: &str,
        interval: Interval,
        start_time: Option<i64>,
        limit: u32,
    ) -> Result<Vec<Kline>, QuantError> {
        let url = format!("{}/api/v3/klines", self.base_url);

        let mut request = self
            .client
            .get(&url)
            .query(&[("symbol", symbol), ("interval", interval.as_str())])
            .query(&[("limit", limit.to_string())]);

        if let Some(st) = start_time {
            request = request.query(&["startTime", st.to_string().as_str()]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| QuantError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();

            return Err(QuantError::Api(format!(
                "Binance API 错误：status={}, body={}",
                status, body
            )));
        }

        let raw: RawKlineResponse = response
            .json()
            .await
            .map_err(|e| QuantError::Parse(e.to_string()))?;
        raw.into_klines()
    }

    /// 获取交易对信息

    pub async fn fetch_exchange_info(&self) -> Result<Vec<ExchangeInfo>, QuantError> {
        let url = format!("{}/api/v3/exchangeInfo", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| QuantError::Network(e.to_string()))?;

        let raw: RawExchangeInfoResponse = response
            .json()
            .await
            .map_err(|e| QuantError::Parse(e.to_string()))?;

        Ok(raw
            .symbols
            .into_iter()
            .filter(|s| s.status == "TRADING")
            .map(|s| ExchangeInfo {
                symbol: s.symbol,
                status: s.status,
                base_asset_precision: s.base_asset_precision,
                quote_asset_precision: s.quote_asset_precision,
            })
            .collect())
    }
}
