//! ## 练习 3: 内部可变性 RefCell<T>
//!
//! ### 学习目标
//! - 理解「内部可变性」模式：在不可变引用下修改内部数据
//! - 学会使用 borrow() / borrow_mut()
//! - 理解运行时借用检查 vs 编译期借用检查
//! - 常见组合 Rc<RefCell<T>>
//!
//! > ⚠️ 待实现。

use std::cell::RefCell;
use std::rc::Rc;
