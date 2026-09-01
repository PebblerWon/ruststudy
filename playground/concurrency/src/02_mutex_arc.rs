//! ## 练习 8: 用 Mutex + Arc 共享可变状态
//!
//! ### 学习目标
//! - 理解 Mutex 互斥锁：同一时刻只允许一个线程访问数据
//! - 理解 Arc 原子引用计数：多线程安全的共享所有权
//! - 常见组合 `Arc<Mutex<T>>`
//!
//! ### 背景
//!
//! 在多线程环境下，`Rc` 是不安全的（因为它不是原子的）。我们需要 `Arc` (Atomic Reference Counted)。
//! 同时，为了安全地修改共享数据，必须使用 `Mutex` 加锁。
//!
//! ```rust,ignore
//! let counter = Arc::new(Mutex::new(0));
//! let c_clone = Arc::clone(&counter);
//! std::thread::spawn(move || {
//!     let mut num = c_clone.lock().unwrap();
//!     *num += 1;
//! });
//! ```
//!
//! ### 你的任务
//!
//! 1. 实现 `increment_counter` 函数，开启 10 个线程，每个线程将计数器加 1。
//! 2. 验证最终计数器的值是否为 10。
//! 3. 尝试在不加锁的情况下修改数据（观察编译错误）。

// ────────────── 实现区域 ──────────────

use std::sync::{Arc, Mutex};
use std::thread;

/// 启动 num_threads 个线程，每个线程将计数器加 1
pub fn increment_counter(num_threads: u32) -> u32 {
    let counter = Arc::new(Mutex::new(0u32));
    let mut handles = vec![];

    for _ in 0..num_threads {
        let counter_clone = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            // 提示：获取锁并修改内部值
            let mut thread_counter = counter_clone.lock().unwrap();
            *thread_counter += 1;
        });

        handles.push(handle);
    }

    // 等待所有线程结束
    for handle in handles {
        handle.join().unwrap();
    }

    // 返回最终结果
    let r = counter.lock().unwrap();
    *r
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_single_thread() {
        assert_eq!(increment_counter(1), 1);
    }

    #[test]
    fn test_increment_multi_threads() {
        // 无论运行多少次，结果都应该是 10
        assert_eq!(increment_counter(10), 10);
    }

    #[test]
    fn test_mutex_safety() {
        // 验证 Mutex 确实保证了互斥访问
        // 这里可以通过压力测试来间接验证，或者检查 Arc::strong_count
        let counter = Arc::new(Mutex::new(0));
        assert_eq!(Arc::strong_count(&counter), 1);
    }
}
