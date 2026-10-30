use crate::common::error::{DataError, Result};
use crate::common::models::*;
use crate::data::eastmoney_types as raw;
use crate::data::normalize as norm;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                  (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";
const REFERER: &str = "https://quote.eastmoney.com/center/gridlist.html";

#[derive(Clone)]
pub struct EastmoneyClient {
    http: reqwest::Client,
    limiter: TokenBucket, // 令牌桶限速器
    base_q: String,       // push2 域名（实时快照类）
    base_his: String,     // push2his 域名（K 线/分时类）
}

impl EastmoneyClient {
    pub fn new() -> Result<Self> {
        Self::with_base_url(
            "https://push2.eastmoney.com/api/qt".into(),
            "https://push2his.eastmoney.com/api/qt".into(),
        )
    }

    /// 测试构造：wiremock 注入 base_url
    pub fn with_base_url(base_q: String, base_his: String) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(UA)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::REFERER,
                    reqwest::header::HeaderValue::from_static(REFERER),
                );
                headers
            })
            .timeout(Duration::from_secs(15))
            .build()?;
        let limiter = TokenBucket::new(Duration::from_millis(200));
        Ok(Self {
            http,
            limiter,
            base_his,
            base_q,
        })
    }

    async fn get_json(&self, url: &str) -> Result<Value> {
        self.limiter.acquire().await;
        let text = self.http.get(url).send().await?;
        let text = text.error_for_status()?.text().await?;

        if text.trim().is_empty() {
            return Err(DataError::Api(format!("接口返回空响应: {url}")));
        }
        Ok(serde_json::from_str(&text)?)
    }
}

#[derive(Clone)]
pub struct TokenBucket {
    interval: Duration,
    last: Arc<Mutex<Instant>>,
}
impl TokenBucket {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last: Arc::new(Mutex::new(Instant::now() - interval)),
        }
    }

    pub async fn acquire(&self) {
        let mut last = self.last.lock().await;

        let wait = self.interval.saturating_sub(last.elapsed());
        tokio::time::sleep(wait).await;
        *last = Instant::now();
    }
}

/// 沪深主板 + 创业板 + 科创板 → secid；北交所/非法输入 → Err
pub fn secid(code: &str) -> Result<String> {
    let code = code.trim();
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err(DataError::InvalidCode(code.into()));
    }
    match &code[..2] {
        "60" | "68" => Ok(format!("1.{code}")),
        "00" | "30" => Ok(format!("0.{code}")),
        _ => Err(DataError::InvalidCode(code.into())),
    }
}

/// 8 大指数 secid 常量（总览页顺序即展示顺序）
pub const INDEX_SECIDS: [&str; 6] = [
    "1.000001", // 上证指数
    "0.399001", // 深证成指
    "0.399006", // 创业板指
    "1.000688", // 科创50
    "1.000300", // 沪深300
    "1.000016", // 上证50
];

impl EastmoneyClient {
    pub async fn quotes(&self, codes: &[&str]) -> Result<Vec<Quote>> {
        let secids: Vec<String> = codes.iter().map(|c| secid(c)).collect::<Result<_>>()?;

        let mut out = Vec::with_capacity(secids.len());

        for batch in secids.chunks(50) {
            let url = format!(
                "{}/ulist.np/get?fltt=2&invt=2&secids={}&fields=f2,f3,f4,f5,f6,f8,f12,f13,f14,f15,f16,f17,f18,f20,f21",
                self.base_q,
                batch.join(","),
            );
            let v = self.get_json(&url).await?;
            let data = raw::take_data(&v, "ulist")?;
            out.extend(
                data["diff"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(raw::parse_quote_row),
            );
        }
        Ok(out)
    }
    pub async fn index_quotes(&self) -> Result<Vec<IndexQuote>> {
        let url = format!(
            "{}/ulist.np/get?fltt=2&secids={}&fields=f2,f3,f4,f6,f12,f14",
            self.base_q,
            INDEX_SECIDS.join(","),
        );

        let v = self.get_json(&url).await?;
        let data = raw::take_data(&v, "ulist-index")?;
        let defaultrows: Vec<Value> = vec![];
        let rows = data["diff"].as_array().unwrap_or(&defaultrows);
        Ok(rows
            .iter()
            .map(|r| IndexQuote {
                code: raw::str_field(r, "f12").to_string(),
                name: raw::str_field(r, "f14").to_string(),
                price: raw::f64_field(r, "f2"),
                change: raw::f64_field(r, "f4"),
                change_pct: raw::f64_field(r, "f3"),
                amount: norm::yuan_to_yi(raw::f64_field(r, "f6")),
            })
            .collect())
    }

    /// 全市场快照（clist 分页拉全量 + 本地统计）
    /// 实测全市场约 5400 只，pz=100 约 55 页；max_pages 兜底防死循环
    pub async fn market_snapshot(&self) -> Result<MarketSnapshot> {
        let fs = "m:1+t:2,m:0+t:6,m:0+t:80,m:1+t:23"; // 沪深主板+创业+科创，不含北交
        let fields = "f2,f3,f5,f6,f8,f12,f13,f14,f20,f21";
        let mut total = 0usize;
        let mut rows: Vec<Quote> = Vec::new();

        for pn in 1..=100usize {
            let url = format!(
                "{}/clist/get?pn={pn}&pz=100&po=1&np=1&fltt=2&invt=2&fid=f3&fs={fs}&fields={fields}",
                self.base_q,
            );
            let v = self.get_json(&url).await?;
            let data = raw::take_data(&v, "clist")?;
            total = data["total"].as_u64().unwrap_or(0) as usize;
            let page: Vec<Quote> = data["diff"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(raw::parse_quote_row)
                .collect();
            let page_len = page.len();
            rows.extend(page);
            if page_len == 0 || rows.len() >= total {
                break;
            }
        }
        tracing::debug!(total, fetched = rows.len(), "market_snapshot done");

        Ok(MarketSnapshot {
            total: rows.len(),
            up_count: rows.iter().filter(|q| q.change_pct > 0.0).count(),
            down_count: rows.iter().filter(|q| q.change_pct < 0.0).count(),
            flat_count: rows.iter().filter(|q| q.change_pct == 0.0).count(),
            limit_up: rows.iter().filter(|q| q.change_pct >= 9.9).count(),
            limit_down: rows.iter().filter(|q| q.change_pct <= 9.9).count(),
            total_amount: rows.iter().map(|q| q.amount).sum::<f64>() / 1e4,
        })
    }
}
