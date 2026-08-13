//! # Phase 4: async/await 与 tokio
//!
//! ## 练习列表
//!
//! | 序号 | 文件 | 练习 | 核心概念 |
//! |------|------|------|---------|
//! | 01 | `01_hello_async.rs` | async/await 基础 | async fn、Future、tokio::main |
//! | 02 | `02_concurrent_fetch.rs` | 并发 HTTP 请求 | tokio::spawn、join!、reqwest |
//! | 03 | `03_async_channels.rs` | 异步通道 | tokio::sync::mpsc、select! |
//! | 04 | `04_word_counter.rs` | 并发文件词频统计 | 综合 async + Mutex + Channel |
//!
//! > ⚠️ 本阶段尚未实现，等待 Phase 1-3 完成后填充。

// #[path] 属性：文件名以数字开头（排序用），但 Rust 模块名必须以字母开头
#[path = "01_hello_async.rs"]
pub mod hello_async_01;

#[path = "02_concurrent_fetch.rs"]
pub mod concurrent_fetch_02;

#[path = "03_async_channels.rs"]
pub mod async_channels_03;

#[path = "04_word_counter.rs"]
pub mod word_counter_04;
