//! ## 练习 10: async/await 基础
//!
//! ### 学习目标
//! - 理解 `async fn` 返回的是 Future，不是立即执行
//! - 学会用 `#[tokio::main]` 标注 main 函数
//! - 用 `tokio::time::sleep` 做异步等待
//! - 理解 Future 的惰性求值
//!
//! ### 背景
//!
//! 在 Rust 中，`async` 函数不会立即运行。它们返回一个“未来”（Future），只有在被 `.await` 时才会开始执行。
//! 这允许我们高效地在一个线程上并发运行成千上万个任务。
//!
//! ```rust,ignore
//! async fn say_hello() {
//!     println!("Hello!");
//! }
//! // say_hello(); // 什么都不发生
//! say_hello().await; // 现在才执行
//! ```
//!
//! ### 你的任务
//!
//! 1. 实现 `greet_async` 函数，模拟一个耗时 1 秒的问候操作。
//! 2. 实现 `run_greetings` 函数，按顺序问候两个人。
//! 3. 编写测试验证异步逻辑的执行顺序和耗时。

// ────────────── 实现区域 ──────────────

use tokio::time::{sleep, Duration};

/// 模拟一个异步问候操作
pub async fn greet_async(name: &str) -> String {
    sleep(Duration::from_secs(1)).await;
    todo!("返回问候语")
}

/// 依次执行两个问候任务
pub async fn run_greetings() -> (String, String) {
    let first = greet_async("Alice").await;
    let second = greet_async("Bob").await;
    (first, second)
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_greet_returns_message() {
        let msg = greet_async("Rust").await;
        assert_eq!(msg, "Hello, Rust!");
    }

    #[tokio::test]
    async fn test_sequential_execution_time() {
        let start = Instant::now();
        let (first, second) = run_greetings().await;
        let duration = start.elapsed();

        assert_eq!(first, "Hello, Alice!");
        assert_eq!(second, "Hello, Bob!");
        
        // 顺序执行应该至少耗时 2 秒
        assert!(duration >= Duration::from_secs(2));
    }
}
