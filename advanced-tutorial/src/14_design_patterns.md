# 第 14 章：设计模式与最佳实践

## 本章目标

- 学会用 Rust 惯用法表达常见设计模式
- 掌握 Newtype、Builder、Typestate、Strategy 等模式
- 理解"Rust 不是 OOP"对设计模式的影响
- 通过 TaskFlow 代码反观其使用的模式与可改进点

## 14.1 Rust 里的设计模式

很多 GoF 设计模式在 Rust 里"消失"了——因为 trait + 泛型 + enum 已经覆盖。
比如：

| OOP 模式 | Rust 对应 |
|---------|----------|
| 策略模式 | 闭包 / trait object / 泛型 |
| 状态模式 | enum + match / typestate |
| 空对象 | `Option<T>` |
| 建造者 | `Builder` struct |
| 访问者 | 较少用，enum + match 更地道 |
| 装饰器 | trait + wrapper / 组合 |
| 单例 | `OnceLock<T>` / lazy_static |

## 14.2 Newtype 模式

用元组结构体给已有类型"重新命名"，获得新的类型安全：

```rust
struct Meters(f64);
struct Feet(f64);

impl Meters {
    fn to_feet(self) -> Feet { Feet(self.0 * 3.28084) }
}

fn fall(d: Meters) -> f64 { /* ... */ }

fn main() {
    let d = Meters(10.0);
    fall(d);          // ✓
    // fall(Feet(32.8)); // ✗ 类型不匹配
}
```

**好处**：
- 编译期防止单位混淆
- 可以为 newtype 实现 trait，而不影响原类型（绕过孤儿规则）
- 零运行时开销

> 📖 对照：TaskFlow 没用 Newtype，但 `Task.id: String` 可以改成 `TaskId(String)`——
> 这样就不会把 `task.title` 误传给需要 ID 的函数。

### Newtype 继承方法

```rust
struct MyVec(Vec<i32>);

impl MyVec {
    fn sum(&self) -> i32 { self.0.iter().sum() }
}

// 用 Deref 让 MyVec 自动获得 Vec 的方法
impl std::ops::Deref for MyVec {
    type Target = Vec<i32>;
    fn deref(&self) -> &Vec<i32> { &self.0 }
}

fn main() {
    let v = MyVec(vec![1, 2, 3]);
    println!("{}", v.len()); // 来自 Vec
    println!("{}", v.sum()); // 来自 MyVec
}
```

> ⚠ `Deref` 仅适用于"智能指针语义"，别滥用为"继承"。

## 14.3 Builder 模式

当结构体字段多、可选字段多，构造函数参数爆炸时：

```rust
pub struct Task {
    title: String,
    priority: Priority,
    due: Option<NaiveDate>,
    tags: Vec<String>,
}

pub struct TaskBuilder {
    title: Option<String>,
    priority: Option<Priority>,
    due: Option<NaiveDate>,
    tags: Vec<String>,
}

impl TaskBuilder {
    pub fn new() -> Self {
        TaskBuilder {
            title: None,
            priority: None,
            due: None,
            tags: vec![],
        }
    }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = Some(t.into()); self }
    pub fn priority(mut self, p: Priority) -> Self { self.priority = Some(p); self }
    pub fn due(mut self, d: NaiveDate) -> Self { self.due = Some(d); self }
    pub fn tag(mut self, t: impl Into<String>) -> Self { self.tags.push(t.into()); self }

    pub fn build(self) -> Result<Task, String> {
        let title = self.title.ok_or("title 必填")?;
        Ok(Task {
            title,
            priority: self.priority.unwrap_or(Priority::Medium),
            due: self.due,
            tags: self.tags,
        })
    }
}

// 使用
let t = TaskBuilder::new()
    .title("学 Rust")
    .priority(Priority::High)
    .tag("study")
    .build()?;
```

> 📖 对照：TaskFlow 用 `clap` 的 derive 间接实现了 builder——`Commands::Add { title, priority, .. }`
> 的字段就是 builder 的步骤。手写 builder 适合复杂内部构造。

`derive_builder` crate 能自动生成 builder。

## 14.4 Typestate 模式：类型编码状态

让"非法状态"在编译期就不可能：

```rust
// 用泛型参数编码"状态"
struct Draft;
struct Published;

struct Post<S> { content: String, _state: std::marker::PhantomData<S> }

impl Post<Draft> {
    fn new(content: String) -> Self { Post { content, _state: PhantomData } }
    fn publish(self) -> Post<Published> { Post { content: self.content, _state: PhantomData } }
}

impl Post<Published> {
    fn content(&self) -> &str { &self.content }
}

fn main() {
    let draft = Post::<Draft>::new("hi".into());
    // draft.content(); // ✗ Draft 状态没有 content 方法
    let pub_ = draft.publish();
    println!("{}", pub_.content()); // ✓
}
```

`PhantomData<T>` 是零大小类型，仅用于"标记"。

> 适合状态机、构建流程（未连接/已连接/已认证）。TaskFlow 的 `Status` 用的是
> 运行期 enum，更灵活但运行期检查；typestate 是编译期检查，更安全但更死板。

## 14.5 策略模式：trait / 闭包

```rust
// 用 trait
trait Sorter<T: Ord> { fn sort(&self, &mut [T]); }
struct QuickSorter;
impl<T: Ord> Sorter<T> for QuickSorter { fn sort(&self, xs: &mut [T]) { xs.sort(); } }

fn sort_with<T: Ord, S: Sorter<T>>(xs: &mut [T], s: S) { s.sort(xs); }

// 更轻量：直接传闭包
fn sort_by<T, F: FnMut(&T, &T) -> std::cmp::Ordering>(xs: &mut [T], cmp: F) {
    xs.sort_by(cmp);
}
```

> 📖 对照：TaskFlow 的 `Store` trait 就是策略模式——`JsonFileStore` 是一种策略，
> 以后可加 `SqliteStore` 等等。

## 14.6 错误处理最佳实践

> 📖 对照：TaskFlow 已经用了 `thiserror` + `anyhow` 双层方案。这里补全要点。

### 库 vs 应用

- **库**：用 `thiserror` 定义自己的错误枚举，返回 `Result<T, MyError>`。
  **不要**返回 `anyhow::Error`——下游无法 `match` 你的错误。
- **应用**：在边界（main）用 `anyhow::Result`，方便加 context。

### 错误链与 context

```rust
use anyhow::Context;

fn load_config(path: &str) -> anyhow::Result<Config> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("读取配置文件失败：{path}"))?;
    let cfg: Config = toml::from_str(&text).context("解析配置失败")?;
    Ok(cfg)
}
```

`anyhow::Error` 的 `{:#}` 打印完整错误链：

```
读取配置文件失败：/etc/app.toml

Caused by:
    No such file or directory (os error 2)
```

### 不要 panic 跨边界

库代码**绝不应该** `panic!` / `unwrap` / `expect`——返回 `Result`。
只有以下情况可 panic：
- 真的不可能恢复（如内存分配失败）
- 测试代码
- 程序初始化阶段（main 早期）

### `?` 还是 `match`

`?` 默认。需要分支处理才 `match`。

## 14.7 RAII：资源即类型

Rust 通过 `Drop` 实现 RAII（Resource Acquisition Is Initialization）：

```rust
struct File { fd: i32 }
impl Drop for File {
    fn drop(&mut self) { unsafe { libc::close(self.fd); } }
}

fn main() {
    let f = File { fd: 3 };
    // f 离开作用域自动 close，无需 finally
}
```

> 📖 对照：TaskFlow 的 `TempDir`（tempfile crate）就是 RAII——测试结束自动删除。
> 永远优先 RAII 而非手动 `close`/`unlock`/`free`。

## 14.8 `Cow` 避免无谓分配

```rust
use std::borrow::Cow;

fn escape(s: &str) -> Cow<str> {
    if s.contains('<') {
        Cow::Owned(s.replace('<', "&lt;"))
    } else {
        Cow::Borrowed(s) // 零拷贝
    }
}
```

适合"大多数情况无需修改，少数需要"的场景。

## 14.9 `Default` + struct 更新语法

```rust
#[derive(Default)]
struct Config {
    host: String,
    port: u16,
    debug: bool,
}

let base = Config { port: 8080, ..Default::default() };
let debug = Config { debug: true, ..base }; // 复用其余字段
```

> 📖 对照：TaskFlow 的 `TaskStats` 用了 `..Default::default()` 来初始化大部分字段。

## 14.10 命名约定

| 项 | 风格 | 例 |
|----|------|---|
| 类型（struct/enum/trait） | UpperCamelCase | `TaskService` |
| 函数/方法/变量/模块 | snake_case | `add_task` |
| 常量/静态 | SCREAMING_SNAKE | `MAX_TAGS` |
| 泛型参数 | 单大写或 UpperCamel | `T` / `Key` |
| lifetime | 短小写带撇 | `'a` / `'ctx` |

`cargo fmt` 自动处理格式，`cargo clippy` 提示命名问题。

## 14.11 文档注释与 rustdoc

```rust
/// 给任务加上截止日期。
///
/// # 参数
/// - `id`: 任务 ID（支持前缀匹配）
/// - `due`: 截止日期，格式 YYYY-MM-DD
///
/// # 错误
/// - `TaskError::NotFound`：ID 不存在
/// - `TaskError::InvalidDate`：日期格式错误
///
/// # 示例
/// ```
/// use taskflow::service::TaskService;
/// let svc = TaskService::new().unwrap();
/// svc.update_task("a1", None, None, None, None, vec![], Some("2026-12-31")).unwrap();
/// ```
pub fn update_task(...) -> Result<Task> { /* ... */ }
```

- `///`：项文档
- `//!`：模块/crate 文档（放在文件顶部）
- 代码块会被 `cargo test --doc` 当**文档测试**跑

> 📖 对照：TaskFlow 的 `clap` 命令注释 `/// 创建新任务` 会自动变成 CLI help 文本。

## 14.12 常见反模式

### 反模式 1：到处 `.clone()`

新手看到借用报错就 clone。**先想清楚所有权流向**，能用借用就用。
只在确实需要独立副本时 clone。

### 反模式 2：`String` 满天飞

参数全用 `String` 而非 `&str`，导致无谓分配。函数参数优先 `&str` / `&[T]` / `impl Trait`。

### 反模式 3：`unwrap()` 在生产代码

```rust
let n: i32 = s.parse().unwrap(); // 用户输入 "abc" 就 panic
```

返回 `Result` 或用 `?`。只在测试、demo、确实不可能失败处用 `unwrap`。

### 反模式 4：用 `Box<dyn Trait>` 代替泛型

不必要的动态分发。能用泛型 `<T: Trait>` 就别用 `Box<dyn Trait>`——零开销 + 编译期检查。

### 反模式 5：模仿 OOP 继承

Rust 没有继承。别用 `Deref` 模拟继承，别建深层 trait 层级。优先**组合**。

### 反模式 6：滥用 `RefCell`

绕过编译期检查的运行期 panic 更难调。先想清楚能不能改设计让编译期通过。

## 14.13 TaskFlow 模式复盘

回看 TaskFlow 用到的模式：

| 模式 | 体现 |
|------|------|
| 分层架构 | cli / service / store / display / error |
| 策略（trait） | `Store` trait + `JsonFileStore` 实现 |
| 错误分层 | `thiserror`（库）+ `anyhow`（应用） |
| RAII | `TempDir` 测试目录 |
| `From` 转换 | `impl From<&Task> for TaskCsvRow` |
| `Default` 简化构造 | `TaskStats { ..Default::default() }` |
| Builder（间接） | clap derive 的 `Commands::Add { ... }` |
| Newtype（待加） | 可把 `String` ID 升级为 `TaskId(String)` |

## 14.14 练习

1. 给 TaskFlow 加一个 Newtype `pub struct TaskId(pub String)`，
   并改造 `Task::id` 字段。思考：哪些函数签名要改？有什么收益？

2. 为 `Task` 写一个 `TaskBuilder`，并加单元测试验证必填字段缺失时返回错误。

3. 用 typestate 模式实现一个 `Connection` 类型，状态为 `Disconnected` / `Connected` /
   `Authenticated`，只有 `Authenticated` 才能调 `query`。

4. 用 `Cow<str>` 改造 TaskFlow 的 `export_tasks`：空任务时返回借用切片，有任务时
   才分配 `String`。

## 14.15 小结

| 概念 | 一句话 |
|------|--------|
| Newtype | 零成本给类型加语义 |
| Builder | 多字段/可选字段构造 |
| Typestate | 把状态编码进类型，编译期检查 |
| 策略 | trait / 闭包，泛型优先 |
| RAII | Drop 自动释放 |
| 库用 thiserror，应用用 anyhow | 错误分层 |
| 别到处 clone / unwrap | 先想清楚所有权与错误 |

> 最后一章我们把所有知识串成实战，给出后续学习路线。

---

[← 第 13 章](./13_unsafe.md) | [下一章 →](./15_practice_next.md)

---

📧 联系作者：pebblerwon@qq.com
