//! ## 练习 11: 并发 HTTP 请求
//!
//! ### 学习目标
//! - 用 `tokio::spawn` 并发执行多个异步任务
//! - 用 `tokio::join!` 等待所有任务完成
//! - 用 `reqwest` crate 发送 HTTP 请求
//! - 对比 `std::thread::spawn` 和 `tokio::spawn`
//!
//! ### 背景
//!
//! `tokio::spawn` 会创建一个后台任务（Task），它比系统线程更轻量。
//! `tokio::join!` 宏可以让你同时等待多个 Future，实现真正的并发。
//!
//! ```rust,ignore
//! let handle = tokio::spawn(async { /* ... */ });
//! let (a, b) = tokio::join!(fetch_a(), fetch_b());
//! ```
//!
//! ### 你的任务
//!
//! 1. 实现 `fetch_status` 函数，获取指定 URL 的 HTTP 状态码。
//! 2. 实现 `concurrent_fetch` 函数，并发请求两个 URL 并返回结果。
//! 3. 编写测试验证并发请求的正确性。

// ────────────── 实现区域 ──────────────

use reqwest;

/// 获取指定 URL 的状态码
pub async fn fetch_status(url: &str) -> Result<u16, reqwest::Error> {
    let resp = reqwest::get(url).await?;
    Ok(resp.status().as_u16())
}

/// 并发获取两个 URL 的状态码
pub async fn concurrent_fetch(
    url1: &str,
    url2: &str,
) -> (Result<u16, reqwest::Error>, Result<u16, reqwest::Error>) {
    // 提示：使用 tokio::join! 同时发起两个请求

    let res = tokio::join!(reqwest::get(url1), reqwest::get(url2));
    (
        res.0.map(|r| r.status().as_u16()),
        res.1.map(|r| r.status().as_u16()),
    )
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_status_ok() {
        // 使用一个稳定的测试地址
        let status = fetch_status("https://httpbin.org/status/200").await;
        assert_eq!(status.unwrap(), 200);
    }

    #[tokio::test]
    async fn test_concurrent_fetch_results() {
        let (res1, res2) = concurrent_fetch(
            "https://httpbin.org/status/201",
            "https://httpbin.org/status/404",
        )
        .await;

        assert_eq!(res1.unwrap(), 201);
        assert_eq!(res2.unwrap(), 404);
    }
}
