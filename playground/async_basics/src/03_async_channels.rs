//! ## 练习 12: 异步通道
//!
//! ### 学习目标
//! - 用 `tokio::sync::mpsc` 做异步生产者-消费者
//! - 理解异步环境下的发送与接收
//! - 了解 `select!` 宏的基本用法（选做）
//!
//! ### 背景
//!
//! 异步通道与同步通道类似，但它的 `recv()` 方法是异步的（需要 `.await`）。
//! 这使得它非常适合在 `async` 任务之间传递消息，而不会阻塞整个线程。
//!
//! ```rust,ignore
//! let (tx, mut rx) = mpsc::channel(32); // 带缓冲的通道
//! tx.send("Hello").await.unwrap();
//! let msg = rx.recv().await;
//! ```
//!
//! ### 你的任务
//!
//! 1. 实现 `producer` 函数，向通道发送一系列数字。
//! 2. 实现 `consumer` 函数，从通道接收数字并求和。
//! 3. 编写测试验证异步通道的通信逻辑。

// ────────────── 实现区域 ──────────────

use tokio::sync::mpsc;

/// 生产者：发送 0 到 n-1 的数字
pub async fn producer(tx: mpsc::Sender<u32>, n: u32) {
    for i in 0..n {
        // 提示：使用 send().await
        tx.send(i).await.unwrap();
    }
}

/// 消费者：接收数字并返回总和
pub async fn consumer(mut rx: mpsc::Receiver<u32>) -> u32 {
    let mut sum = 0;
    // 提示：循环接收直到通道关闭
    while let Some(val) = rx.recv().await {
        sum += val;
    }
    sum
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_channel_communication() {
        let (tx, rx) = mpsc::channel(10);

        // 启动生产者任务
        tokio::spawn(producer(tx, 5));

        // 在主任务中消费
        let sum = consumer(rx).await;

        // 0 + 1 + 2 + 3 + 4 = 10
        assert_eq!(sum, 10);
    }

    #[tokio::test]
    async fn test_channel_closes_when_dropped() {
        let (tx, mut rx) = mpsc::channel(1);
        tx.send(42).await.unwrap();
        drop(tx); // 显式关闭发送端

        assert_eq!(rx.recv().await, Some(42));
        assert_eq!(rx.recv().await, None); // 通道已关闭
    }
}
