use std::sync::Arc;
use stock_data::{DataError, EastmoneyClient, PollingHandle, spawn_polling};

use crate::ui::overview_panel::NorthDeal;
pub struct DataBridge {
    rt: tokio::runtime::Runtime,
    pub client: Arc<EastmoneyClient>,
}

impl DataBridge {
    pub fn new() -> anyhow::Result<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()?;

        let client = Arc::new(EastmoneyClient::new()?);

        Ok(Self { rt, client })
    }

    /// 一次性请求：在 gpui 后台线程阻塞驱动 tokio future。
    /// ⚠️ 只允许在 cx.background_spawn 里调用，严禁在 UI（foreground）线程调用。
    pub fn block_on<F: Future>(&self, fut: F) -> F::Output {
        self.rt.block_on(fut)
    }

    /// 启动轮询：任务跑在 tokio runtime 内，结果经 tx 推给 UI。
    /// 返回句柄由 UI 侧保存（窗口关闭/退出时 stop）。
    pub fn start_polling_market_snapshot(
        &self,
        tx: async_channel::Sender<stock_data::MarketSnapshot>,
    ) -> PollingHandle {
        let client = self.client.clone();
        // spawn_polling 内部 tokio::spawn，必须在 runtime 上下文里调用
        self.rt.block_on(async move {
            spawn_polling(
                move || {
                    let client = client.clone();
                    async move { client.market_snapshot().await }
                },
                move |snapshot| {
                    let _ = tx.try_send(snapshot);
                },
            )
        })
    }

    /// 启动轮询：任务跑在 tokio runtime 内，结果经 tx 推给 UI。
    /// 返回句柄由 UI 侧保存（窗口关闭/退出时 stop）。
    pub fn start_polling_index_quotes(
        &self,
        tx: async_channel::Sender<Vec<stock_data::IndexQuote>>,
    ) -> PollingHandle {
        let client = self.client.clone();
        // spawn_polling 内部 tokio::spawn，必须在 runtime 上下文里调用

        self.rt.block_on(async move {
            spawn_polling(
                move || {
                    let client = client.clone();
                    async move { client.index_quotes().await }
                },
                move |data| {
                    let _ = tx.try_send(data);
                },
            )
        })
    }

    /// 启动轮询：查询市场行情
    pub fn start_polling_market_fund_flow(
        &self,
        tx: async_channel::Sender<stock_data::MarketFundFlow>,
    ) -> PollingHandle {
        let client = self.client.clone();
        // spawn_polling 内部 tokio::spawn，必须在 runtime 上下文里调用

        self.rt.block_on(async move {
            spawn_polling(
                move || {
                    let client = client.clone();
                    async move { client.get_market_fund_flow().await }
                },
                move |data| {
                    let _ = tx.try_send(data);
                },
            )
        })
    }
    pub fn get_all_a_codes(
        &self,
        tx: async_channel::Sender<Arc<Vec<String>>>,
    ) -> Result<(), DataError> {
        let client = self.client.clone();
        self.rt.block_on(async move {
            let data = client.get_all_a_codes().await.unwrap_or(Arc::new(vec![]));

            tx.try_send(data)
                .map_err(|err| DataError::ProcessError(err.to_string()))
        })
    }

    /// 启动轮询：查询市场行情
    pub fn start_polling_northbound_summary(
        &self,
        tx: async_channel::Sender<Vec<stock_data::NorthboundSummary>>,
    ) -> PollingHandle {
        let client = self.client.clone();
        // spawn_polling 内部 tokio::spawn，必须在 runtime 上下文里调用

        self.rt.block_on(async move {
            spawn_polling(
                move || {
                    let client = client.clone();
                    async move { client.northbound_summary().await }
                },
                move |data| {
                    let _ = tx.try_send(data);
                },
            )
        })
    }

    /// 启动轮询：查询市场行业板块
    pub fn start_polling_industry_board(
        &self,
        tx: async_channel::Sender<Vec<stock_data::StockBoard>>,
    ) -> PollingHandle {
        let client = self.client.clone();
        // spawn_polling 内部 tokio::spawn，必须在 runtime 上下文里调用

        self.rt.block_on(async move {
            spawn_polling(
                move || {
                    let client = client.clone();
                    async move { client.get_stock_industry_board().await }
                },
                move |data| {
                    let _ = tx.try_send(data);
                },
            )
        })
    }

    /// 启动轮询：查询市场概念板块
    pub fn start_polling_concept_board(
        &self,
        tx: async_channel::Sender<Vec<stock_data::StockBoard>>,
    ) -> PollingHandle {
        let client = self.client.clone();
        // spawn_polling 内部 tokio::spawn，必须在 runtime 上下文里调用

        self.rt.block_on(async move {
            spawn_polling(
                move || {
                    let client = client.clone();
                    async move { client.get_stock_concept_board().await }
                },
                move |data| {
                    let _ = tx.try_send(data);
                },
            )
        })
    }

    pub fn fetch_north_deal(&self, tx: async_channel::Sender<NorthDeal>) -> Result<(), DataError> {
        let client = self.client.clone();
        self.rt.block_on(async move {
            let sg = client
                .northbound_history(stock_data::MutualType::Sg, 1)
                .await?;
            let sz = client
                .northbound_history(stock_data::MutualType::Sz, 1)
                .await?;
            if sg.is_empty() || sz.is_empty() {
                return Err(DataError::Api("northbound_history 空数据".into()));
            }

            tx.try_send(NorthDeal {
                total_wan: sg[0].deal_amt + sz[0].deal_amt,
                date: sg[0].date,
            })
            .map_err(|err| DataError::ProcessError(err.to_string()))
        })
    }
}
