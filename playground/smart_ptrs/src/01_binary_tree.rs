//! ## 练习 1: 用 Box<T> 构建二叉树
//!
//! ### 学习目标
//! - 理解为什么递归类型需要 Box（编译期大小未知）
//! - 学会使用 Box 在堆上分配数据
//!
//! ### 背景
//!
//! 二叉树节点的左右子节点也是 TreeNode 类型：
//! ```rust,ignore
//! struct TreeNode {
//!     value: i32,
//!     left: Option<TreeNode>,   // ❌ 编译错误：无限大小的类型
//!     right: Option<TreeNode>,  // ❌ 编译错误：无限大小的类型
//! }
//! ```
//! Box 把数据放在堆上，本身只是一个固定大小的指针：
//! ```rust,ignore
//! struct TreeNode {
//!     value: i32,
//!     left: Option<Box<TreeNode>>,   // ✅ Box 是固定大小的指针
//!     right: Option<Box<TreeNode>>,
//! }
//! ```
//!
//! > ⚠️ 待实现：参照 Phase 1 的练习格式，补充实现区和测试区。

/// 二叉树节点
pub struct TreeNode {
    pub value: i32,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
}
