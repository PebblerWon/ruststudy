# 第 1 章：所有权与移动语义

## 本章目标

- 理解 Rust 为什么没有 GC、也没有手动 `free`
- 掌握所有权的三条规则
- 区分"移动（move）"与"拷贝（Copy）"
- 知道函数传参何时会发生所有权转移
- 学会用 `Clone` 显式复制堆数据

## 1.1 为什么需要所有权

> 📖 对照：在 TaskFlow 里你写过 `tasks.push(task.clone())`，也写过 `self.store.save(&tasks)`。
> 你大概隐约感觉到"加 `&` 就不会拿走所有权"，但没深究。本章把这件事讲透。

C/C++ 里你 `malloc` 一块内存就要负责 `free`，忘了就内存泄漏，多 `free` 就崩；
Java/Python 用 GC 帮你回收，代价是运行时停顿和不可预测的延迟。

Rust 的第三条路：**编译期通过"所有权"规则静态决定每块堆内存何时释放**，
既没有运行时开销，也不会泄漏（除 `Rc` 循环引用等特殊情况）。

## 1.2 所有权三规则

1. **每个值在任一时刻有且仅有一个"所有者"变量。**
2. **当所有者离开作用域，值被自动 drop（释放）。**
3. **把值赋给另一个变量、或传给函数、或从函数返回时，所有权"移动"（move）
   到新变量——除非该类型实现了 `Copy` trait。**

规则 2 解释了 TaskFlow 里你从没写过 `free(tasks)`：`tasks: Vec<Task>` 在
`run()` 结束时自动释放。

## 1.3 移动（Move）：堆数据的所有权转移

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // ← s1 的所有权移动到 s2

    // println!("{s1}"); // ✗ 编译错误：s1 已被移动，不能再使用
    println!("{s2}");    // ✓
}
```

`String` 在栈上存的是 `(指针, 长度, 容量)`，真正的字符数据在堆上。
`let s2 = s1;` 不是拷贝堆，而是把栈上那三件套复制一份给 `s2`，并**让 `s1` 失效**。
这样 `s1` 和 `s2` 不会同时指向同一块堆，避免"双重释放"。

> 💡 如果 Rust 不让 `s1` 失效，那么 `s1` 和 `s2` 离开作用域时都会尝试释放同一块堆，就崩了。

### 函数传参也是 move

```rust
fn take(s: String) {
    println!("拿到了 {s}");
}

fn main() {
    let s = String::from("hello");
    take(s);            // ← 所有权交给了 take 的参数
    // println!("{s}"); // ✗ s 已被移动
}
```

这就是为什么 TaskFlow 里 `service.add_task(...)` 之后你想再用某变量会报错——
那些函数签名拿的是 `&str`（借用），就是为了避免把所有权拿走。

## 1.4 Copy：栈数据的"自动复制"

整型、浮点、布尔、字符，以及由它们组成的元组/数组，**默认实现 `Copy`**。
赋值或传参时不移动，而是按位复制：

```rust
fn main() {
    let x = 5;
    let y = x;      // i32 实现了 Copy，x 仍然可用
    println!("{x} {y}"); // ✓ 两个都能用
}
```

什么类型能 `Copy`？**完全在栈上、没有堆资源、且所有字段都 `Copy`** 的类型。
所以 `String`（有堆）不能 `Copy`，`Vec<T>` 不能 `Copy`，
而 `(i32, f64)` 可以，`(i32, String)` 不可以。

## 1.5 Clone：显式深拷贝

当你**真的**需要一份独立的堆拷贝时，用 `.clone()`：

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // 深拷贝堆数据
    println!("{s1} {s2}"); // ✓ 两个都可用
}
```

> 📖 对照：`task.clone()` 在 `service.add_task` 里出现，因为你要把 task
> 既存进 `tasks` 列表，又作为函数返回值返回，必须复制一份。

**别滥用 `clone()`**。新手习惯性到处 `clone` 会让代码慢且啰嗦。能用借用就用借用，
等第 2 章学完借用，你会发现很多 `clone` 其实是不必要的。

## 1.6 所有权与函数返回

把值作为返回值返回，就把所有权交还给调用者：

```rust
fn make_string() -> String {
    String::from("made") // 所有权移动到调用者
}

fn main() {
    let s = make_string(); // ✓ s 接管所有权
    println!("{s}");
}
```

但下面这种"借进来又还出去"的写法很低效（凭空多一次 move），第 2 章会教你用引用避免：

```rust
fn echo(s: String) -> String { s } // 没必要，调用方传引用即可
```

## 1.7 元组的所有权

元组按字段各自决定 move 还是 copy：

```rust
fn main() {
    let t = (String::from("a"), 5);
    let (s, n) = t;        // 解构：String 移动，i32 复制
    // println!("{:?}", t); // ✗ t.0 已被移动，整个 t 不可用
    println!("{s} {n}");
}
```

## 1.8 部分移动（Partial Move）

解构结构体时，可以只搬走部分字段，剩下的字段就不能再整体使用了：

```rust
struct User { name: String, age: u32 }

fn main() {
    let u = User { name: "Ann".into(), age: 30 };
    let name = u.name;   // 搬走 name
    // let u2 = u;       // ✗ u 已部分移动
    println!("{}", u.age); // ✓ 未移动的字段还能用
}
```

## 1.9 常见陷阱

### 陷阱 1：以为 `let s2 = s1;` 是拷贝

```rust
let s1 = String::from("hi");
let s2 = s1;
println!("{s1}"); // ✗ borrow of moved value: `s1`
```

编译器会提示你"consider cloning"。**String/Vec/HashMap 等堆类型赋值即 move**。

### 陷阱 2：在循环里反复拿所有权

```rust
// ✗ 每次都 move，第一次循环后 list 就废了
fn bad(list: Vec<i32>) {
    for _ in 0..3 {
        process(list); // 第二次循环报错
    }
}

// ✓ 借用（第 2 章详解）
fn good(list: &Vec<i32>) {
    for _ in 0..3 {
        process_ref(list);
    }
}
```

### 陷阱 3：把 `Copy` 类型放进 `Box` 后就不再 Copy

```rust
let b1 = Box::new(5);
let b2 = b1;            // Box 没实现 Copy，b1 被 move
// println!("{b1}");    // ✗
```

`Box<T>` 拥有堆上 `T`，所以 `Box` 自己不是 `Copy`（哪怕 `T` 是）。

## 1.10 练习

1. 下面代码能编译吗？为什么？怎么改最小改动让它编译通过？
   ```rust
   fn main() {
       let s = String::from("data");
       let v = vec![s];
       println!("{s}");
   }
   ```

2. 写一个函数 `longest(a: String, b: String) -> String`，返回长度更大的那个。
   思考：参数和返回值的所有权分别怎么流动？调用方传进去之后还能用原变量吗？

3. 解释为什么 `let n = 5; let m = n;` 之后 `n` 还能用，而 `String` 不行。

4. 用 `cargo new` 的 playground 验证：把一个 `Vec<String>` 赋值给另一个变量后，
   原变量是否可用？再试 `Vec<i32>` 呢？结果一样吗，为什么？

## 1.11 小结

| 概念 | 一句话 |
|------|--------|
| 所有权 | 每个值有唯一所有者，离开作用域自动 drop |
| Move | 堆类型赋值/传参时所有权转移，原变量失效 |
| Copy | 栈类型自动按位复制，原变量仍可用 |
| Clone | 显式深拷贝，慎用 |

> 下一章我们学**借用**——在不拿走所有权的前提下使用一个值。这是 TaskFlow 里
> `&str`、`&Vec<Task>`、`&self` 背后的真相。

---

[← 概览](./00_overview.md) | [下一章 →](./02_borrowing_slices.md)

---

📧 联系作者：pebblerwon@qq.com
