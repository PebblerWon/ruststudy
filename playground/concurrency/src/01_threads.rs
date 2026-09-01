//! ## 练习 7: 用 std::thread 并行计算
//!
//! ### 学习目标
//! - 学会用 `thread::spawn` 创建新线程
//! - 理解 `move` 闭包：把变量所有权转移到线程
//! - 用 `JoinHandle::join` 等待线程结束并获取返回值
//!
//! ### 背景
//!
//! Rust 的线程模型非常轻量。通过 `std::thread::spawn`，你可以将任务分发到多个 CPU 核心上并行执行。
//! 关键在于 `move` 关键字，它确保线程捕获变量的所有权，从而避免数据竞争。
//!
//! ```rust,ignore
//! let handle = std::thread::spawn(move || {
//!     // 这里的代码在新线程中运行
//! });
//! handle.join().unwrap(); // 等待线程结束
//! ```
//!
//! ### 你的任务
//!
//! 1. 实现一个计算斐波那契数列的函数 `fib(n)`。
//! 2. 实现 `parallel_fib` 函数，开启两个线程分别计算 `fib(n-1)` 和 `fib(n-2)`，然后求和。
//! 3. 编写测试验证并行计算结果的正确性。

// ────────────── 实现区域 ──────────────

/// 计算第 n 个斐波那契数（递归版，用于演示）
pub fn fib(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }

    fib(n - 1) + fib(n - 2)
}

/// 使用多线程并行计算斐波那契数
///
/// 只在顶层开启两个线程分别计算 `fib(n-1)` 和 `fib(n-2)`，
/// 子线程内部使用串行 `fib`。
///
/// 注意：不要在递归的每一层都 `thread::spawn`——那样产生的线程数
/// 会以斐波那契速率爆炸增长（`parallel_fib(30)` 需要约 83 万个线程）。
pub fn parallel_fib(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }

    // move 闭包把 n 的所有权转移进线程；
    // join() 返回 Result<T, Box<dyn Any + Send>>，线程 panic 时为 Err
    let handle_left = std::thread::spawn(move || fib(n - 1));
    let handle_right = std::thread::spawn(move || fib(n - 2));
    let a = handle_left.join().expect("子线程 panic");
    let b = handle_right.join().expect("子线程 panic");
    a + b
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fib_basic() {
        assert_eq!(fib(0), 0);
        assert_eq!(fib(1), 1);
        assert_eq!(fib(5), 5);
        assert_eq!(fib(10), 55);
    }

    #[test]
    fn test_parallel_fib_correctness() {
        // 并行计算的结果应该和串行一致
        assert_eq!(parallel_fib(0), 0);
        assert_eq!(parallel_fib(1), 1);
        assert_eq!(parallel_fib(10), 55);
        assert_eq!(parallel_fib(20), 6765);
    }

    #[test]
    fn test_parallel_fib_matches_serial() {
        // 更大输入下仍与串行版本一致
        assert_eq!(parallel_fib(30), fib(30));
    }

    #[test]
    fn test_thread_join_returns_value() {
        // 验证 join() 能正确拿到线程的返回值
        let handle = std::thread::spawn(|| 42);
        let result = handle.join().unwrap();
        assert_eq!(result, 42);
    }
}
