//! # Phase 6: 声明式宏 macro_rules!
//!
//! ## 练习列表
//!
//! | 序号 | 文件 | 练习 | 核心概念 |
//! |------|------|------|---------|
//! | 01 | `01_vec_of_strings.rs` | 字符串向量宏 | 基础宏语法、`$x:expr` |
//! | 02 | `02_assert_approx_eq.rs` | 近似断言宏 | 多分支匹配、`$left`/`$right` |
//!
//! > ⚠️ 本阶段尚未实现，等待 Phase 1-5 完成后填充。

// #[path] 属性：文件名以数字开头（排序用），但 Rust 模块名必须以字母开头
#[path = "01_vec_of_strings.rs"]
pub mod vec_of_strings_01;

#[path = "02_assert_approx_eq.rs"]
pub mod assert_approx_eq_02;
