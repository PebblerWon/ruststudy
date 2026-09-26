mod fetcher;
mod kline_store;
mod types;

use crate::common::error::QuantError;
use crate::common::models::{Interval, Kline};
use crate::data::fetcher::BinanceClient;
use crate::data::kline_store::KlineStore;
use std::sync::Arc;

pub struct DataService {
    client: BinanceClient,
    store: KlineStore,
}

impl DataService {
    pub fn new(client: BinanceClient, store: KlineStore) -> Self {
        Self { client, store }
    }

    /// 获取 K 线数据（优先读本地缓存，缺失时从 API 获取）
    ///
    /// # 参数
    /// - `symbol`: 交易对名称
    /// - `interval`: K 线周期
    /// - `limit`: 需要的 K 线数量
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: Interval,
        limit: u32,
    ) -> Result<Vec<Kline>, QuantError> {
        let range = self.store.get_data_range(symbol, interval)?;
        match range {
            Some((_min, max_time)) => {
                let start_time = max_time + interval.as_millis();

                let new_klines = self
                    .client
                    .fetch_klines(symbol, interval, Some(start_time), limit)
                    .await?;
                if !new_klines.is_empty() {
                    self.store.save_klines(symbol, interval, &new_klines)?;
                }

                let mut all = self.store.load_klines(symbol, interval)?;
                if all.len() > limit as usize {
                    all = all.split_off(all.len() - limit as usize);
                }
                Ok(all)
            }
            None => {
                let klines = self
                    .client
                    .fetch_klines(symbol, interval, None, limit)
                    .await?;
                self.store.save_klines(symbol, interval, &klines)?;
                Ok(klines)
            }
        }
    }

    /// 批量获取多个交易对的K线数据
    pub async fn get_klines_batch(
        &self,
        symbols: &[&str],
        interval: Interval,
        limit: u32,
    ) -> Result<Vec<(String, Vec<Kline>)>, QuantError> {
        let mut results = Vec::new();
        for symbol in symbols {
            let klines = self.get_klines(symbol, interval, limit).await?;
            results.push((symbol.to_string(), klines));
        }
        Ok(results)
    }
}
