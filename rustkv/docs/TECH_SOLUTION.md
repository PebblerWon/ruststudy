# RustKV 技术方案

## 1. 技术栈

### 1.1 核心依赖

| 类别 | 库 | 版本 | 用途 | myapp 是否用过 |
|------|-----|------|------|---------------|
| 异步运行时 | `tokio` | 1.x (full) | async runtime、定时器、异步 IO | ❌ 新学 |
| CLI 解析 | `clap` | 4.x (derive) | 命令行参数解析 | ✅ 复用 |
| 序列化 | `serde` + `serde_json` | 1.x | 快照/WAL 序列化 | ✅ 复用 |
| 时间 | `chrono` | 0.4.x | 日志时间戳 | ✅ 复用 |
| 错误(应用) | `anyhow` | 1.x | 应用层错误处理 | ✅ 复用 |
| 错误(库) | `thiserror` | 1.x | 库层错误类型 | ✅ 复用 |
| 目录 | `dirs` | 5.x | 获取 home 目录 | ✅ 复用 |
| 基准测试 | `criterion` | 0.5.x (dev) | 性能基准测试 | ❌ 新学 |

### 1.2 标准库重点模块（学习目标）

| 模块 | 学习内容 | 对应阶段 |
|------|---------|---------|
| `std::boxed::Box` | 堆分配、递归类型 | Phase 1 |
| `std::rc::Rc` | 引用计数、共享只读 | Phase 1 |
| `std::cell::RefCell` | 内部可变性、运行时借用检查 | Phase 1 |
| `std::sync::Arc` | 线程安全引用计数 | Phase 2 |
| `std::sync::Mutex` | 互斥锁、RAII 锁释放 | Phase 2 |
| `std::thread` | 线程创建、JoinHandle | Phase 2 |
| `std::sync::mpsc` | 通道、生产者-消费者 | Phase 2 |
| `std::marker::{Send, Sync}` | 线程安全标记 trait | Phase 2 |
| `std::ops` | 运算符重载 (Add 等) | Phase 4 |
| `std::iter::Iterator` | 自定义迭代器、关联类型 | Phase 4 |
| `std::ops::Drop` | 析构、RAII 资源清理 | Phase 4 |
| `std::convert::{From, Into}` | 类型转换 trait | Phase 1 |
| `macro_rules!` | 声明式宏 | Phase 4 |

### 1.3 Cargo.toml 配置参考

```toml
[package]
name = "rustkv"
version = "0.1.0"
edition = "2021"
description = "An embedded concurrent key-value store for Rust learning"

[lib]
name = "rustkv"
path = "src/lib.rs"

[[bin]]
name = "rustkv"
path = "src/main.rs"

[dependencies]
tokio = { version = "1", features = ["full"] }  # Phase 3 启用
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "1"
dirs = "5"
clap = { version = "4", features = ["derive"] }  # Phase 5 启用

[dev-dependencies]
criterion = "0.5"
tempfile = "3"
assert_cmd = "2"
predicates = "3"
```

---

## 2. 项目结构

```
rustkv/
├── Cargo.toml
├── docs/
│   ├── PRD.md
│   ├── DEV_PLAN.md
│   └── TECH_SOLUTION.md
├── src/
│   ├── lib.rs              # 库入口：模块导出
│   ├── main.rs             # CLI 入口（Phase 5）
│   ├── cli.rs              # clap 命令定义（Phase 5）
│   ├── engine.rs           # 核心引擎：KV 操作
│   ├── models/
│   │   ├── mod.rs          # 模块导出
│   │   ├── value.rs        # Value 枚举（泛型 + From + Add）
│   │   ├── entry.rs        # Entry 结构体（含 TTL）
│   │   └── linked_list.rs  # 自定义 LinkedList（Box 递归类型）
│   ├── wal.rs              # Write-Ahead Log（mpsc → async）
│   ├── ttl.rs              # TTL 过期管理（async timer）
│   ├── snapshot.rs         # 快照持久化（JSON 序列化）
│   ├── scanner.rs          # 范围扫描（自定义 Iterator）
│   ├── error.rs            # 错误类型定义
│   └── macros.rs           # 声明式宏（kv! / log_kv!）
├── tests/
│   └── integration_test.rs # 集成测试
└── benches/
    └── bench.rs            # 基准测试（Phase 5）
```

### 各文件职责与学习重点

| 文件 | 职责 | 学习重点 |
|------|------|---------|
| `models/value.rs` | Value 枚举定义 | 泛型、From/Into 手动实现、运算符重载 |
| `models/linked_list.rs` | 链表数据结构 | Box 递归类型、Option 链式操作 |
| `models/entry.rs` | 键值条目 | Instant 时间处理、TTL 判定 |
| `engine.rs` | 核心存储引擎 | RefCell→Arc<Mutex> 演进、所有权设计 |
| `wal.rs` | 写前日志 | mpsc channel → async channel 演进 |
| `ttl.rs` | 过期清理 | tokio::spawn、async timer、select! |
| `snapshot.rs` | 快照持久化 | serde 序列化、文件 IO |
| `scanner.rs` | 范围扫描 | 自定义 Iterator、关联类型 |
| `error.rs` | 错误类型 | thiserror（复用 myapp 技能） |
| `macros.rs` | 宏定义 | macro_rules!、片段说明符 |
| `cli.rs` | CLI 定义 | clap derive（复用 myapp 技能） |

---

## 3. 架构设计

### 3.1 分层架构

```
┌─────────────────────────────────────────┐
│              CLI 层（Phase 5）           │
│  main.rs + cli.rs                       │
│  职责：解析参数 → 调用 Engine → 输出结果  │
└──────────────────┬──────────────────────┘
                   │ 调用
                   ▼
┌─────────────────────────────────────────┐
│            Engine 层（核心）              │
│  engine.rs                              │
│  职责：KV 增删改查、TTL 判断、并发控制     │
│  演进：RefCell → Arc<Mutex> → async      │
└──────┬──────────┬───────────┬───────────┘
       │          │           │
       ▼          ▼           ▼
┌──────────┐ ┌────────┐ ┌──────────┐
│ WAL 层   │ │ TTL 层 │ │Snapshot层│
│ wal.rs   │ │ttl.rs  │ │snapshot  │
│          │ │        │ │  .rs     │
│thread +  │ │tokio:: │ │serde +   │
│mpsc →    │ │spawn + │ │文件 IO   │
│async     │ │sleep   │ │          │
└──────────┘ └────────┘ └──────────┘

┌─────────────────────────────────────────┐
│           数据模型层                      │
│  models/value.rs + linked_list.rs        │
│  职责：类型定义、转换、运算符重载          │
└─────────────────────────────────────────┘
```

### 3.2 数据流

```
CLI 命令 (PUT/GET/DEL/SCAN/MERGE)
    │
    ▼
engine.rs: 加锁 → 操作 HashMap → 解锁
    │
    ├── PUT/DEL → wal.rs: 发送到 channel → 后台写文件
    ├── PUT --ttl → ttl.rs: 注册过期任务
    ├── SCAN → scanner.rs: 返回自定义 Iterator
    ├── MERGE → value.rs: 运算符重载 (Add)
    └── PERSIST → snapshot.rs: 全量序列化
```

### 3.3 演进路线（核心设计）

本项目刻意设计三阶段演进，让学习者体验 Rust 并发模型的递进：

```
Phase 1: RefCell<HashMap>              → 单线程，内部可变性
Phase 2: Arc<Mutex<HashMap>>           → 多线程，编译期线程安全
Phase 3: Arc<Mutex<HashMap>> + tokio   → 异步，非阻塞 IO
```

每一步演进都有明确的对比和学习价值，详见后续章节的代码对比。

---

## 4. 核心模块设计

### 4.1 Value 类型 (`models/value.rs`)

**学习目标：** 泛型枚举、From/Into 手动实现、运算符重载

```rust
use std::collections::HashMap;
use crate::models::linked_list::LinkedList;

/// 多态值类型：支持字符串、整数、列表、哈希
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    List(LinkedList),       // 自定义链表（Box 递归类型，见 §4.2）
    Hash(HashMap<String, String>),
}

/// 手动实现 From<&str>（myapp 用 #[from] 自动生成，这里手写学习原理）
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Integer(n)
    }
}

/// Display trait 友好展示
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Integer(n) => write!(f, "{n}"),
            Value::List(list) => write!(f, "{list}"),
            Value::Hash(map) => {
                let pairs: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
        }
    }
}

impl Value {
    /// 返回类型名称（用于错误信息）
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "String",
            Value::Integer(_) => "Integer",
            Value::List(_) => "List",
            Value::Hash(_) => "Hash",
        }
    }
}
```

**设计要点：**

- `Value` 是枚举而非 trait 对象——值类型有限且固定，枚举比 `dyn` 更高效（栈分配、无虚函数调用）
- `List(LinkedList)` 使用自定义链表而非 `Vec<String>`——刻意使用 `Box` 递归类型作为学习练习
- `From` 手动实现而非 `#[from]` 自动生成——理解 trait 转换原理，对比 myapp 中 `#[from]` 的用法
- 运算符重载（`Add`）实现见 [§ 4.8](#48-运算符重载-modelsvaluers-扩展)

### 4.2 LinkedList (`models/linked_list.rs`)

**学习目标：** Box 智能指针、递归类型、为何递归类型必须用 Box

```rust
/// 链表节点：递归类型，必须用 Box 包装
/// 不用 Box 会导致「无限大小」编译错误
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub value: String,
    pub next: Option<Box<Node>>,   // Box 把 Node 放到堆上，打破大小循环
}

/// 单链表
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LinkedList {
    head: Option<Box<Node>>,
    len: usize,
}

impl LinkedList {
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }

    /// 头部插入：O(1)
    pub fn push(&mut self, value: String) {
        let new_node = Box::new(Node {
            value,
            next: self.head.take(),  // take() 把 Option 的值取走，留下 None
        });
        self.head = Some(new_node);
        self.len += 1;
    }

    /// 头部弹出：O(1)
    pub fn pop(&mut self) -> Option<String> {
        self.head.take().map(|node| {
            self.head = node.next;  // 下一个节点成为新的 head
            self.len -= 1;
            node.value
        })
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    /// 从 Vec 创建（测试辅助）
    pub fn from_vec(items: &[&str]) -> Self {
        let mut list = Self::new();
        for item in items.iter().rev() {
            list.push(item.to_string());
        }
        list
    }
}

impl std::fmt::Display for LinkedList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut current = &self.head;
        let mut items = Vec::new();
        while let Some(node) = current {
            items.push(&node.value);
            current = &node.next;
        }
        write!(f, "[{}]", items.join(", "))
    }
}
```

**设计要点：**

- **为何必须用 Box**：`Node` 包含 `Option<Node>`，如果不用 Box，编译器需要计算 `Node` 的大小 = `String` 大小 + `Option<Node>` 大小 = 无限递归。`Box` 是固定大小的指针（8 字节），打破循环
- `Option<Box<Node>>` 是 Rust 链表的标准模式：`Option` 表示「可能没有」，`Box` 表示「在堆上」
- `take()` 方法：把 `Option` 的值取出来并留下 `None`，避免所有权问题——这是 Rust 链表操作的核心技巧
- 推荐对比学习：`Vec<String>` 是连续内存数组，`LinkedList` 是堆上链式结构，各有适用场景

### 4.3 Engine 基础（Phase 1 单线程版）(`engine.rs`)

**学习目标：** RefCell 内部可变性、Rc 引用计数、生命周期限制

```rust
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::models::{Entry, Value};

/// 引擎配置
#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub wal_enabled: bool,
    pub ttl_check_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".rustkv"),
            wal_enabled: true,
            ttl_check_interval: Duration::from_secs(1),
        }
    }
}

/// Phase 1：单线程引擎，用 RefCell 实现内部可变性
pub struct Engine {
    store: RefCell<HashMap<String, Entry>>,
    config: Rc<Config>,  // Rc 共享配置，多个方法引用同一份
}

impl Engine {
    pub fn new(config: Config) -> Self {
        Self {
            store: RefCell::new(HashMap::new()),
            config: Rc::new(config),
        }
    }

    pub fn put(&self, key: &str, value: Value, ttl: Option<Duration>) {
        let entry = Entry::new(value, ttl);
        self.store.borrow_mut().insert(key.to_string(), entry);
        // borrow_mut() 获取可变借用，insert 后 RefMut guard 自动 drop
    }

    /// 返回 Option<Value> 而非 &Value
    /// 原因：RefCell::borrow() 返回的 Ref<'_, T> 是 RAII guard，
    /// 不能逃逸出函数，所以必须 clone 出来
    pub fn get(&self, key: &str) -> Option<Value> {
        let store = self.store.borrow();
        store.get(key).and_then(|entry| {
            if entry.is_expired() { None } else { Some(entry.value.clone()) }
        })
    }

    pub fn del(&self, key: &str) -> bool {
        self.store.borrow_mut().remove(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.store.borrow().len()
    }

    pub fn keys(&self, pattern: &str) -> Vec<String> {
        let store = self.store.borrow();
        store.keys()
            .filter(|k| k.starts_with(pattern))
            .cloned()
            .collect()
    }
}
```

**设计要点：**

- **RefCell**：`borrow()` 获取不可变借用，`borrow_mut()` 获取可变借用。借用检查在运行时进行，违反规则（如同时 borrow 和 borrow_mut）会 panic
- **Rc**：`Rc::new(config)` 创建引用计数的 Config。适合单线程共享只读数据，`Rc::clone()` 增加引用计数而非深拷贝
- **生命周期限制**：`get` 看似可以返回 `&Value`，但 `RefCell::borrow()` 返回的 `Ref<'_, T>` 是 RAII guard，不能逃逸出函数。所以返回 `Option<Value>`（clone），这是 RefCell 的固有限制
- **为什么 Phase 1 用 RefCell 而非直接 `&mut self`**：Engine 方法接收 `&self`（不可变引用），但需要修改内部 HashMap。RefCell 允许在不可变引用下修改内部数据——这就是「内部可变性」

### 4.4 Engine 线程安全版（Phase 2 演进）

**学习目标：** Arc、Mutex、RAII 锁释放、Send/Sync

```rust
// Phase 2：将 RefCell 替换为 Mutex，Rc 替换为 Arc

use std::sync::{Arc, Mutex};

pub struct Engine {
    store: Arc<Mutex<HashMap<String, Entry>>>,
    config: Arc<Config>,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(config),
        }
    }

    pub fn put(&self, key: &str, value: Value, ttl: Option<Duration>)
        -> Result<(), KvError>
    {
        let entry = Entry::new(value, ttl);
        // lock() 返回 LockResult<MutexGuard>，? 传播 PoisonError
        let mut store = self.store.lock()?;
        store.insert(key.to_string(), entry);
        Ok(())
        // store 在此处 drop，Mutex 自动释放锁（RAII）
    }

    pub fn get(&self, key: &str) -> Result<Option<Value>, KvError> {
        let store = self.store.lock()?;
        Ok(store.get(key).and_then(|entry| {
            if entry.is_expired() { None } else { Some(entry.value.clone()) }
        }))
        // 锁在此释放
    }

    pub fn del(&self, key: &str) -> Result<bool, KvError> {
        let mut store = self.store.lock()?;
        Ok(store.remove(key).is_some())
    }

    pub fn len(&self) -> Result<usize, KvError> {
        Ok(self.store.lock()?.len())
    }

    /// 多线程并发写入示例
    pub fn concurrent_put(
        engine: &Engine,
        entries: Vec<(String, Value)>,
    ) -> Vec<std::thread::JoinHandle<()>> {
        entries.into_iter().map(|(key, value)| {
            let store = Arc::clone(&engine.store);  // Arc::clone 增加引用计数
            std::thread::spawn(move || {
                let mut s = store.lock().unwrap();
                s.insert(key, Entry::new(value, None));
            })
            // store 的 Arc 引用计数在闭包结束时 -1
        }).collect()
    }
}
```

**对比表：**

| 特性 | RefCell | Mutex | RwLock |
|------|---------|-------|--------|
| 借用检查 | 运行时 panic | 阻塞等待 | 阻塞等待 |
| 线程安全 | ❌ 单线程 | ✅ 多线程 | ✅ 多线程 |
| 读并发 | ❌ | ❌（读也互斥） | ✅（多读并发） |
| 适用场景 | 单线程内部可变性 | 多线程读写 | 多读少写 |

| 特性 | Rc | Arc |
|------|-----|-----|
| 线程安全 | ❌ 单线程 | ✅ 多线程 |
| 计数操作 | 非原子 | 原子（CAS） |
| Send trait | ❌ | ✅ |
| 适用场景 | 单线程共享 | 多线程共享 |

**设计要点：**

- **RefCell → Mutex**：`borrow_mut()` → `lock()?`，借用检查从运行时 panic 变为阻塞等待
- **Rc → Arc**：`Rc` 不是 `Send`（非线程安全），跨线程必须用 `Arc`。尝试跨线程发送 `Rc` 会编译错误——这就是 Rust 的「无畏并发」
- **RAII 锁释放**：`MutexGuard` 在 drop 时自动释放锁，不需要手动 `unlock()`，也不会忘记释放
- **PoisonError**：如果持有锁的线程 panic，锁会「中毒」，后续 `lock()` 返回 `Err`。用 `?` 传播

### 4.5 WAL 写前日志 (`wal.rs`)

**学习目标：** mpsc channel（Phase 2）→ async channel（Phase 3）

#### Phase 2：std::sync::mpsc 版本

```rust
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalOp {
    Put { key: String, value: Value, ttl_secs: Option<u64> },
    Del { key: String },
}

pub struct Wal {
    sender: Sender<WalOp>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Wal {
    pub fn new(path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel::<WalOp>();

        let handle = thread::spawn(move || {
            // 后台线程：消费 channel，批量写文件
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .expect("打开 WAL 文件失败");

            for op in rx {
                // rx 是 Iterator：channel 关闭（sender 全部 drop）后迭代结束
                let line = serde_json::to_string(&op).expect("WAL 序列化失败");
                writeln!(file, "{line}").expect("WAL 写入失败");
            }
        });

        Self { sender: tx, handle: Some(handle) }
    }

    pub fn append(&self, op: WalOp) -> Result<(), KvError> {
        self.sender.send(op).map_err(|_| KvError::WalClosed)
        // send 是非阻塞的：放入 channel buffer 后立即返回
        // 实际写文件由后台线程异步完成
    }
}
```

#### Phase 3：tokio::sync::mpsc 版本

```rust
use tokio::sync::mpsc::{self, Sender};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct AsyncWal {
    sender: Sender<WalOp>,
    handle: tokio::task::JoinHandle<()>,
}

impl AsyncWal {
    pub async fn new(path: PathBuf) -> Self {
        // 有界 channel，容量 1024：满时 send().await 挂起生产者（背压）
        let (tx, mut rx) = mpsc::channel::<WalOp>(1024);

        let handle = tokio::spawn(async move {
            let mut file = File::create(&path).await.expect("打开 WAL 文件失败");

            while let Some(op) = rx.recv().await {
                // recv().await：channel 为空时挂起 task，不阻塞线程
                let line = serde_json::to_string(&op).expect("WAL 序列化失败");
                file.write_all(format!("{line}\n").as_bytes()).await
                    .expect("WAL 写入失败");
                file.flush().await.ok();
            }
            // rx.recv() 返回 None 时结束（sender 全部 drop）
        });

        Self { sender: tx, handle }
    }

    pub async fn append(&self, op: WalOp) -> Result<(), KvError> {
        self.sender.send(op).await
            .map_err(|_| KvError::WalClosed)
        // async send：如果 channel 满则 await 等待，不阻塞线程
    }
}
```

**对比表：**

| 特性 | std::sync::mpsc | tokio::sync::mpsc |
|------|----------------|-------------------|
| 阻塞模型 | 阻塞 OS 线程 | 不阻塞线程（await 让出执行权） |
| buffer | 无界（默认） | 有界（需指定容量） |
| send 满时 | 立即返回（无界） | await 等待（背压） |
| recv 空时 | 阻塞线程 | await 挂起 task |
| 适用场景 | OS 线程 | async task |
| 关闭检测 | `for op in rx` 迭代结束 | `rx.recv().await` 返回 `None` |

**设计要点：**

- **mpsc** = Multiple Producer, Single Consumer：多个发送者，一个消费者
- **channel 关闭**：所有 `Sender` drop 后，`Receiver` 的迭代/recv 会自然结束
- **有界 vs 无界**：异步 channel 默认有界（背压保护），同步 channel 默认无界（可能内存泄漏）

### 4.6 TTL 过期管理 (`ttl.rs`)

**学习目标：** tokio::spawn、async 定时器、select! 多路复用、优雅关闭

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct TtlManager {
    shutdown: mpsc::Sender<()>,
}

impl TtlManager {
    /// 启动后台 TTL 清理任务
    pub fn spawn(
        store: Arc<Mutex<HashMap<String, Entry>>>,
        check_interval: Duration,
    ) -> Self {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        tokio::spawn(async move {
            loop {
                // select! 同时监听定时器和关闭信号，先触发的分支执行
                tokio::select! {
                    _ = tokio::time::sleep(check_interval) => {
                        // 定时器到期：清理过期键
                        let mut store = store.lock().unwrap();
                        let now = Instant::now();
                        // retain：只保留未过期的 entry
                        store.retain(|_, entry| {
                            entry.expires_at.map_or(true, |exp| now < exp)
                        });
                    }
                    _ = shutdown_rx.recv() => {
                        // 收到关闭信号：退出循环
                        break;
                    }
                }
            }
        });

        Self { shutdown: shutdown_tx }
    }

    /// 通知后台任务停止（优雅关闭）
    pub async fn shutdown(self) {
        let _ = self.shutdown.send(()).await;
    }
}
```

**设计要点：**

- **tokio::spawn**：启动一个异步任务，类似 `thread::spawn` 但不创建 OS 线程，而是在 tokio runtime 上调度
- **tokio::time::sleep**：异步等待，不阻塞线程。在等待期间 tokio 可以执行其他 task
- **select!**：同时等待多个异步事件。这里同时监听「定时器到期」和「关闭信号」，先触发的分支执行
- **优雅关闭**：通过 channel 发送关闭信号，让后台任务自行退出，避免强制 abort（可能导致数据不一致）
- **retain**：`HashMap::retain` 保留满足条件的元素，删除不满足的——比手动遍历删除更高效

### 4.7 自定义 Iterator (`scanner.rs`)

**学习目标：** Iterator trait、关联类型、迭代器组合链

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;

/// 范围扫描迭代器：遍历指定前缀的键值对
pub struct ScanIterator {
    items: Vec<(String, Value)>,
    index: usize,
}

impl ScanIterator {
    pub fn new(
        store: &Arc<Mutex<HashMap<String, Entry>>>,
        prefix: &str,
    ) -> Result<Self, KvError> {
        let store = store.lock()?;
        let now = Instant::now();
        let mut items: Vec<(String, Value)> = store.iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .filter(|(_, e)| !e.is_expired(now))
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));  // 按键名排序
        Ok(Self { items, index: 0 })
    }
}

/// 实现 Iterator trait：这是 Rust 迭代器的核心
/// 关联类型 type Item = (String, Value) 决定迭代器产出的元素类型
impl Iterator for ScanIterator {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.items.len() {
            let item = self.items[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None  // 返回 None 表示迭代结束
        }
    }
}

// 使用方式：
// for (key, value) in engine.scan("user:")? {
//     println!("{key} => {value}");
// }
//
// 迭代器组合链（实现 Iterator 后自动获得）：
// let results: Vec<String> = engine.scan("user:")?
//     .filter(|(k, _)| k.contains("admin"))
//     .map(|(k, _)| k)
//     .collect();
```

**设计要点：**

- **`type Item`**：关联类型（associated type），不同于泛型参数。一个类型只能有一个 `Iterator` 实现，保证「一对一」关系
- **`next()` 方法**：返回 `Option<Self::Item>`，`Some` 表示还有元素，`None` 表示结束
- **for 循环语法糖**：`for x in iter` 等价于 `while let Some(x) = iter.next()`
- **零成本抽象**：实现 `Iterator` 后自动获得 `.map()`, `.filter()`, `.collect()`, `.sum()` 等方法，编译期单态化，无运行时开销

### 4.8 运算符重载 (`models/value.rs` 扩展)

**学习目标：** std::ops trait、运算符重载模式

```rust
use std::ops::Add;
use crate::error::KvError;

impl Add for Value {
    /// Add trait 的关联类型：运算结果的类型
    /// 这里返回 Result 以处理类型不匹配的情况
    type Output = Result<Value, KvError>;

    /// Value + Value：合并操作
    /// - String + String = 拼接
    /// - Integer + Integer = 累加
    /// - List + List = 拼接
    /// - Hash + Hash = 覆盖同名 key
    /// - 类型不匹配 → Err
    fn add(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => {
                Ok(Value::Integer(a + b))
            }
            (Value::String(a), Value::String(b)) => {
                Ok(Value::String(format!("{a}{b}")))
            }
            (Value::List(mut a), Value::List(b)) => {
                a.extend(b);  // LinkedList 拼接
                Ok(Value::List(a))
            }
            (Value::Hash(mut a), Value::Hash(b)) => {
                a.extend(b);  // HashMap 覆盖同名 key
                Ok(Value::Hash(a))
            }
            (l, r) => Err(KvError::TypeMismatch {
                left: l.type_name(),
                right: r.type_name(),
            }),
        }
    }
}

// 使用：
// let merged = Value::from(10) + Value::from(20)?;  // → Integer(30)
// let merged = Value::from("Hello, ") + Value::from("World")?;  // → String("Hello, World")
```

**设计要点：**

- **`type Output`**：运算符重载必须定义输出类型。这里返回 `Result` 以处理类型不匹配
- **消费所有权**：`fn add(self, rhs)` 消费两个操作数。如果想保留原值，需要 clone 后传入，或实现 `Add<&Value> for &Value`
- **Rust 的运算符重载**：通过实现 `std::ops` 下的 trait 实现（`Add`=`+`，`Sub`=`-`，`Mul`=`*`，`Index`=`[]` 等）
- **不能创建新运算符**：只能重载已有的运算符，且操作数数量固定

### 4.9 声明式宏 (`macros.rs`)

**学习目标：** macro_rules!、片段说明符、宏卫生性

```rust
/// kv! 宏：快速 PUT 多个键值对
/// 用法：kv!(engine, "name" => "RustKV", "version" => "0.1.0");
#[macro_export]
macro_rules! kv {
    // $engine:expr —— 匹配一个表达式，绑定为 engine
    // $($key:expr => $val:expr),* —— 匹配零或多个 "key => value"，逗号分隔
    // $(,)? —— 允许尾随逗号
    ($engine:expr, $($key:expr => $val:expr),* $(,)?) => {
        {
            $(
                $engine.put($key, $val.into(), None);
            )*
            // $()* 重复展开：每个 key-value 对生成一个 put 调用
        }
    };
}

/// log_kv! 宏：带时间戳的日志输出
/// 用法：log_kv!("PUT", "name", "RustKV");
#[macro_export]
macro_rules! log_kv {
    ($op:expr, $key:expr, $val:expr) => {
        println!("[{}] {}: {} = {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            $op, $key, $val);
    };
}

// 使用：
// kv!(engine, "name" => "RustKV", "count" => 42i64);
// log_kv!("PUT", "name", "RustKV");
```

**宏语法解析：**

| 语法 | 含义 |
|------|------|
| `$name:expr` | 匹配一个表达式，绑定为 `name` |
| `$($x:tt),*` | 匹配零或多个 token tree，逗号分隔 |
| `$(),* $(,)?` | 支持尾随逗号 |
| `$($...)*` | 重复展开 |
| `#[macro_export]` | 导出宏供外部 crate 使用 |

**片段说明符参考：**

| 说明符 | 匹配内容 | 示例 |
|--------|---------|------|
| `expr` | 表达式 | `1 + 2`, `foo()` |
| `tt` | 单个 token tree | 任何语法单元 |
| `ident` | 标识符 | `foo`, `bar` |
| `ty` | 类型 | `String`, `&'a str` |
| `block` | 代码块 | `{ ... }` |
| `literal` | 字面量 | `42`, `"hello"` |
| `pat` | 模式 | `Some(x)` |

**设计要点：**

- **宏 vs 函数**：宏在编译期展开，可以做函数做不到的事（如变长参数、生成不同代码）；但宏更难调试
- **卫生性**：宏引入的变量不会与调用方的变量冲突——这是 Rust 宏的安全特性
- **`#[macro_export]`**：让宏可以在外部 crate 中通过 `use crate_name::kv;` 导入使用

---

## 5. 错误处理 (`error.rs`)

**复用 myapp 技能：** thiserror + anyhow 模式

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvError {
    #[error("键不存在：{0}")]
    KeyNotFound(String),

    #[error("类型不匹配：{left} + {right}")]
    TypeMismatch { left: &'static str, right: &'static str },

    #[error("WAL 已关闭")]
    WalClosed,

    #[error("存储锁中毒")]
    LockPoisoned,

    #[error("IO 错误：{0}")]
    IoError(#[from] std::io::Error),

    #[error("序列化错误：{0}")]
    SerializeError(#[from] serde_json::Error),

    #[error("数据损坏：{0}")]
    CorruptedData(String),
}

// Mutex 的 PoisonError 转换
impl<T> From<std::sync::PoisonError<T>> for KvError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        KvError::LockPoisoned
    }
}

// KvError → anyhow::Error 自动转换（通过 std::error::Error trait）
// main 层用 anyhow::Result 统一处理
```

**复用要点：** 与 myapp 的 `TaskError` 模式完全一致——`thiserror` 定义业务错误，`#[from]` 自动转换，`anyhow` 在 main 层统一处理。新增 `LockPoisoned` 处理 Mutex 中毒。

---

## 6. 关键技术点说明

### 6.1 为什么 Phase 1 用 RefCell 而非直接 `&mut`？

Engine 的方法签名是 `&self`（不可变引用），但需要修改内部 HashMap。Rust 的所有权规则不允许在不可变引用下修改数据——除非使用内部可变性（`RefCell`）。这是「内部可变性」模式的标准教学案例。

对比：myapp 中的 `TaskService` 用 `&self` + `store: JsonFileStore`（owned），不需要 RefCell，因为每次操作都重新 load/save 文件。RustKV 是内存存储，需要持有可变状态。

### 6.2 为什么 Rc 不能跨线程而 Arc 可以？

`Rc` 的引用计数操作不是原子的（非线程安全），多线程并发修改计数会导致数据竞争。`Arc` 使用原子操作（CAS）更新计数，保证线程安全。编译器通过 `Send` trait 在编译期阻止 `Rc` 跨线程——这就是 Rust 的「无畏并发」。

**实践验证：** T2.5 测试中故意尝试跨线程发送 `Rc`，观察编译器报错信息。

### 6.3 为什么 async channel 需要有界容量？

无界 channel 在生产速度超过消费速度时会导致内存无限增长。有界 channel 满了之后 `send().await` 会挂起生产者，形成背压（backpressure），保护系统稳定。

### 6.4 为什么用自定义 LinkedList 而非 Vec？

Vec 是连续内存数组，在头部插入需要 O(n) 移动元素；LinkedList 在头部插入是 O(1)。但更重要的是：LinkedList 是学习 `Box` 递归类型的最佳练习——Rust 官方文档和教科书都用链表讲解 Box。

### 6.5 Iterator 的关联类型 vs 泛型参数

`Iterator` 用关联类型 `type Item` 而非 `Iterator<T>`，因为一个迭代器只产出一种类型的元素。如果用泛型，同一个类型可以同时实现 `Iterator<String>` 和 `Iterator<i64>`，这在语义上不合理。关联类型强制「一对一」关系。

### 6.6 select! 宏的工作原理

`select!` 宏会同时启动所有分支的 Future，哪个先完成就执行哪个分支，其余分支被丢弃。这在多路复用场景非常有用：同时监听定时器、channel 消息、关闭信号。

---

## 7. 测试策略

### 7.1 单元测试

| 模块 | 测试内容 | 学习重点 |
|------|---------|---------|
| `models/value.rs` | From 转换、Add 运算符、Display | 泛型测试、运算符重载测试 |
| `models/linked_list.rs` | push/pop/len、空链表边界 | Box 递归类型测试 |
| `engine.rs` | put/get/del、TTL 过期、并发写入 | RefCell/Mutex 测试 |
| `wal.rs` | 写入 + 恢复、channel 关闭 | 多线程测试 |
| `scanner.rs` | 前缀匹配、空结果、迭代器组合 | Iterator 测试 |
| `macros.rs` | kv! 宏展开正确 | 宏测试 |

### 7.2 集成测试

| 场景 | 测试方式 |
|------|---------|
| CLI 正常路径 | assert_cmd（复用 myapp 模式） |
| 并发写入一致性 | 多线程 + 验证键数量 |
| TTL 过期 | tokio::time::sleep + 验证 None |
| WAL 恢复 | 写入 → 重新加载 → 验证数据 |
| 快照恢复 | 持久化 → 重新加载 → 验证数据 |
| MERGE 运算符 | 不同类型合并 + 类型不匹配错误 |

### 7.3 基准测试（Phase 5 可选）

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_put(c: &mut Criterion) {
    let engine = Engine::new(Config::default());
    c.bench_function("put", |b| {
        b.iter(|| {
            engine.put(black_box("key"), Value::from("value"), None);
        })
    });
}

criterion_group!(benches, bench_put);
criterion_main!(benches);
```

---

## 8. 学习要点完整映射表

| Rust 概念 | RustKV 中的位置 | 阶段 | 学习深度 |
|-----------|----------------|------|---------|
| 泛型函数/结构体 | `Value`、`Engine` | Phase 1 | ★★★ |
| 显式生命周期 | `Engine::get` 签名设计 | Phase 1 | ★★ |
| `Box` 智能指针 | `LinkedList::Node` | Phase 1 | ★★★ |
| `Rc` 引用计数 | `Engine::config` | Phase 1 | ★★ |
| `RefCell` 内部可变性 | `Engine::store` (P1) | Phase 1 | ★★★ |
| `Arc` 线程安全引用 | `Engine::store` (P2) | Phase 2 | ★★★ |
| `Mutex` 互斥锁 | `Engine::store` (P2) | Phase 2 | ★★★ |
| `std::thread` | `Wal` 后台线程 | Phase 2 | ★★★ |
| `mpsc` Channel | `Wal` 写队列 | Phase 2 | ★★★ |
| `Send`/`Sync` | 并发测试验证 | Phase 2 | ★★ |
| `async`/`await` | TTL、WAL 异步版 | Phase 3 | ★★★ |
| tokio runtime | `#[tokio::main]` | Phase 3 | ★★★ |
| `tokio::spawn` | TTL 清理任务 | Phase 3 | ★★★ |
| `tokio::time` | TTL 定时器 | Phase 3 | ★★ |
| `tokio::fs` | 异步 WAL 写入 | Phase 3 | ★★ |
| `tokio::sync::mpsc` | 异步 channel | Phase 3 | ★★★ |
| `select!` 宏 | TTL 多路复用 | Phase 3 | ★★★ |
| 自定义 `Iterator` | `ScanIterator` | Phase 4 | ★★★ |
| 关联类型 | `Iterator::Item` | Phase 4 | ★★ |
| 运算符重载 | `Value: Add` | Phase 4 | ★★★ |
| `Drop` trait | `Engine::drop` | Phase 4 | ★★ |
| `From`/`Into` 手动实现 | `Value: From<&str>` | Phase 1 | ★★ |
| `macro_rules!` | `kv!` / `log_kv!` | Phase 4 | ★★ |

> ★ = 了解概念，★★ = 能独立实现，★★★ = 深入理解并能灵活运用
