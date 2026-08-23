# 第 8 章：智能指针与内部可变性

## 本章目标

- 理解"智能指针"与普通引用的区别
- 掌握 `Box<T>`：堆分配、递归类型
- 掌握 `Rc<T>` / `Arc<T>`：共享所有权
- 掌握 `RefCell<T>` / `Cell<T>`：内部可变性
- 理解 `Drop` trait 与析构顺序
- 解释 TaskFlow 为何完全没用智能指针

## 8.1 什么是智能指针

普通引用 `&T` 只是"借一下"，不拥有数据，不负责释放。
**智能指针**拥有数据，离开作用域时自动释放（通过 `Drop` trait）。

| 类型 | 拥有? | 何时用 |
|------|------|------|
| `&T` / `&mut T` | ✗ | 单一所有者借用 |
| `Box<T>` | ✓ 单一 | 把数据搬到堆；递归类型；trait 对象 |
| `Rc<T>` | ✓ 共享（单线程） | 多处共享同一份只读数据 |
| `Arc<T>` | ✓ 共享（多线程） | 同上，线程安全 |
| `RefCell<T>` | ✓ 单一 | 运行期检查的内部可变性 |
| `Mutex<T>` / `RwLock<T>` | ✓ | 多线程内部可变性 |
| `Vec<T>` / `String` / `HashMap` | ✓ | 其实也是智能指针（拥有堆缓冲） |

> 是的，`Vec` 和 `String` 本质上也是智能指针——它们 own 堆缓冲并在 drop 时释放。

## 8.2 `Box<T>`：最简单的堆分配

```rust
let b = Box::new(5);       // 5 被搬到堆
println!("{b}");
// b 离开作用域时，先 drop 堆上的 5，再 drop 栈上的 Box
```

### 用途 1：把大对象搬到堆，避免栈拷贝

```rust
let huge = Box::new([0u64; 1_000_000]); // 8MB，放堆
```

### 用途 2：递归类型

```rust
enum List {
    Cons(i32, Box<List>), // 不加 Box 大小无限，编译器无法定大小
    Nil,
}

use List::*;
let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
```

> 类似 C 的链表节点 `struct Node { val: i32, next: Box<Node> }`。

### 用途 3：trait 对象

```rust
let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Circle{r:1.0}), Box::new(Square{s:2.0})];
```

## 8.3 `Rc<T>`：引用计数（单线程共享）

```rust
use std::rc::Rc;

let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a); // 引用计数 +1，不复制字符串
let c = Rc::clone(&a); // 引用计数 = 3
println!("count = {}", Rc::strong_count(&a)); // 3

drop(c); // 计数 -1 = 2
// 当计数归 0，字符串才被释放
```

`Rc::clone` **不拷贝数据**，只增加计数。多个所有者共享一份只读数据。

### 适用场景

- 多个子节点共享同一个父节点（如 DOM 树）
- 图结构
- 缓存

### `Rc` 是只读的

`Rc<T>` 内部的 `T` 不能直接 `&mut`。要改共享数据，配 `RefCell`：

```rust
let shared = Rc::new(RefCell::new(vec![1, 2, 3]));
let s2 = Rc::clone(&shared);
s2.borrow_mut().push(4); // 运行期借用检查
```

### 循环引用泄漏

`Rc` 会循环引用导致泄漏：

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node { next: RefCell<Option<Rc<Node>>> }
// A → B → A → B ... 计数永不归 0，泄漏
```

用 `Weak<T>` 打破循环（弱引用不增加 strong count）。`Rc::downgrade` 创建 `Weak`，
`weak.upgrade()` 返回 `Option<Rc<T>>`（可能已被释放）。

## 8.4 `Arc<T>`：原子引用计数（多线程）

`Arc` = Atomic Rc。API 与 `Rc` 几乎一样，但 `clone` 用原子操作，线程安全。

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3]);
let handles: Vec<_> = (0..3).map(|_| {
    let d = Arc::clone(&data);
    thread::spawn(move || println!("{:?}", d))
}).collect();
for h in handles { h.join().unwrap(); }
```

> 第 9 章并发会再详谈。规则：**单线程用 `Rc`，多线程用 `Arc`**。
> 错用 `Rc` 跨线程会编译错误（`Rc` 非 `Send`/`Sync`）。

## 8.5 内部可变性

借用规则在**编译期**检查：同一时刻只能"多读"或"一写"。
但有时我们想：表面 `&T`，实际能改内部——比如 `Rc<T>` 共享的数据要修改。

`RefCell<T>` / `Cell<T>` 把借用检查**推迟到运行期**：

### `Cell<T>`：Copy 类型的内部可变

```rust
use std::cell::Cell;

let c = Cell::new(5);
let r1 = &c;
let r2 = &c;
r1.set(10); // 即使有多个 &Cell 也能改
println!("{}", r2.get()); // 10
```

`Cell` 通过"复制进出"工作，只能用于 `T: Copy`，没有 `&mut T`。

### `RefCell<T>`：运行期借用检查

```rust
use std::cell::RefCell;

let cell = RefCell::new(vec![1, 2, 3]);
{
    let mut borrow = cell.borrow_mut(); // 运行期可变借用
    borrow.push(4);
} // 借用在这里结束
println!("{:?}", cell.borrow()); // [1,2,3,4]
```

`borrow()` 返回 `Ref<T>`，`borrow_mut()` 返回 `RefMut<T>`。
运行期跟踪借用计数，若违反"多读 or 一写"则 **panic**：

```rust
let b1 = cell.borrow_mut();
let b2 = cell.borrow_mut(); // panic: already mutably borrowed
```

### `RefCell` 适用场景

- `Rc<RefCell<T>>`：单线程共享可变数据（最常见组合）
- mock 测试中替换实现
- 实现观察者模式、回调注册

> ⚠️ 运行期 panic 比编译期错误糟糕得多——只在确实需要时用 `RefCell`。
> 编译期借用检查能解决的问题别用 `RefCell` 绕过。

## 8.6 `Drop` trait：自定义析构

```rust
struct Custom { name: &'static str }
impl Drop for Custom {
    fn drop(&mut self) {
        println!("dropping {}", self.name);
    }
}

fn main() {
    let a = Custom { name: "A" };
    let b = Custom { name: "B" };
    // 作用域结束时按声明逆序 drop：B 先 drop，A 后 drop
}
```

### 何时手写 `Drop`

- 释放非 Rust 资源：文件句柄、锁、C 指针、套接字
- 打印日志
- 维护不变量（如回滚未完成的事务）

### 提前 drop：`std::mem::drop`

```rust
let big = Box::new([0u8; 1_000_000]);
// 用完了，提前释放，不等作用域结束
drop(big);
```

### `Drop` 与 `Copy` 互斥

实现 `Copy` 的类型不能有 `Drop`——Copy 意味着"按位复制就够了"，没"析构"概念。

## 8.7 组合使用：`Rc<RefCell<T>>` 经典模式

实现一个双向链表节点：

```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Node {
    value: i32,
    next: RefCell<Option<Rc<Node>>>,
}

fn main() {
    let a = Rc::new(Node { value: 1, next: RefCell::new(None) });
    let b = Rc::new(Node { value: 2, next: RefCell::new(None) });

    *a.next.borrow_mut() = Some(Rc::clone(&b));
    println!("{:?}", a);
}
```

> Rust 写链表比 C++ 难，正是因为所有权约束。生产里通常用 `Vec<Enum>`、
> `indexmap` + 索引代替指针。要真正写复杂链表推荐阅读
> [Learn Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)。

## 8.8 智能指针选型决策树

```
要在线程间共享吗？
├─ 是 → Arc<T>
│        需要修改吗？
│        ├─ 是 → Arc<Mutex<T>> 或 Arc<RwLock<T>>  （第 9 章）
│        └─ 否 → Arc<T>
└─ 否 → 单所有者？
         ├─ 是 → Box<T>（堆）/ 直接 T（栈）
         └─ 否 → Rc<T>
                  需要修改吗？
                  ├─ 是 → Rc<RefCell<T>>
                  └─ 否 → Rc<T>
```

## 8.9 `Deref` 与强制解引用

智能指针都实现 `Deref`，让 `&Box<T>` 能当 `&T` 用：

```rust
let b = Box::new(String::from("hi"));
// b 是 Box<String>，但 println! 需要 &str
println!("{}", b); // Box → &String → &str，自动 deref
```

`DerefMut` 是可变版本。这就是为什么 `&Vec` 能传给 `&[T]`、`&String` 能传给 `&str`。

> 📖 对照：TaskFlow 里 `service.store.load()` 返回 `Vec<Task>`，
> `self.store.save(&tasks)` 传 `&Vec<Task>` 但参数是 `&[Task]`——靠 `Deref`。

## 8.10 常见陷阱

### 陷阱 1：用 `Rc` 跨线程

```rust
use std::thread;
let r = Rc::new(5);
thread::spawn(move || println!("{r}")); // ✗ Rc 未实现 Send
```

换 `Arc`。

### 陷阱 2：`RefCell` 运行时 panic

```rust
let cell = RefCell::new(0);
let b = cell.borrow(); // 不可变借用
cell.borrow_mut();     // panic! 已有不可变借用
```

调试困难（运行期才暴露）。务必保证借用生命周期短。

### 陷阱 3：循环引用泄漏

`Rc` 环会导致内存永不释放。用 `Weak` 打破。检测：用 `Rc::strong_count` /
`Weak::strong_count` 跟踪。Valgrind 风格工具：`leak.rs` 等。

### 陷阱 4：以为 `Box` 是引用

```rust
let b = Box::new(5);
let r: &i32 = &b;        // ✓ &Box<i32> deref 到 &i32
let owned: i32 = *b;     // 拷贝出来
```

`Box` 是拥有型，不是借用——传 `Box` 给函数会 move 所有权。

## 8.11 练习

1. 用 `Box` 定义一个二叉树类型 `Tree`，并实现 `fn insert(&mut self, v: i32)`。

2. 用 `Rc<RefCell<Node>>` 实现一个单向链表，支持 `push_front` 和 `pop_front`。

3. 写一段代码：用 `Arc<Mutex<Vec<i32>>>` 在 4 个线程中各 push 100 个数，
   最后统计总数。提示：参考第 9 章。

4. 解释：为什么 TaskFlow 项目里**完全没用** `Rc` / `RefCell` / `Box`？
   提示：单一所有者 + 编译期借用检查就够了。

## 8.12 小结

| 概念 | 一句话 |
|------|--------|
| `Box<T>` | 堆分配，单一所有者，递归类型必备 |
| `Rc<T>` | 单线程共享，引用计数 |
| `Arc<T>` | 多线程共享，原子引用计数 |
| `RefCell<T>` | 运行期借用检查，内部可变性（单线程） |
| `Cell<T>` | Copy 类型的内部可变性 |
| `Drop` | 自定义析构，逆序调用 |
| `Rc<RefCell<T>>` | 单线程共享可变经典组合 |

> 下一章我们用这些积木去搭**并发与异步**——多线程、`Mutex`、`Send`/`Sync`、
> `async/await` 与 `tokio`。

---

[← 第 7 章](./07_closures_iterators.md) | [下一章 →](./09_concurrency_async.md)

---

📧 联系作者：pebblerwon@qq.com
