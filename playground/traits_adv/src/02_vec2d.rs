//! ## 练习 15: 运算符重载
//!
//! ### 学习目标
//! - 实现 `std::ops::Add`、`Sub`、`Mul` trait
//! - 理解运算符重载的本质是 trait 实现
//! - 掌握如何为自定义类型赋予数学运算能力
//!
//! ### 背景
//!
//! 在 Rust 中，`+`、`-`、`*` 等运算符并不是语言内置的魔法，而是通过 trait 实现的。
//! 例如，`a + b` 实际上是调用了 `std::ops::Add::add(a, b)`。
//!
//! ```rust,ignore
//! impl std::ops::Add for Vec2d {
//!     type Output = Vec2d;
//!     fn add(self, rhs: Self) -> Self::Output { /* ... */ }
//! }
//! ```
//!
//! ### 你的任务
//!
//! 1. 为 `Vec2d` 实现加法（对应分量相加）。
//! 2. 为 `Vec2d` 实现减法（对应分量相减）。
//! 3. 为 `Vec2d` 实现标量乘法（向量乘以 f64）。
//! 4. 编写测试验证运算逻辑。

// ────────────── 实现区域 ──────────────

use std::ops::{Add, Sub, Mul};

/// 二维向量
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}

impl Add for Vec2d {
    type Output = Vec2d;

    fn add(self, rhs: Self) -> Self::Output {
        todo!("实现向量加法")
    }
}

impl Sub for Vec2d {
    type Output = Vec2d;

    fn sub(self, rhs: Self) -> Self::Output {
        todo!("实现向量减法")
    }
}

impl Mul<f64> for Vec2d {
    type Output = Vec2d;

    fn mul(self, rhs: f64) -> Self::Output {
        todo!("实现标量乘法")
    }
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_addition() {
        let v1 = Vec2d { x: 1.0, y: 2.0 };
        let v2 = Vec2d { x: 3.0, y: 4.0 };
        let result = v1 + v2;
        assert_eq!(result, Vec2d { x: 4.0, y: 6.0 });
    }

    #[test]
    fn test_vector_subtraction() {
        let v1 = Vec2d { x: 5.0, y: 5.0 };
        let v2 = Vec2d { x: 2.0, y: 3.0 };
        let result = v1 - v2;
        assert_eq!(result, Vec2d { x: 3.0, y: 2.0 });
    }

    #[test]
    fn test_scalar_multiplication() {
        let v = Vec2d { x: 2.0, y: 3.0 };
        let result = v * 3.0;
        assert_eq!(result, Vec2d { x: 6.0, y: 9.0 });
    }

    #[test]
    fn test_chained_operations() {
        // (v1 + v2) * 2.0
        let v1 = Vec2d { x: 1.0, y: 1.0 };
        let v2 = Vec2d { x: 2.0, y: 2.0 };
        let result = (v1 + v2) * 2.0;
        assert_eq!(result, Vec2d { x: 6.0, y: 6.0 });
    }
}
