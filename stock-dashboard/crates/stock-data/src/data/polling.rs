use super::eastmoney::EastmoneyClient;
use crate::Quote;
use crate::common::trading_hours::{self, TradingPhase};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

/// 轮询句柄：drop 不停止任务，需调用 stop()（watch 通知）
pub struct PollingHandle {
    stop_tx: watch::Sender<bool>,
}

impl PollingHandle {
    pub fn stop(&self) {
        let _ = self.stop_tx.send(true);
    }
}

/// 启动轮询任务。
/// `on_quotes` 在 tokio 线程被调用，Phase 3 中它内部做 cx.update + cx.notify。
/// 失败退避：1s→2s→4s→…→30s 封顶，成功后恢复时段标准间隔。
pub fn spawn_polling<F>(client: Arc<EastmoneyClient>, on_quotes: F) -> PollingHandle
where
    F: Fn(Vec<Quote>) + Send + 'static,
{
    let (stop_tx, stop_rx) = watch::channel(false);

    tokio::spawn(async move {
        let mut backoff_secs = 0u64;
        loop {
            if *stop_rx.borrow() {
                return;
            }
            let phase = {
                let (t, w) = trading_hours::now_shanghai();
                trading_hours::phase(t, w)
            };

            let period = if backoff_secs > 0 {
                backoff_secs
            } else if phase == TradingPhase::Morning || phase == TradingPhase::Afternoon {
                5
            } else {
                60
            };

            tokio::time::sleep(Duration::from_secs(period)).await;

            match client.quotes(&["600519", "000001"]).await {
                Ok(qs) => {
                    backoff_secs = 0;
                    on_quotes(qs);
                }
                Err(e) => {
                    backoff_secs = (backoff_secs * 2).max(1).min(30);
                    tracing::warn!(error = %e, backoff_secs, "polling failed");
                }
            }
        }
    });
    PollingHandle { stop_tx }
}
