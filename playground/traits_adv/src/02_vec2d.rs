//! ## 练习 2: 运算符重载
//!
//! ### 学习目标
//! - 实现 std::ops::Add、Sub、Mul trait
//! - 理解运算符重载的本质是 trait 实现
//!
//! > ⚠️ 待实现。

/// 二维向量
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}
