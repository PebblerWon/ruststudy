use crate::common::error::QuantError;
use crate::common::models::{Interval, Kline};
use crate::data;
use polars::prelude::*;
use std::path::{Path, PathBuf};

/// K 线数据本地缓存
/// 使用 Parquet 文件格式，利用 polars 的高效列式 IO
pub struct KlineStore {
    /// 缓存根目录（如 ./data/klines/）
    cache_dir: PathBuf,
}

impl KlineStore {
    pub fn new(cache_dir: &Path) -> Result<Self, QuantError> {
        std::fs::create_dir_all(cache_dir).map_err(|e| QuantError::Io(e.to_string()))?;
        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
        })
    }

    /// 生成缓存文件路径：{cache_dir}/{symbol}/{interval}.parquet
    fn file_path(&self, symbol: &str, interval: Interval) -> PathBuf {
        self.cache_dir
            .join(symbol.to_lowercase())
            .join(format!("{}.parquet", interval.as_str()))
    }

    pub fn save_klines(
        &self,
        symbol: &str,
        interval: Interval,
        klines: &[Kline],
    ) -> Result<(), QuantError> {
        if klines.is_empty() {
            return Ok(());
        }

        let df = klines_to_dataframe(klines)?;
        let path = self.file_path(symbol, interval);
        let merged = if path.exists() {
            let existing = self.load_klines(symbol, interval)?;
            let existing_df = klines_to_dataframe(&existing)?;
            // 纵向拼接后按 open_time 去重（polars 0.46 使用 lazy concat）
            concat(&[existing_df.lazy(), df.lazy()], UnionArgs::default())?
                .unique(
                    Some(Vec::from(["open_time".to_string()])),
                    UniqueKeepStrategy::First,
                )
                .collect()?
        } else {
            df
        };
        let mut sorted = merged.sort(["open_time"], SortMultipleOptions::default())?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| QuantError::Io(e.to_string()))?;
        }
        let file = std::fs::File::create(&path).map_err(|e| QuantError::Io(e.to_string()))?;

        ParquetWriter::new(file).finish(&mut sorted)?;

        tracing::info!(
            symbol = symbol,
            interval = interval.as_str(),
            count = sorted.height(),
            "K 线数据已保存"
        );
        Ok(())
    }

    pub fn load_klines(&self, symbol: &str, interval: Interval) -> Result<Vec<Kline>, QuantError> {
        let path = self.file_path(symbol, interval);

        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = std::fs::File::open(&path).map_err(|e| QuantError::Io(e.to_string()))?;
        let df = ParquetReader::new(file).finish()?;

        let a = dataframe_to_klines(&df)?;
        Ok(a)
    }

    pub fn get_data_range(
        &self,
        symbol: &str,
        interval: Interval,
    ) -> Result<Option<(i64, i64)>, QuantError> {
        let klines = self.load_klines(symbol, interval)?;
        if klines.is_empty() {
            return Ok(None);
        }
        let min_time = klines.first().unwrap().open_time;
        let max_time = klines.last().unwrap().open_time;

        Ok(Some((min_time, max_time)))
    }
}

/// 将 Kline 切片转为 polars DataFrame
fn klines_to_dataframe(klines: &[Kline]) -> Result<DataFrame, QuantError> {
    let open_times: Vec<i64> = klines.iter().map(|k| k.open_time).collect();

    let opens: Vec<f64> = klines.iter().map(|k| k.open).collect();
    let highs: Vec<f64> = klines.iter().map(|k| k.high).collect();
    let lows: Vec<f64> = klines.iter().map(|k| k.low).collect();
    let closes: Vec<f64> = klines.iter().map(|k| k.close).collect();
    let volumes: Vec<f64> = klines.iter().map(|k| k.volume).collect();
    let close_times: Vec<i64> = klines.iter().map(|k| k.close_time).collect();
    let quote_volumes: Vec<f64> = klines.iter().map(|k| k.quote_volume).collect();
    let trades_counts: Vec<u32> = klines.iter().map(|k| k.trades_count).collect();
    let is_closed_list: Vec<bool> = klines.iter().map(|k| k.is_closed).collect();

    let df = df! {
        "open_time"=>&open_times,
        "open"=>&opens,
        "high"=>&highs,
        "low"=>&lows,
        "close"=>&closes,
        "volume"=>&volumes,
        "close_time"=>&close_times,
        "quote_volume" => &quote_volumes,
        "trades_count" => &trades_counts,
        "is_closed" => &is_closed_list,
    }?;
    Ok(df)
}

fn dataframe_to_klines(df: &DataFrame) -> Result<Vec<Kline>, QuantError> {
    let open_times = df
        .column("open_time")
        .map_err(|_| QuantError::Parse("缺少 open_time 列".into()))?
        .i64()
        .map_err(|_| QuantError::Parse("open_time 列类型不匹配".into()))?;

    let opens = df
        .column("open")
        .map_err(|_| QuantError::Parse("缺少 open 列".into()))?
        .f64()
        .map_err(|_| QuantError::Parse("open 列类型不匹配".into()))?;
    let highs = df
        .column("high")
        .map_err(|_| QuantError::Parse("缺少 high 列".into()))?
        .f64()
        .map_err(|_| QuantError::Parse("high 列类型不匹配".into()))?;
    let lows = df
        .column("low")
        .map_err(|_| QuantError::Parse("缺少 low 列".into()))?
        .f64()
        .map_err(|_| QuantError::Parse("low 列类型不匹配".into()))?;
    let closes = df
        .column("close")
        .map_err(|_| QuantError::Parse("缺少 close 列".into()))?
        .f64()
        .map_err(|_| QuantError::Parse("close 列类型不匹配".into()))?;
    let volumes = df
        .column("volume")
        .map_err(|_| QuantError::Parse("缺少 volume 列".into()))?
        .f64()
        .map_err(|_| QuantError::Parse("volume 列类型不匹配".into()))?;
    let close_times = df
        .column("close_time")
        .map_err(|_| QuantError::Parse("缺少 close_time 列".into()))?
        .i64()
        .map_err(|_| QuantError::Parse("close_time 列类型不匹配".into()))?;
    let quote_volumes = df
        .column("quote_volume")
        .map_err(|_| QuantError::Parse("缺少 quote_volume 列".into()))?
        .f64()
        .map_err(|_| QuantError::Parse("quote_volume 列类型不匹配".into()))?;
    let trades_counts = df
        .column("trades_count")
        .map_err(|_| QuantError::Parse("缺少 trades_count 列".into()))?
        .u32()
        .map_err(|_| QuantError::Parse("trades_count 列类型不匹配".into()))?;
    let is_closeds = df
        .column("is_closed")
        .map_err(|_| QuantError::Parse("缺少 is_closed 列".into()))?
        .bool()
        .map_err(|_| QuantError::Parse("is_closed 列类型不匹配".into()))?;
    let len = opens.len();
    let mut klines = Vec::with_capacity(len);
    for i in 0..len {
        klines.push(Kline {
            open_time: open_times
                .get(i)
                .ok_or_else(|| QuantError::Parse(format!("open_time 第 {} 行为空", i)))?,
            open: opens
                .get(i)
                .ok_or_else(|| QuantError::Parse(format!("open 第 {} 行为空", i)))?,
            high: highs
                .get(i)
                .ok_or_else(|| QuantError::Parse(format!("high 第 {} 行为空", i)))?,
            low: lows
                .get(i)
                .ok_or_else(|| QuantError::Parse(format!("low 第 {} 行为空", i)))?,
            close: closes
                .get(i)
                .ok_or_else(|| QuantError::Parse(format!("close 第 {} 行为空", i)))?,
            volume: volumes
                .get(i)
                .ok_or_else(|| QuantError::Parse(format!("volume 第 {} 行为空", i)))?,
            close_time: close_times
                .get(i)
                .ok_or_else(|| QuantError::Parse(format!("close_time 第 {} 行为空", i)))?,
            quote_volume: quote_volumes
                .get(i)
                .ok_or_else(|| QuantError::Parse(format!("quote_volume 第 {} 行为空", i)))?,
            trades_count: trades_counts
                .get(i)
                .ok_or_else(|| QuantError::Parse(format!("trades_count 第 {} 行为空", i)))?,
            is_closed: is_closeds
                .get(i)
                .ok_or_else(|| QuantError::Parse(format!("is_closed 第 {} 行为空", i)))?,
        });
    }
    Ok(klines)
}
