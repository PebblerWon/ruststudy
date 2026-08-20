//! ## 练习 18: vec_of_strings! 宏
//!
//! ### 学习目标
//! - 理解 `macro_rules!` 基础语法
//! - 学会 `$x:expr` 片段说明符
//! - 理解重复模式 `$($x:expr),*`
//!
//! ### 背景
//!
//! Rust 的声明式宏允许你编写像函数一样的代码，但它在编译期展开。
//! 通过匹配不同的语法片段，你可以生成复杂的代码结构。
//!
//! ```rust,ignore
//! macro_rules! my_macro {
//!     ($($x:expr),*) => { ... }
//! }
//! ```
//!
//! ### 你的任务
//!
//! 1. 实现 `vec_of_strings!` 宏，它接受任意数量的字符串字面量或表达式。
//! 2. 宏应该返回一个 `Vec<String>`，其中每个元素都是输入字符串的 `.to_string()` 版本。
//! 3. 编写测试验证宏生成的向量与手动创建的向量一致。

// ────────────── 实现区域 ──────────────

#[macro_export]
macro_rules! vec_of_strings {
    // 提示：使用重复模式捕获多个表达式，并在展开时对每个表达式调用 .to_string()
    ($($x:expr),*) => {
        todo!("实现宏逻辑")
    };
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use crate::vec_of_strings;

    #[test]
    fn test_empty_macro() {
        let v = vec_of_strings!();
        assert_eq!(v, Vec::<String>::new());
    }

    #[test]
    fn test_single_string() {
        let v = vec_of_strings!("hello");
        assert_eq!(v, vec!["hello".to_string()]);
    }

    #[test]
    fn test_multiple_strings() {
        let v = vec_of_strings!("a", "b", "c");
        assert_eq!(v, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    }

    #[test]
    fn test_with_expressions() {
        let s = "world";
        let v = vec_of_strings!("hello", s);
        assert_eq!(v, vec!["hello".to_string(), "world".to_string()]);
    }
}
