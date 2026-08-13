//! ## 练习 2: 用 Rc<T> 共享只读数据
//!
//! ### 学习目标
//! - 理解引用计数：多个所有者共享同一份数据
//! - 学会使用 Rc::clone 和 strong_count
//! - 理解为什么 Rc 不能用于多线程（见 Phase 3 的 Arc）
//!
//! > ⚠️ 待实现。

use std::rc::Rc;

/// 图节点，多个子节点可以共享同一个父节点
pub struct GraphNode {
    pub value: i32,
    pub parent: Option<Rc<GraphNode>>,
}
