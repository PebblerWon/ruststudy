//! ## 练习 19: assert_approx_eq! 近似断言宏
//!
//! ### 学习目标
//! - 学会多分支匹配语法
//! - 理解宏卫生性
//! - 学会 `format!` 在宏中使用
//!
//! ### 背景
//!
//! 浮点数运算通常会有精度误差，直接使用 `assert_eq!` 往往会失败。
//! 我们需要一个能容忍微小误差的断言宏。这个宏需要支持两种调用方式：
//! 1. `assert_approx_eq!(a, b)`：使用默认误差 `1e-6`
//! 2. `assert_approx_eq!(a, b, eps)`：使用自定义误差
//!
//! ### 你的任务
//!
//! 1. 实现 `assert_approx_eq!` 宏，处理两种参数情况。
//! 2. 当断言失败时，打印出清晰的错误信息（包括左值、右值和误差）。
//! 3. 编写测试验证宏在相等和不等时的行为。

// ────────────── 实现区域 ──────────────

#[macro_export]
macro_rules! assert_approx_eq {
    // 情况 1: 两个参数，使用默认误差
    ($left:expr, $right:expr) => {{
        let eps = 1e-6;
        let a = $left - $right;
        if a < -eps || a > eps {
            // println!("left={},right={},eps={}", *left_val, *right_val, eps)
            panic!("left={},right={},eps={}", $left, $right, eps)
        }
    }};
    // 情况 2: 三个参数，使用指定误差
    ($left:expr, $right:expr, $eps:expr) => {
        match (&$left, &$right, &$eps) {
            (left_val, right_val, eps) => {
                let eps = *eps;
                let a = *left_val - *right_val;
                if a < -eps || a > eps {
                    // println!("left={},right={},eps={}", *left_val, *right_val, eps);
                    panic!("left={},right={},eps={}", *left_val, *right_val, eps);
                }
            }
        }
    };
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use crate::assert_approx_eq;

    #[test]
    fn test_approx_equal_default_eps() {
        let a = 1.0 / 3.0 * 3.0;
        assert_approx_eq!(a, 1.0);
    }

    #[test]
    fn test_approx_equal_custom_eps() {
        let a = 0.1 + 0.2;
        assert_approx_eq!(a, 0.3, 1e-10);
    }

    #[test]
    #[should_panic]
    fn test_approx_not_equal() {
        let a = 1.0;
        let b = 2.0;
        assert_approx_eq!(a, b);
    }
}
