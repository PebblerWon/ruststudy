# 第 6 章：Trait 进阶

## 本章目标

- 复习并扩展 trait：默认方法、关联类型、关联常量、超 trait
- 理解 trait 对象 `dyn Trait` 与动态分发
- 区分"静态分发（泛型）"与"动态分发（dyn）"
- 掌握对象安全（object safety）的判定
- 解释 TaskFlow 里 `Store` trait 为何未用作 `dyn`

## 6.1 trait 回顾

> 📖 对照：TaskFlow 的 `pub trait Store { fn load(&self) -> Result<Vec<Task>>; ... }`
> 是最朴素的 trait——一组方法签名。

```rust
trait Greet {
    fn say_hi(&self) -> String;
}

struct Dog;
impl Greet for Dog {
    fn say_hi(&self) -> String { "汪!".into() }
}
```

## 6.2 默认方法

trait 方法可以有默认实现，实现者可选覆盖：

```rust
trait Logger {
    fn log(&self, msg: &str); // 必须实现

    fn warn(&self, msg: &str) { // 默认实现
        self.log(&format!("[WARN] {msg}"));
    }
    fn error(&self, msg: &str) {
        self.log(&format!("[ERROR] {msg}"));
    }
}

struct PrintLogger;
impl Logger for PrintLogger {
    fn log(&self, msg: &str) { println!("{msg}"); }
    // warn / error 自动有
}
```

> 📖 对照：标准库的 `Iterator` trait 只有 `next` 必须实现，`map`/`filter`/`collect`
> 等几十个方法全是基于 `next` 的默认实现——下一章详谈。

## 6.3 关联类型（Associated Type）

```rust
trait Iterator {
    type Item; // 关联类型，由实现者指定
    fn next(&mut self) -> Option<Self::Item>;
}

struct Counter { count: u32 }
impl Iterator for Counter {
    type Item = u32; // ← 这里指定
    fn next(&mut self) -> Option<u32> {
        self.count += 1;
        if self.count <= 5 { Some(self.count) } else { None }
    }
}
```

关联类型 vs 泛型参数：

```rust
// 关联类型：每个实现者只能有一种 Item
trait Iterator { type Item; ... }

// 泛型：可以让 Counter 既是 Iterator<u32> 又是 Iterator<u64>（标准库没用这种）
trait IteratorBad<T> { fn next(&mut self) -> Option<T>; }
```

关联类型表达"**这种迭代器只有一种产出类型**"，更准确。`FromIterator`、`Deref`、
`Add` 等都有关联类型。

## 6.4 关联常量

```rust
trait Pi {
    const PI: f64;
}
struct Math;
impl Pi for Math {
    const PI: f64 = 3.141592653589793;
}
```

## 6.5 超 trait（Supertrait）

trait 可以要求实现者**同时**实现另一个 trait：

```rust
trait Named: std::fmt::Display { // 实现 Named 必须先实现 Display
    fn name(&self) -> String;
}

struct User { login: String }
impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.login)
    }
}
impl Named for User {
    fn name(&self) -> String { self.login.clone() }
}
```

`Named: Display` 意味着 `Named` 的方法体内可以调用 `self.to_string()`（Display 提供）。

## 6.6 trait 对象：`dyn Trait`

有时你想把不同类型塞进同一个集合，按统一接口调用：

```rust
trait Shape { fn area(&self) -> f64; }

struct Circle { r: f64 }
struct Square { side: f64 }
impl Shape for Circle { fn area(&self) -> f64 { 3.14 * self.r * self.r } }
impl Shape for Square { fn area(&self) -> f64 { self.side * self.side } }

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { r: 1.0 }),
        Box::new(Square { side: 2.0 }),
    ];
    for s in &shapes {
        println!("{}", s.area());
    }
}
```

`Box<dyn Shape>` 是 **trait 对象**：运行期通过虚函数表（vtable）调用 `area`。
这就是**动态分发**。

### 静态 vs 动态分发对比

| | 泛型 `<T: Shape>` | `dyn Shape` |
|---|------------------|-------------|
| 决策时机 | 编译期 | 运行期 |
| 性能 | 零开销（单态化） | 一次虚函数调用 |
| 二进制大小 | 每类型一份代码 | 一份 |
| 异构集合 | ✗ | ✓ |
| 对象安全 | 任意 trait | 仅对象安全 trait |

> 📖 对照：TaskFlow 的 `Store` trait 没被当 `dyn Store` 用——因为只有一个实现
> `JsonFileStore`。若以后想支持 `SqliteStore`、`MemoryStore` 并在运行期切换，
> 就可以用 `Box<dyn Store>`。

## 6.7 对象安全（Object Safety）

不是所有 trait 都能做 `dyn`。一个 trait **对象安全**需满足：

1. 方法不能返回 `Self`（trait 对象不知道真实类型）。
2. 方法不能含泛型类型参数（每个具体类型组合要生成一份 vtable 项，无穷无尽）。
3. 第一个参数必须是 `self` / `&self` / `&mut self` / `Box<Self>` 等。

```rust
// ✗ 不是对象安全：clone 返回 Self
trait BadClone { fn clone(&self) -> Self; }
// let x: Box<dyn BadClone> = ...; // 编译错误

// ✓ 对象安全
trait Draw { fn draw(&self); }
let d: Box<dyn Draw> = /* ... */;
```

`Clone`、`Iterator`（带关联类型但 next 返回 `Option<Self::Item>` 实际可对象安全，
但很多方法非对象安全）等不能直接 `dyn`。

## 6.8 `impl Trait` 与 `dyn Trait` 的选择

| 场景 | 选择 |
|------|------|
| 函数参数，调用方多种类型 | `impl Trait` 或泛型 |
| 返回闭包 / 迭代器组合 | `impl Trait` |
| 运行期才知道具体类型 | `Box<dyn Trait>` |
| 异构集合 | `Vec<Box<dyn Trait>>` |

```rust
// 返回闭包必须用 impl Fn 或 Box<dyn Fn>
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
```

## 6.9 trait 扩展方法（Extension Trait 模式）

标准库常用模式：给已有类型"补"方法。

```rust
trait TitleCase {
    fn title_case(&self) -> String;
}

impl TitleCase for str {
    fn title_case(&self) -> String {
        self.split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(first) => first.to_uppercase().chain(c.flat_map(|x| x.to_lowercase())).collect(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn main() {
    println!("{}", "hello world".title_case()); // Hello World
}
```

`impl Trait for str` 让所有 `&str` / `String` 都能用 `title_case`。
serde、tokio、itertools 都大量使用这种"扩展 trait"。

## 6.10 `Display` vs `Debug` vs `ToString`

```rust
trait Display { fn fmt(&self, f: &mut Formatter) -> Result; }
trait Debug { fn fmt(&self, f: &mut Formatter) -> Result; }
```

- `Display`：面向用户，必须手写。
- `Debug`：面向开发者，可 `#[derive(Debug)]`。
- `ToString`：实现 `Display` 后自动获得 `.to_string()`（别手动 impl ToString）。

> 📖 对照：TaskFlow 给 `Task`、`Status`、`Priority` 手写了 `Display`，
> 这样 `format!("{task}")` 就能输出 `(uuid)[高] 标题 ...`。

## 6.11 `From` / `Into` / `TryFrom` / `TryInto`

```rust
let s: String = "hi".into();       // From<&str> for String
let n: i32 = "42".parse().unwrap(); // FromStr

// 自定义
struct Meters(u32);
struct Feet(u32);
impl From<Meters> for Feet {
    fn from(m: Meters) -> Feet { Feet(m.0 * 3) }
}

let m = Meters(10);
let f: Feet = m.into(); // 链式
```

> 📖 对照：TaskFlow 的 `impl From<&Task> for TaskCsvRow` 让 `wtr.serialize(TaskCsvRow::from(i))`
> 能自动转换。实现 `From` 自动获得 `Into`，反之不成立——所以**总是 impl From**。

`TryFrom` / `TryInto` 是可能失败的版本，返回 `Result`：

```rust
let big: i64 = 1_000_000_000_000;
let small: i32 = big.try_into().unwrap_err(); // 超出 i32 范围，返回 Err
```

## 6.12 `AsRef` / `AsMut` / `Borrow`

廉价引用转换，常用于"接受多种字符串形态"的 API：

```rust
fn load<P: AsRef<Path>>(path: P) -> String {
    let p = path.as_ref(); // &Path
    std::fs::read_to_string(p).unwrap()
}

load("file.txt");            // &str
load(String::from("f.txt")); // String
load(PathBuf::from("f.txt"));// PathBuf
load(std::path::Path::new("f.txt")); // &Path
```

> 📖 对照：标准库 `std::fs::read_to_string<P: AsRef<Path>>` 就是这样设计的。
> 这正是"函数参数尽量接受多种形态"的惯用法。

## 6.13 trait 常见陷阱

### 陷阱 1：孤儿规则（Orphan Rule）

不能为外部类型实现外部 trait：

```rust
// ✗ 在你的 crate 里给 Vec 实现 Display（两者都是外部类型）
impl std::fmt::Display for Vec<i32> { /* ... */ }
```

规则：trait 或类型至少有一个是当前 crate 定义的。这避免不同 crate 互相覆盖实现。

### 陷阱 2：trait 方法冲突

两个 trait 有同名方法，类型同时实现二者时调用要消歧：

```rust
trait A { fn f(&self); }
trait B { fn f(&self); }
struct S;
impl A for S { fn f(&self) { println!("A"); } }
impl B for S { fn f(&self) { println!("B"); } }

fn main() {
    let s = S;
    A::f(&s); // 显式指定
    B::f(&s);
}
```

### 陷阱 3：把非对象安全 trait 用作 `dyn`

```rust
trait Iter { type Item; fn next(&mut self) -> Option<Self::Item>; fn map<U, F: Fn(Self::Item) -> U>(self, f: F) -> Map<Self, F, U>; }
// let it: Box<dyn Iter<Item = i32>> = ...; // ✗ map 有泛型，非对象安全
```

## 6.14 练习

1. 定义 `trait Storage { type Key; type Value; fn get(&self, k: &Self::Key) -> Option<&Self::Value>; }`
   并为 `HashMap<String, String>` 实现它（`Key = String, Value = String`）。

2. 把 TaskFlow 的 `Store` trait 改写成支持 `dyn Store` 的形式
   （提示：检查方法签名是否对象安全；返回 `Vec<Task>` 是否有问题？）。

3. 写一个 `trait Summary { fn summarize(&self) -> String; fn author(&self) -> String; }`
   给 `Article` 和 `Tweet` 实现，并放进 `Vec<Box<dyn Summary>>` 遍历。

4. 解释为什么 `Clone` trait 不能直接 `Box<dyn Clone>`。

## 6.15 小结

| 概念 | 一句话 |
|------|--------|
| 默认方法 | trait 提供默认实现，实现者可覆盖 |
| 关联类型 | 实现 trait 时指定，"一对一" |
| 超 trait | `trait B: A` 要求先实现 A |
| `dyn Trait` | 动态分发，异构集合，有 vtable 开销 |
| 对象安全 | 才能做 `dyn`，禁止返回 Self / 泛型方法 |
| `impl Trait` | 静态分发语法糖 |

> 下一章我们深入**闭包与迭代器**——Rust 函数式编程的精髓，也是 TaskFlow 里
> `filter`/`map`/`collect` 背后的全貌。

---

[← 第 5 章](./05_generics.md) | [下一章 →](./07_closures_iterators.md)

---

📧 联系作者：pebblerwon@qq.com
