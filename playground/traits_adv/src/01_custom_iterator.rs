//! ## 练习 14: 自定义迭代器
//!
//! ### 学习目标
//! - 为自定义类型实现 `Iterator` trait
//! - 理解关联类型 `type Item`
//! - 学会用 `.map().filter().sum()` 组合迭代器
//!
//! ### 背景
//!
//! Rust 的 `for` 循环和迭代器适配器（如 `map`, `filter`）都依赖于 `Iterator` trait。
//! 只要实现了这个 trait，你的类型就能无缝接入 Rust 强大的迭代器生态。
//!
//! ```rust,ignore
//! impl Iterator for MyType {
//!     type Item = u32;
//!     fn next(&mut self) -> Option<Self::Item> { /* ... */ }
//! }
//! ```
//!
//! ### 你的任务
//!
//! 1. 为 `Counter` 结构体实现 `Iterator`，使其能产生从 1 到 `max` 的数字序列。
//! 2. 编写测试验证迭代器的基本功能以及适配器组合使用。

// ────────────── 实现区域 ──────────────

/// 计数器，实现 Iterator 后可以用迭代器适配器
pub struct Counter {
    pub count: usize,
    pub max: usize,
}

impl Counter {
    pub fn new(max: usize) -> Self {
        Counter { count: 0, max }
    }
}

impl Iterator for Counter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // 提示：每次调用 count + 1，如果超过 max 则返回 None
        todo!("实现 next 逻辑")
    }
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_iterates_to_max() {
        let counter = Counter::new(5);
        let values: Vec<usize> = counter.collect();
        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_iterator_with_adapters() {
        let counter = Counter::new(10);
        
        // 求 1 到 10 中偶数的平方和
        let sum: usize = counter
            .filter(|&x| x % 2 == 0)
            .map(|x| x * x)
            .sum();
            
        assert_eq!(sum, 4 + 16 + 36 + 64 + 100);
    }

    #[test]
    fn test_empty_counter() {
        let counter = Counter::new(0);
        let values: Vec<usize> = counter.collect();
        assert!(values.is_empty());
    }
}
