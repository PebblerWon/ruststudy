//! ## 练习 1: vec_of_strings! 宏
//!
//! ### 学习目标
//! - 理解 macro_rules! 基础语法
//! - 学会 `$x:expr` 片段说明符
//! - 理解重复模式 `$($x:expr),*`
//!
//! ### 目标
//!
//! ```rust,ignore
//! vec_of_strings!("a", "b", "c")
//! // 等价于
//! vec!["a".to_string(), "b".to_string(), "c".to_string()]
//! ```
//!
//! > ⚠️ 待实现。
