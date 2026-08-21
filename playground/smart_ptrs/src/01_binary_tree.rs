//! ## 练习 4: 用 Box<T> 构建二叉树
//!
//! ### 学习目标
//! - 理解为什么递归类型需要 Box（编译期大小未知）
//! - 学会使用 Box 在堆上分配数据
//! - 掌握递归数据结构的基本操作（插入、遍历）
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
//! ### 你的任务
//!
//! 1. 为 `TreeNode` 实现 `insert` 方法，按二叉搜索树（BST）规则插入值。
//! 2. 实现 `contains` 方法，判断树中是否包含某个值。
//! 3. 实现 `in_order` 方法，返回中序遍历结果（升序）。

// ────────────── 实现区域 ──────────────

/// 二叉树节点
#[derive(Debug)]
pub struct TreeNode {
    pub value: i32,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
}

impl TreeNode {
    /// 创建一个新节点
    pub fn new(value: i32) -> Self {
        // todo!("实现创建新节点的逻辑");
        TreeNode {
            value,
            left: None,
            right: None,
        }
    }

    /// 插入一个值到二叉搜索树中
    ///
    /// 规则：
    /// - 如果值小于当前节点，往左子树插
    /// - 如果值大于当前节点，往右子树插
    /// - 如果相等，忽略（不重复插入）
    pub fn insert(&mut self, value: i32) {
        if self.value == value {
            return;
        }
        let mut target = &mut self.right;
        if value < self.value {
            target = &mut self.left;
        }
        match target {
            None => {
                *target = Some(Box::new(TreeNode::new(value)));
            }
            Some(n) => {
                n.insert(value);
            }
        }
    }

    /// 判断树中是否包含某个值
    pub fn contains(&self, value: i32) -> bool {
        if self.value == value {
            return true;
        }
        let target = if value < self.value {
            &self.left
        } else {
            &self.right
        };
        match target {
            None => false,
            Some(n) => n.contains(value),
        }
    }

    /// 中序遍历，返回排序后的向量
    pub fn in_order(&self) -> Vec<i32> {
        // todo!("实现中序遍历")
        let mut nodes = vec![self];
        let mut res: Vec<i32> = vec![];

        while !nodes.is_empty() {
            let target = nodes.last().unwrap();

            let left = &target.left;

            if let Some(n) = left {
                nodes.push(n);
            } else {
                loop {
                    let n = nodes.pop();
                    if let Some(node) = n {
                        res.push(node.value);
                        if let Some(r) = &node.right {
                            nodes.push(r);
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        res
    }
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let node = TreeNode::new(10);
        assert_eq!(node.value, 10);
        assert!(node.left.is_none());
        assert!(node.right.is_none());
    }

    #[test]
    fn test_insert_left_and_right() {
        let mut root = TreeNode::new(10);
        root.insert(5);
        root.insert(15);

        assert!(root.left.is_some());
        assert!(root.right.is_some());
        assert_eq!(root.left.as_ref().unwrap().value, 5);
        assert_eq!(root.right.as_ref().unwrap().value, 15);
    }

    #[test]
    fn test_contains_value() {
        let mut root = TreeNode::new(10);
        root.insert(5);
        root.insert(15);
        root.insert(3);

        assert!(root.contains(10));
        assert!(root.contains(5));
        assert!(root.contains(15));
        assert!(root.contains(3));
        assert!(!root.contains(99));
    }

    #[test]
    fn test_in_order_traversal() {
        let mut root = TreeNode::new(5);
        root.insert(3);
        root.insert(7);
        root.insert(1);
        root.insert(4);

        let sorted = root.in_order();
        assert_eq!(sorted, vec![1, 3, 4, 5, 7]);
    }

    #[test]
    fn test_duplicate_insert_ignored() {
        let mut root = TreeNode::new(10);
        root.insert(10);
        root.insert(10);

        // 应该只有一个节点
        assert!(root.left.is_none());
        assert!(root.right.is_none());
    }

    #[test]
    fn test_deep_tree() {
        let mut root = TreeNode::new(5);
        for i in 1..=10 {
            root.insert(i);
        }

        assert_eq!(root.in_order(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert!(root.contains(1));
        assert!(root.contains(10));
    }
}
