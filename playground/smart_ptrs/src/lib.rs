//! # Phase 2: 智能指针与内部可变性
//!
//! ## 练习列表
//!
//! | 序号 | 文件 | 练习 | 核心概念 |
//! |------|------|------|---------|
//! | 01 | `01_binary_tree.rs` | 用 Box 构建二叉树 | Box<T>、递归类型 |
//! | 02 | `02_graph.rs` | 用 Rc 共享只读数据 | Rc<T>、引用计数 |
//! | 03 | `03_refcell_demo.rs` | 内部可变性 | RefCell<T>、运行时借用检查 |
//!
//! > ⚠️ 本阶段尚未实现，等待 Phase 1 完成后填充。

// #[path] 属性：文件名以数字开头（排序用），但 Rust 模块名必须以字母开头
#[path = "01_binary_tree.rs"]
pub mod binary_tree_01;

#[path = "02_graph.rs"]
pub mod graph_02;

#[path = "03_refcell_demo.rs"]
pub mod refcell_demo_03;
