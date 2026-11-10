use crate::common::error::{DataError, Result};
use crate::common::models::*;
use crate::data::cache::{AnyTtlCach, cache_prefix};
use crate::data::eastmoney_types as raw;
use crate::data::normalize as norm;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                  (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";
const REFERER: &str = "https://quote.eastmoney.com/center/gridlist.html";

const A_SHARE_LIST_URL: &str = "https://assets.linkdiary.cn/shares/zh_a_list.json";

const HIS_UT: &str = "b2884a393a59ad64002292a3e90d46a5";

#[derive(Clone)]
pub struct EastmoneyClient {
    http: reqwest::Client,
    limiter: TokenBucket,   // 令牌桶限速器
    base_q: String,         // push2 域名（实时快照类）
    base_his: String,       // push2his 域名（K 线/分时类）
    cache: Arc<AnyTtlCach>, // 缓存
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
            cache: Arc::new(AnyTtlCach::new()),
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
    /// 沪主板+深主板+创业板+科创板+北交所，在市约 5580 只
    /// （官方中上协 2026-09 口径 5579 家上市公司；我方为证券口径，A+B 双重上市只计 A 股证券）
    /// ⚠️ 1. fs 不能用裸 m:0+t:81 —— 会混入新三板 6700+ 僵尸品种；
    ///       北交所需用 m:0+t:81+s:2048 精确圈定（365 只，代码段 82/83/87/88/92）
    /// ⚠️ 2. 市场过滤符会附带历史退市股档案（约 350 条，简称带"退"/"PT"，f2 返 "-"），
    ///       已按 is_suspended 剔除，不参与 total 与涨跌统计
    /// ⚠️ 3. B 股（沪 m:1+t:3 / 深 m:0+t:7，共 114 条）不在统计内：官方口径含 B 公司，
    ///       但 B 股流动性趋零，涨跌家数无参考价值
    /// 实测在市+停牌+退市约 5900+ 条，pz=100 约 60 页；100 页上限足够，防死循环兼兜底
    pub async fn market_snapshot(&self) -> Result<MarketSnapshot> {
        let fs = "m:1+t:2,m:0+t:6,m:0+t:80,m:1+t:23,m:0+t:81+s:2048"; // 沪主板+深主板+创业板+科创板+北交所
        let fields = "f2,f3,f5,f6,f8,f12,f13,f14,f20,f21";
        let mut total = 0usize;
        let mut rows: Vec<Quote> = Vec::new();

        for pn in 1..=100 {
            let url = format!(
                "{}/clist/get?pn={pn}&pz=100&po=1&np=1&fltt=2&invt=2&fid=f3&fs={fs}&fields={fields}",
                self.base_q,
            );
            let v = self.get_json(&url).await?;
            let data = raw::take_data(&v, &url)?;
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
        if rows.len() < total {
            tracing::warn!(
                fetched = rows.len(),
                total,
                "market_snapshot 分页未拉全，涨跌统计可能失真"
            );
        }

        // 停牌股与退市股档案的 f2 均返回 "-"（is_suspended），
        // 二者都不该计入在市家数与涨跌统计，统一剔除
        let active: Vec<&Quote> = rows.iter().filter(|q| !q.is_suspended).collect();

        Ok(MarketSnapshot {
            // 在市证券数（含真平盘）；停牌与退市档案不计入，保证 up+down+flat == total
            total: active.len(),
            up_count: active.iter().filter(|q| q.change_pct > 0.0).count(),
            down_count: active.iter().filter(|q| q.change_pct < 0.0).count(),
            flat_count: active.iter().filter(|q| q.change_pct == 0.0).count(),
            limit_up: active.iter().filter(|q| q.change_pct >= 9.9).count(),
            limit_down: active.iter().filter(|q| q.change_pct <= -9.9).count(),
            total_amount: rows.iter().map(|q| q.amount).sum::<f64>() / 1e4,
        })
    }

    /// 获取全市场所有代码
    pub async fn get_all_a_codes(&self) -> Result<Arc<Vec<String>>> {
        if let Some(cache_value) = self
            .cache
            .get::<Vec<String>>(cache_prefix::ALL_A_CODES)
            .await
        {
            return Ok(cache_value);
        }
        let v = self.get_json(A_SHARE_LIST_URL).await?;
        let value_data = v
            .get("list")
            .ok_or_else(|| DataError::Api(format!("data 为空")))?;
        if let Some(data) = value_data.as_array() {
            let insert_value: Vec<String> = data
                .iter()
                .map(|i| i.as_str().unwrap_or("").to_string())
                .filter(|i| !i.is_empty())
                .collect();
            let res = self
                .cache
                .insert(
                    cache_prefix::ALL_A_CODES,
                    insert_value,
                    Duration::from_hours(24),
                )
                .await;

            Ok(res)
        } else {
            return Err(DataError::Api(format!(
                "接口返回异常: {}",
                A_SHARE_LIST_URL
            )));
        }
    }

    pub async fn get_market_fund_flow(&self) -> Result<MarketFundFlow> {
        const FFLOW_FIELDS_2: &str = "f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61,f62,f63,f64,f65";

        let url = format!(
            "{}/stock/fflow/daykline/get?lmt=0&klt=101&secid=1.000001&secid2=0.399001&fields1=f1%2Cf2%2Cf3%2Cf7&fields2={FFLOW_FIELDS_2}&ut={}",
            self.base_his, HIS_UT
        );
        let v = self.get_json(&url).await?;
        let data = raw::take_data(&v, &url)?;
        let defaultrows: Vec<Value> = vec![];
        let klines = data["klines"].as_array().unwrap_or(&defaultrows);
        raw::parse_market_fund_flow_kline(klines.last())
    }
}
