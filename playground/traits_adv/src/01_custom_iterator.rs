//! ## 练习 1: 自定义迭代器
//!
//! ### 学习目标
//! - 为自定义类型实现 Iterator trait
//! - 理解关联类型 `type Item`
//! - 学会用 .map().filter().sum() 组合迭代器
//!
//! > ⚠️ 待实现。

/// 计数器，实现 Iterator 后可以用迭代器适配器
pub struct Counter {
    pub count: usize,
    pub max: usize,
}
