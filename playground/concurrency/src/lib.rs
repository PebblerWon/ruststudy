//! # Phase 3: 多线程与并发
//!
//! ## 练习列表
//!
//! | 序号 | 文件 | 练习 | 核心概念 |
//! |------|------|------|---------|
//! | 01 | `01_threads.rs` | 并行计算斐波那契 | thread::spawn、move 闭包、JoinHandle |
//! | 02 | `02_mutex_arc.rs` | 共享计数器 | Mutex<T>、Arc<T>、lock() |
//! | 03 | `03_channels.rs` | 生产者-消费者 | mpsc::channel、send/recv |
//!
//! > ⚠️ 本阶段尚未实现，等待 Phase 1 完成后填充。

// #[path] 属性：文件名以数字开头（排序用），但 Rust 模块名必须以字母开头
#[path = "01_threads.rs"]
pub mod threads_01;

#[path = "02_mutex_arc.rs"]
pub mod mutex_arc_02;

#[path = "03_channels.rs"]
pub mod channels_03;
