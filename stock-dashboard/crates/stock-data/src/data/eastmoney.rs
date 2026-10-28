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
        let text = self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

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
pub const INDEX_SECIDS: [&str; 8] = [
    "1.000001", // 上证指数
    "0.399001", // 深证成指
    "0.399006", // 创业板指
    "1.000688", // 科创50
    "1.000300", // 沪深300
    "1.000016", // 上证50
    "1.000905", // 中证500
    "0.899050", // 北证50（北交所指数本身可用，只是北交个股不支持）
];
