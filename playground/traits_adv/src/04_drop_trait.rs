//! ## 练习 17: Drop trait
//!
//! ### 学习目标
//! - 实现 `Drop` trait 自定义资源释放逻辑
//! - 理解 RAII (Resource Acquisition Is Initialization) 模式
//! - 理解 `Drop` 的调用时机（离开作用域时自动调用）
//!
//! ### 背景
//!
//! Rust 没有析构函数，而是通过 `Drop` trait 来管理资源的生命周期。
//! 当一个值即将被销毁时，Rust 会自动调用其 `drop` 方法。这对于关闭文件、释放锁或打印日志非常有用。
//!
//! ```rust,ignore
//! impl Drop for MyType {
//!     fn drop(&mut self) {
//!         println!("Cleaning up!");
//!     }
//! }
//! ```
//!
//! ### 你的任务
//!
//! 1. 定义一个 `FileHandle` 结构体，模拟一个文件句柄。
//! 2. 实现 `Drop` trait，在句柄销毁时打印一条“已关闭”的消息。
//! 3. 编写测试验证 `drop` 方法的执行时机。

// ────────────── 实现区域 ──────────────

use std::sync::{Arc, Mutex};

/// 模拟一个文件句柄
pub struct FileHandle {
    pub name: String,
    // 用于在测试中追踪 drop 是否被调用
    pub status: Arc<Mutex<String>>,
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        // 提示：更新状态并打印消息
        let mut s = self.status.lock().unwrap();
        *s = format!("Closed {}", self.name);
        println!("Dropping file handle: {}", self.name);
    }
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drop_is_called() {
        let status = Arc::new(Mutex::new("Open".to_string()));

        {
            let _handle = FileHandle {
                name: "test.txt".to_string(),
                status: Arc::clone(&status),
            };
        } // _handle 在这里离开作用域，触发 drop

        assert_eq!(*status.lock().unwrap(), "Closed test.txt");
    }

    #[test]
    fn test_multiple_drops() {
        let status1 = Arc::new(Mutex::new("Open".to_string()));
        let status2 = Arc::new(Mutex::new("Open".to_string()));

        {
            let _h1 = FileHandle {
                name: "a.txt".to_string(),
                status: status1.clone(),
            };
            let _h2 = FileHandle {
                name: "b.txt".to_string(),
                status: status2.clone(),
            };
        }

        assert_eq!(*status1.lock().unwrap(), "Closed a.txt");
        assert_eq!(*status2.lock().unwrap(), "Closed b.txt");
    }
}
