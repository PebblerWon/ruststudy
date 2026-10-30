use super::eastmoney::EastmoneyClient;
use crate::common::trading_hours::{self, TradingPhase};
use crate::{DataError, Quote, data};
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

pub fn spawn_polling<T, Fut, F, G>(fetch: G, on_data: F) -> PollingHandle
where
    G: Fn() -> Fut + Send + 'static,
    T: Send + 'static,
    Fut: Future<Output = Result<T, DataError>> + Send + 'static,
    F: Fn(T) + Send + 'static,
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

            match fetch().await {
                Ok(data) => {
                    backoff_secs = 0;
                    on_data(data);
                }
                Err(e) => {
                    backoff_secs = (backoff_secs * 2).max(1).min(30);
                    tracing::warn!(error = %e, backoff_secs, "polling failed");
                }
            }

            tokio::time::sleep(Duration::from_secs(period)).await;
        }
    });
    PollingHandle { stop_tx }
}
