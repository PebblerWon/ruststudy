//! ## 练习 6: 内部可变性 RefCell<T>
//!
//! ### 学习目标
//! - 理解「内部可变性」模式：在不可变引用下修改内部数据
//! - 学会使用 `borrow()` / `borrow_mut()`
//! - 理解运行时借用检查 vs 编译期借用检查
//! - 常见组合 `Rc<RefCell<T>>`
//!
//! ### 背景
//!
//! Rust 的借用规则通常要求：要么有多个不可变引用，要么有一个可变引用。
//! 但有时我们需要在拥有不可变引用的同时修改某些内部状态（比如缓存、计数器）。
//! `RefCell<T>` 将借用检查从编译期推迟到运行期。
//!
//! ```rust,ignore
//! use std::cell::RefCell;
//! let c = RefCell::new(5);
//! *c.borrow_mut() += 1; // 即使 c 是不可变的，也能修改内部值
//! ```
//!
//! > ⚠️ 注意：如果违反借用规则（例如同时调用 borrow 和 borrow_mut），程序会在运行期 panic。
//!
//! ### 你的任务
//!
//! 1. 实现一个 `Counter` 结构体，内部使用 `RefCell` 存储计数。
//! 2. 实现 `increment` 方法，即使在 `&self` 下也能增加计数。
//! 3. 实现 `get_count` 方法返回当前计数。
//! 4. 编写测试验证多个 `Rc` 共享同一个 `RefCell` 时的行为。

// ────────────── 实现区域 ──────────────

use std::cell::RefCell;
use std::rc::Rc;

/// 一个简单的计数器，演示内部可变性
pub struct Counter {
    count: RefCell<u32>,
}

impl Counter {
    /// 创建一个初始值为 0 的计数器
    pub fn new() -> Self {
        // todo!("初始化 RefCell")
        Counter {
            count: RefCell::new(0),
        }
    }

    /// 增加计数
    ///
    /// 注意：这个方法只接受 &self，但依然能修改内部状态
    pub fn increment(&self) {
        // todo!("使用 borrow_mut 修改内部值")
        *self.count.borrow_mut() += 1;
    }
    /// 获取当前计数
    pub fn get_count(&self) -> u32 {
        *self.count.borrow()
    }
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_increment() {
        let counter = Counter::new();
        assert_eq!(counter.get_count(), 0);

        counter.increment();
        assert_eq!(counter.get_count(), 1);

        counter.increment();
        assert_eq!(counter.get_count(), 2);
    }

    #[test]
    fn test_shared_counter_with_rc() {
        // 多个所有者共享同一个计数器
        let counter = Rc::new(Counter::new());
        let alias = Rc::clone(&counter);

        counter.increment();
        assert_eq!(alias.get_count(), 1); // 通过别名也能看到变化

        alias.increment();
        assert_eq!(counter.get_count(), 2);
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn test_borrow_panic_on_conflict() {
        let counter = Counter::new();
        let _borrowed = counter.count.borrow(); // 不可变借用

        // 此时再尝试可变借用会 panic
        counter.increment();
    }
}
