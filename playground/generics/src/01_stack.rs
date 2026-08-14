//! ## 练习 1: 泛型栈 Stack<T>
//!
//! ### 学习目标
//!
//! - 理解泛型参数 `<T>` 的含义：T 是一个类型参数，代表"任意类型"
//! - 学会在结构体和方法上声明泛型参数
//! - 理解 Rust 泛型是「编译期单态化」：每个具体类型会生成一份代码
//!
//! ### 概念讲解
//!
//! 泛型让你写一份代码适用于多种类型。对比：
//!
//! ```rust,ignore
//! // 不用泛型：每个类型写一遍
//! struct IntStack { items: Vec<i32> }
//! struct StringStack { items: Vec<String> }
//!
//! // 用泛型：一份代码搞定
//! struct Stack<T> { items: Vec<T> }
//! ```
//!
//! `<T>` 是类型参数声明，`T` 是惯例命名（Type 的首字母）。
//! 在 `impl` 块上也要声明 `<T>`，告诉编译器"这些方法也是泛型的"。
//!
//! ### 你的任务
//!
//! 实现下面的 `Stack<T>` 结构体，让所有测试通过。
//! 把每个 `todo!()` 替换为真正的实现代码。

// ────────────── 实现区域 ──────────────

/// 一个泛型栈结构，内部用 Vec<T> 存储。
///
/// 泛型参数 T 可以是任意类型：
/// - Stack<i32>     存整数
/// - Stack<String>  存字符串
/// - Stack<Vec<u8>> 存字节向量
pub struct Stack<T> {
    // 提示：内部用一个 Vec<T> 存储数据
    items: Vec<T>,
}

impl<T> Stack<T> {
    /// 创建一个空栈
    pub fn new() -> Self {
        Stack { items: Vec::new() }
    }

    /// 将元素压入栈顶
    ///
    /// 提示：Vec 的 push 方法会添加到末尾，末尾即为"栈顶"
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// 弹出栈顶元素，返回 Some(item)。
    /// 如果栈为空，返回 None。
    ///
    /// 提示：Vec 的 pop 方法返回 Option<T>，正好符合需求
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// 查看栈顶元素（不移除），返回 Some(&item)。
    /// 如果栈为空，返回 None。
    ///
    /// 提示：Vec 的 last 方法返回 Option<&T>
    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    /// 返回栈中元素数量
    pub fn len(&self) -> usize {
        // todo!("调用 self.items 的 len 方法")
        self.items.len()
    }

    /// 判断栈是否为空
    pub fn is_empty(&self) -> bool {
        // todo!("调用 self.items 的 is_empty 方法，或利用 len() 判断")
        self.items.is_empty()
    }
}

// ────────────── 测试区域 ──────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_is_empty() {
        let stack: Stack<i32> = Stack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_push_and_pop() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop(), Some(3)); // 后进先出
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None); // 空栈返回 None
    }

    #[test]
    fn test_peek_does_not_remove() {
        let mut stack = Stack::new();
        stack.push("hello");
        stack.push("world");

        // peek 只查看不移除
        assert_eq!(stack.peek(), Some(&"world"));
        assert_eq!(stack.len(), 2); // 数量没变

        // pop 才移除
        assert_eq!(stack.pop(), Some("world"));
        assert_eq!(stack.peek(), Some(&"hello"));
    }

    #[test]
    fn test_peek_empty_stack() {
        let stack: Stack<f64> = Stack::new();
        assert!(stack.peek().is_none());
    }

    #[test]
    fn test_generic_works_with_different_types() {
        // 泛型让同一个 Stack 适用于不同类型

        let mut int_stack = Stack::new();
        int_stack.push(42);
        assert_eq!(int_stack.pop(), Some(42));

        let mut string_stack = Stack::new();
        string_stack.push("hello".to_string());
        string_stack.push("world".to_string());
        assert_eq!(string_stack.pop(), Some("world".to_string()));

        let mut bool_stack = Stack::new();
        bool_stack.push(true);
        bool_stack.push(false);
        assert_eq!(bool_stack.pop(), Some(false));
    }

    #[test]
    fn test_stack_with_custom_type() {
        // 泛型也适用于自定义类型

        #[derive(Debug, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        let mut stack = Stack::new();
        stack.push(Point { x: 1, y: 2 });
        stack.push(Point { x: 3, y: 4 });

        assert_eq!(stack.pop(), Some(Point { x: 3, y: 4 }));
        assert_eq!(stack.peek(), Some(&Point { x: 1, y: 2 }));
    }
}
