//! # Phase 5: 高级 Trait 与运算符重载
//!
//! ## 练习列表
//!
//! | 序号 | 文件 | 练习 | 核心概念 |
//! |------|------|------|---------|
//! | 01 | `01_custom_iterator.rs` | 自定义迭代器 | Iterator trait、关联类型 type Item |
//! | 02 | `02_vec2d.rs` | 运算符重载 | std::ops::Add/Sub/Mul |
//! | 03 | `03_from_into.rs` | 手动实现 From/TryFrom | 类型转换 trait |
//! | 04 | `04_drop_trait.rs` | 资源释放 | Drop trait、RAII |
//!
//! > ⚠️ 本阶段尚未实现，等待 Phase 1-3 完成后填充。

// #[path] 属性：文件名以数字开头（排序用），但 Rust 模块名必须以字母开头
#[path = "01_custom_iterator.rs"]
pub mod custom_iterator_01;

#[path = "02_vec2d.rs"]
pub mod vec2d_02;

#[path = "03_from_into.rs"]
pub mod from_into_03;

#[path = "04_drop_trait.rs"]
pub mod drop_trait_04;
