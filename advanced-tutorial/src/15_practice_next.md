# 第 15 章：综合实战与下一步

## 本章目标

- 用一个综合小项目把前 14 章的核心知识串起来
- 给出可执行的实战题（每题标注考察哪些章节）
- 整理常见学习陷阱与解决方案
- 规划下一步学习方向与资源

## 15.1 综合实战：实现一个 `MiniKV` 存储

我们来设计一个**支持并发、带 TTL 的内存键值存储**。它综合考察：

| 章节 | 用到的知识 |
|------|----------|
| 1-3 | 所有权/借用/生命周期 |
| 4 | `HashMap`、`String`、`PathBuf` |
| 5-6 | 泛型、Trait（`Storage` 抽象） |
| 7 | 迭代器（清理过期项） |
| 8 | `Arc`、`Mutex`（或 `RwLock`） |
| 9 | 多线程、`tokio` 可选 |
| 10 | 模式匹配 `match` |
| 12 | features（持久化可选） |

### 15.1.1 需求

```rust
let store = MiniKv::new();
store.set("a", "1", None);            // 永久
store.set("b", "2", Some(Duration::from_secs(60))); // 60s 后过期
assert_eq!(store.get("a"), Some("1".to_string()));
store.delete("a");
assert_eq!(store.get("a"), None);
```

支持多线程共享：

```rust
let store = Arc::new(MiniKv::new());
let s = Arc::clone(&store);
thread::spawn(move || { s.set("x", "1", None); });
```

### 15.1.2 参考实现骨架

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct MiniKv {
    inner: Arc<RwLock<HashMap<String, Entry>>>,
}

struct Entry {
    value: String,
    expires_at: Option<Instant>,
}

impl Entry {
    fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |t| Instant::now() >= t)
    }
}

impl MiniKv {
    pub fn new() -> Self {
        MiniKv { inner: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn set(&self, key: impl Into<String>, value: impl Into<String>, ttl: Option<Duration>) {
        let mut m = self.inner.write().unwrap();
        m.insert(key.into(), Entry {
            value: value.into(),
            expires_at: ttl.map(|d| Instant::now() + d),
        });
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let m = self.inner.read().unwrap();
        match m.get(key) {
            Some(e) if !e.is_expired() => Some(e.value.clone()),
            _ => None,
        }
    }

    pub fn delete(&self, key: &str) -> bool {
        let mut m = self.inner.write().unwrap();
        m.remove(key).is_some()
    }

    pub fn cleanup_expired(&self) -> usize {
        let mut m = self.inner.write().unwrap();
        let before = m.len();
        m.retain(|_, e| !e.is_expired());
        before - m.len()
    }

    pub fn len(&self) -> usize {
        let m = self.inner.read().unwrap();
        m.values().filter(|e| !e.is_expired()).count()
    }
}
```

### 15.1.3 考察点说明

- `Arc<RwLock<HashMap<...>>>`：第 8、9 章。读多写少选 `RwLock`。
- `impl Into<String>`：第 6 章 `Into`，第 5 章 `impl Trait`。让 `set("a", "1", ...)` 接受 `&str` 和 `String`。
- `retain`：第 7 章迭代器适配（`HashMap::retain` 内部即 filter）。
- `match m.get(key)`：第 10 章模式匹配，带守卫 `if !e.is_expired()`。
- `Clone` for `MiniKv`：克隆的是 `Arc`，引用计数 +1，不复制数据。
- `Option<Duration>`：表示"可能有过期时间"，第 0 章已有基础。

### 15.1.4 进阶练习

1. 加 feature `persist`，启用后 `MiniKv` 把数据定期 dump 到 JSON 文件
   （参考 TaskFlow 的 `JsonFileStore`）。
2. 用 `tokio` 改造为异步版本：`async fn set(...)`，`async fn get(...)`。
3. 加一个 `watch(key)` 返回 `tokio::sync::watch::Receiver`，能在 key 变化时收到通知。
4. 用 `BTreeMap` 替换 `HashMap`，支持 `range("a".."z")` 范围查询。
5. 写并发测试：10 个线程各 set 1000 次，最后验证 `len() == 10000`。

## 15.2 进阶练习题清单

每道题标注考察的章节，做完一题回头查对应章节。

### 题 1：`LinearMap<K, V>`（考察 4, 5, 6, 7）

用 `Vec<(K, V)>` 实现一个简化版 `HashMap`：

```rust
pub struct LinearMap<K, V> { entries: Vec<(K, V)> }
impl<K: PartialEq, V> LinearMap<K, V> {
    pub fn new() -> Self { /* ... */ }
    pub fn insert(&mut self, k: K, v: V) -> Option<V> { /* ... */ }
    pub fn get(&self, k: &K) -> Option<&V> { /* ... */ }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> { /* ... */ }
}
```

### 题 2：`Optional<T>` 自己实现（考察 1-3, 10）

不使用标准库 `Option`，自己写一个 `enum Optional<T> { Some(T), None }`，
实现 `map` / `and_then` / `unwrap_or`，并写单元测试。

### 题 3：`Tree<T>` 二叉搜索树（考察 8, 10）

```rust
enum Tree<T: Ord> { Leaf, Node { val: T, left: Box<Tree<T>>, right: Box<Tree<T>> } }
impl<T: Ord> Tree<T> {
    fn new() -> Self { Tree::Leaf }
    fn insert(&mut self, v: T) { /* ... */ }
    fn contains(&self, v: &T) -> bool { /* ... */ }
    fn in_order(&self) -> impl Iterator<Item = &T> { /* ... */ }  // 进阶
}
```

### 题 4：`Logger` trait 多实现（考察 6, 11）

```rust
trait Logger { fn log(&self, level: Level, msg: &str); }
```

实现 `ConsoleLogger`、`FileLogger`、`MemoryLogger`，放进 `Vec<Box<dyn Logger>>`
同时输出。再加 `#[macro_export] log!` 宏简化调用。

### 题 5：并发 Web 爬虫（考察 9）

用 `tokio` + `reqwest` 并发抓取 10 个 URL，限制最大并发数为 4（用
`tokio::sync::Semaphore`），统计总字节数。

### 题 6：CLI 计算器（考察 10, 11, 12）

写一个支持 `+ - * /` 的命令行计算器：
- 用 `enum Expr` 表示 AST
- 用 `match` 求值
- 用 `clap` 接 `calc "1 + 2 * 3"`
- feature `strict`：开启后除以 0 返回错误而非 panic

## 15.3 常见学习陷阱与对策

### 陷阱 1：被借用检查器劝退

**症状**：编译错误一堆"borrow of moved value"，想放弃。
**对策**：
- 画所有权流向图（谁拥有谁，谁借给谁）
- 优先用借用 `&` / `&mut`，最后才考虑 `clone`
- 用 `Rc`/`Arc` 共享，用 `RefCell`/`Mutex` 内部可变
- 把大函数拆小，缩小借用作用域

### 陷阱 2：到处 `clone()`

**症状**：代码能跑了，但满屏 `.clone()`。
**对策**：
- 静下来想：这里真的需要副本吗？
- 函数参数能改 `&str` 就别要 `String`
- 用生命周期标注让引用传得更远
- 必要时重构数据结构（如用 `Arc<str>` 共享只读）

### 陷阱 3：分不清 `String` 和 `&str`

**症状**：参数类型选错，导致调用方被迫 clone。
**对策**：
- 函数参数：`&str` / `&[T]` / `impl Trait`
- 结构体字段（长期持有）：`String` / `Vec<T>`
- 返回值：要拥有就 `String`，要借用就 `&str`（带生命周期）

### 陷阱 4：滥用 `unwrap`

**症状**：开发时跑得好好的，用户输入异常就 panic。
**对策**：
- 库代码返回 `Result`
- 应用代码在边界用 `?` 传播
- 只在测试 / 确实不可能失败处 `unwrap`

### 陷阱 5：`async` 滥用

**症状**：把所有函数都改成 `async fn`，结果到处 `Send` 报错。
**对策**：
- 没有 IO 并发需求就别上 async
- async 函数里别持有同步锁跨 `.await`
- 优先用同步 + 线程池（`rayon`）

### 陷阱 6：跳过 The Book

**症状**：只看项目实战，基础概念一知半解。
**对策**：把 [The Rust Book](https://doc.rust-lang.org/book/) 前 10 章读一遍，
配合本教程查漏补缺。

## 15.4 下一步学习方向

### 方向 1：Web 后端

| 主题 | 推荐 |
|------|------|
| HTTP 框架 | `axum`（tokio 生态首选）/ `actix-web` |
| 数据库 | `sqlx`（编译期检查 SQL）/ `sea-orm` |
| 序列化 | `serde` + `serde_json`（已会）/ `toml` |
| 鉴权 | `jsonwebtoken`、`argon2`（密码哈希） |
| 模板 | `askama`（编译期）/ `maud`（DSL 宏） |

**练手项目**：把 TaskFlow 改造成 REST API 服务，数据存 SQLite。

### 方向 2：系统工具

| 主题 | 推荐 |
|------|------|
| CLI 增强 | `clap`（已会）/ `ratatui`（TUI）/ `crossterm` |
| 文件系统 | `walkdir`、`notify`（监听） |
| 进程 | `std::process`、`sysinfo` |
| 日志 | `tracing` + `tracing-subscriber` |

**练手项目**：写一个 `find` 替代品，支持正则、并行搜索、彩色输出。

### 方向 3：嵌入式 / WASM

| 主题 | 推荐 |
|------|------|
| 嵌入式 | `embedded-hal`、`cortex-m`、`probe-rs` |
| WASM | `wasm-bindgen`、`wasm-pack` |
| no_std | 不用标准库，适合 MCU |

### 方向 4：深入语言

| 主题 | 资源 |
|------|------|
| 类型系统 | [Rust Nomicon](https://doc.rust-lang.org/nomicon/) |
| 异步原理 | [Tokio Tutorial](https://tokio.rs/tokio/tutorial) |
| 宏进阶 | [The Little Book of Rust Macros](https://veykril.github.io/tlborm/) |
| unsafe | [Rustonomicon](https://doc.rust-lang.org/nomicon/) + Miri |
| 编译器内部 | [rustc dev guide](https://rustc-dev-guide.rust-lang.org/) |

### 方向 5：给 TaskFlow 加功能（承接已有项目）

| 功能 | 涉及章节 | 难度 |
|------|---------|------|
| Newtype `TaskId` | 14 | ★ |
| Builder 构造 `Task` | 14 | ★ |
| `tag_frequency` 用 `BTreeMap` | 4 | ★ |
| 并发批量导出 | 9 | ★★ |
| TUI 界面（`ratatui`） | 12 | ★★★ |
| SQLite 持久化（`rusqlite`） | 6 | ★★★ |
| HTTP 同步（`reqwest` + `tokio`） | 9 | ★★★★ |
| 插件系统（`dyn Store`） | 6 | ★★★ |

## 15.5 推荐学习资源汇总

| 资源 | 类型 | 适合阶段 |
|------|------|---------|
| [The Rust Book](https://doc.rust-lang.org/book/) | 官方教程 | 入门必读 |
| [Rust By Example](https://doc.rust-lang.org/rust-by-example/) | 示例 | 边查边用 |
| [Rustlings](https://github.com/rust-lang/rustlings) | 练习题 | 巩固基础 |
| [Rust Reference](https://doc.rust-lang.org/reference/) | 语言规范 | 深入细节 |
| [Rust Nomicon](https://doc.rust-lang.org/nomicon/) | unsafe/高级 | 进阶 |
| [Tokio Tutorial](https://tokio.rs/tokio/tutorial) | 异步 | 学并发 |
| [Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/) | 链表 | 深入所有权 |
| [Programming Rust (O'Reilly)](https://www.oreilly.com/library/view/programming-rust-2nd/9781492052586/) | 参考书 | 全面 |
| [Zero To Production In Rust](https://www.zero2prod.com/) | 全栈实战 | Web 后端 |
| [Jon Gjengset YouTube](https://www.youtube.com/@jonhoo) | 视频直播 | 高级实战 |

### 必装工具

```bash
rustup component add clippy rust-src rustfmt
cargo install cargo-expand      # 宏展开
cargo install cargo-watch       # 文件变化自动重编译
cargo install cargo-edit        # cargo add
cargo install cargo-outdated    # 检查过期依赖
cargo install cargo-audit       # 安全漏洞
cargo install cargo-bloat       # 体积分析
cargo install cargo-nextest     # 更快测试运行器
rustup toolchain install nightly
rustup +nightly component add miri  # UB 检测
```

## 15.6 知识图谱回顾

```
                    ┌─────────────────────────┐
                    │  所有权 (1) ── 借用 (2) │
                    │       └── 生命周期 (3)  │
                    └────────────┬────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              ▼                  ▼                  ▼
         字符串/集合(4)      泛型(5)          Trait进阶(6)
                                 │                  │
                                 └──────┬───────────┘
                                        ▼
                                  闭包/迭代器(7)
                                        │
                          ┌─────────────┼─────────────┐
                          ▼             ▼             ▼
                     智能指针(8)    并发/异步(9)   模式匹配(10)
                          │             │             │
                          └──────┬──────┴──────┬──────┘
                                 ▼             ▼
                              宏(11)      Cargo/cfg(12)
                                 │             │
                                 └──────┬──────┘
                                        ▼
                                   Unsafe(13)
                                        │
                                        ▼
                                  设计模式(14)
                                        │
                                        ▼
                                  综合实战(15)
```

## 15.7 最终寄语

你已经走过了 Rust 最陡峭的部分：

- ✅ 所有权、借用、生命周期——Rust 区别于其它语言的核心
- ✅ 泛型与 trait——抽象的两种方式
- ✅ 闭包与迭代器——函数式风格
- ✅ 智能指针与并发——内存模型与无畏并发
- ✅ 模式匹配与运算符重载——类型驱动的控制流
- ✅ 宏、Cargo 工程化、Unsafe——元编程与底层
- ✅ 设计模式——把概念串成工程

接下来最好的学习方式：**写一个你想用的项目**。把 TaskFlow 升级成你自己的
"完美任务管理器"，或者另起炉灶做个爬虫、TUI、Web 服务。在真实需求里遇到的
每一个编译错误，都是一次成长。

Rust 的回报是**长期**的：初期慢，后期你会发现自己写代码时心里特别踏实——
"能编译，就大概率没数据竞争；能编译，就大概率没悬垂引用"。这种信心，是
GC 语言和 C/C++ 都给不了的。

**Keep Rusting, fearlessly.**

---

[← 第 14 章](./14_design_patterns.md) | [返回概览](./00_overview.md)

---

📧 联系作者：pebblerwon@qq.com
