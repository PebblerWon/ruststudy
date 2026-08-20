//! ## 练习 5: 用 Rc<T> 共享只读数据
//!
//! ### 学习目标
//! - 理解引用计数：多个所有者共享同一份数据
//! - 学会使用 `Rc::clone` 增加引用计数
//! - 理解为什么 `Rc` 不能用于多线程（见 Phase 3 的 `Arc`）
//! - 掌握 `strong_count` 查看当前引用数
//!
//! ### 背景
//!
//! 在 Rust 中，通常一个值只有一个所有者。但有时我们需要多个变量共享同一个数据。
//! `Rc<T>` (Reference Counted) 通过记录有多少个引用来实现共享所有权。
//! 当最后一个引用被丢弃时，数据才会被释放。
//!
//! ```rust,ignore
//! use std::rc::Rc;
//! let a = Rc::new(10);
//! let b = Rc::clone(&a); // 引用计数 +1
//! println!("count: {}", Rc::strong_count(&a)); // 输出 2
//! ```
//!
//! ### 你的任务
//!
//! 1. 创建一个图结构，其中多个节点可以共享同一个父节点。
//! 2. 实现函数统计某个节点被多少个其他节点引用。
//! 3. 验证引用计数的变化。

// ────────────── 实现区域 ──────────────

use std::rc::Rc;

/// 图节点，多个子节点可以共享同一个父节点
#[derive(Debug)]
pub struct GraphNode {
    pub value: i32,
    pub parent: Option<Rc<GraphNode>>,
}

impl GraphNode {
    /// 创建一个新节点
    pub fn new(value: i32) -> Rc<Self> {
        todo!("创建一个新的 Rc 包裹的 GraphNode")
    }

    /// 创建一个带有父节点的子节点
    pub fn with_parent(value: i32, parent: Rc<GraphNode>) -> Rc<Self> {
        todo!("创建一个带有父引用的节点")
    }
}

/// 返回某个节点当前的强引用计数
pub fn get_ref_count(node: &Rc<GraphNode>) -> usize {
    todo!("使用 Rc::strong_count 获取引用数")
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let node = GraphNode::new(10);
        assert_eq!(node.value, 10);
        assert!(node.parent.is_none());
    }

    #[test]
    fn test_shared_parent() {
        let parent = GraphNode::new(1);
        let child1 = GraphNode::with_parent(2, Rc::clone(&parent));
        let child2 = GraphNode::with_parent(3, Rc::clone(&parent));

        // 两个子节点都指向同一个父节点
        assert_eq!(child1.parent.as_ref().unwrap().value, 1);
        assert_eq!(child2.parent.as_ref().unwrap().value, 1);
        
        // 验证它们指向的是同一个对象（指针地址相同）
        assert!(Rc::ptr_eq(&child1.parent.as_ref().unwrap(), &child2.parent.as_ref().unwrap()));
    }

    #[test]
    fn test_ref_count_increases() {
        let node = GraphNode::new(42);
        assert_eq!(get_ref_count(&node), 1);

        let _alias = Rc::clone(&node);
        assert_eq!(get_ref_count(&node), 2);

        let _another = Rc::clone(&node);
        assert_eq!(get_ref_count(&node), 3);
    }

    #[test]
    fn test_ref_count_decreases_on_drop() {
        let node = GraphNode::new(99);
        {
            let _temp = Rc::clone(&node);
            assert_eq!(get_ref_count(&node), 2);
        } // _temp 在这里被丢弃
        
        assert_eq!(get_ref_count(&node), 1);
    }

    #[test]
    fn test_graph_structure() {
        // 构建一个简单的树形图：
        //       1
        //      / \
        //     2   3
        let root = GraphNode::new(1);
        let left = GraphNode::with_parent(2, Rc::clone(&root));
        let right = GraphNode::with_parent(3, Rc::clone(&root));

        assert_eq!(get_ref_count(&root), 3); // root, left.parent, right.parent
        assert_eq!(left.value, 2);
        assert_eq!(right.value, 3);
    }
}
