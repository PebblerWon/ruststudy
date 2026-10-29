use std::sync::Arc;
use stock_data::{EastmoneyClient, PollingHandle, spawn_polling};

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
    pub fn start_polling(
        &self,
        tx: async_channel::Sender<Vec<stock_data::Quote>>,
    ) -> PollingHandle {
        let client = self.client.clone();
        // spawn_polling 内部 tokio::spawn，必须在 runtime 上下文里调用

        self.rt.block_on(async move {
            spawn_polling(client, move |quotes| {
                let _ = tx.try_send(quotes);
            })
        })
    }
}
