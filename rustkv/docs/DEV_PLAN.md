# RustKV 开发计划

## 总体安排

- **总工期：** 4 周
- **阶段划分：** 5 个阶段（数据结构 → 并发 → 异步 → 高级 trait → CLI 集成）
- **学习导向：** 每个阶段引入 2-3 个新的 Rust 核心概念，代码即练习
- **演进路线：** 同一存储引擎经历 RefCell → Arc<Mutex> → async 三次重构

---

## 阶段一：基础数据结构与泛型（第 1 周）

> 目标：掌握泛型、Box 递归类型、Rc/RefCell 智能指针
> 补强 myapp：泛型、生命周期、智能指针、内部可变性

### 任务清单

- [ ] **T1.1 项目初始化**
  - `cargo init --lib` 创建 library 项目
  - 配置 `Cargo.toml`（tokio/clap 先注释，后续阶段启用）
  - 建立项目目录结构
  - **产出：** 可编译的空项目骨架
  - **技术方案：** 见 [TECH_SOLUTION.md § 1.3](docs/TECH_SOLUTION.md)

- [ ] **T1.2 Value 类型定义**
  - 定义泛型 `Value` 枚举：String / Integer / List / Hash
  - 为 `Value` 实现 `Display` trait
  - 手动实现 `From<&str>` / `From<i64>` / `From<String>`（非 derive）
  - **学习点：** 泛型枚举、From/Into trait 手动实现
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.1](docs/TECH_SOLUTION.md)

- [ ] **T1.3 LinkedList 实现**
  - 用 `Box<Node>` 实现递归链表类型
  - 实现 `push` / `pop` / `len` / `is_empty` 方法
  - 实现 `Display` trait
  - **学习点：** Box 智能指针、递归类型、为何递归类型必须用 Box
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.2](docs/TECH_SOLUTION.md)

- [ ] **T1.4 Engine 基础（单线程版）**
  - 定义 `Engine` 结构体，持有 `RefCell<HashMap<String, Entry>>`
  - 实现 `put` / `get` / `del` / `keys` 方法（返回 Result）
  - 用 `Rc<Config>` 共享配置
  - 定义 `Entry` 结构体（含 TTL 字段）
  - **学习点：** RefCell 内部可变性、Rc 引用计数、生命周期限制
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.3](docs/TECH_SOLUTION.md)

- [ ] **T1.5 单元测试**
  - 测试各数据类型的 put/get
  - 测试 LinkedList 操作边界
  - 测试 From/Into 转换正确性
  - **产出：** 基础数据层可用，单元测试通过

**阶段一验收：**

```rust
let engine = Engine::new(Config::default());
engine.put("name", Value::from("RustKV"), None);
assert_eq!(engine.get("name"), Some(Value::String("RustKV".into())));
engine.put("count", Value::from(42i64), None);
engine.del("name");
assert_eq!(engine.len(), 1);
```

---

## 阶段二：线程安全与并发（第 2 周）

> 目标：掌握多线程、Mutex/Arc、mpsc Channel、Send/Sync
> 补强 myapp：多线程、并发原语

### 任务清单

- [ ] **T2.1 Engine 线程安全改造**
  - 将 `RefCell<HashMap>` 改为 `Arc<Mutex<HashMap>>`
  - 所有操作方法加锁：`let store = self.store.lock()?;`
  - 理解 Mutex 的 RAII 锁释放
  - **学习点：** Arc 引用计数（线程安全版）、Mutex 互斥锁
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.4](docs/TECH_SOLUTION.md)

- [ ] **T2.2 WAL 写前日志**
  - 定义 `WalOp` 枚举（Put / Del）
  - 用 `mpsc::channel` 创建写操作队列
  - 后台线程消费 channel，写入 WAL 文件
  - **学习点：** std::thread::spawn、mpsc channel、生产者-消费者模式
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.5](docs/TECH_SOLUTION.md)

- [ ] **T2.3 后台快照线程**
  - 定时触发全量快照（thread::sleep + 循环）
  - 快照序列化为 JSON 文件
  - WAL 文件截断
  - **学习点：** JoinHandle、线程生命周期管理

- [ ] **T2.4 启动恢复**
  - 加载最新快照文件
  - 回放 WAL 日志恢复状态
  - **学习点：** 错误恢复、文件 IO 组合

- [ ] **T2.5 并发测试**
  - 多线程并发 PUT/GET 测试
  - 验证数据一致性
  - 尝试跨线程发送 `Rc`（观察编译器报错，理解 Send/Sync）
  - **学习点：** Send/Sync trait、并发测试技巧
  - **技术方案：** 见 [TECH_SOLUTION.md § 5.2](docs/TECH_SOLUTION.md)

**阶段二验收：**

```rust
let engine = Arc::new(Engine::new(Config::default())?);
let mut handles = vec![];
for i in 0..10 {
    let e = Arc::clone(&engine);
    handles.push(thread::spawn(move || {
        e.put(&format!("key-{i}"), Value::from(i as i64), None).unwrap();
    }));
}
for h in handles { h.join().unwrap(); }
assert_eq!(engine.len()?, 10);
```

---

## 阶段三：异步重构（第 3 周）

> 目标：掌握 async/await、tokio 运行时、异步 channel、select!
> 补强 myapp：异步编程（最大缺口）

### 任务清单

- [ ] **T3.1 tokio 集成**
  - 启用 tokio 依赖（features = ["full"]）
  - `#[tokio::main]` 标注 main
  - 将 WAL 后台线程改为 `tokio::spawn` + async task
  - **学习点：** async/await 基础、Future trait、async fn
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.5](docs/TECH_SOLUTION.md)（async 版对比）

- [ ] **T3.2 TTL 过期管理**
  - PUT 时记录 `expires_at`
  - `tokio::spawn` 后台定时清理任务
  - `tokio::time::sleep` + 循环
  - **学习点：** tokio::spawn、异步定时器、async channel
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.6](docs/TECH_SOLUTION.md)

- [ ] **T3.3 异步文件 IO**
  - WAL 写入改为 `tokio::fs` 异步 IO
  - 快照写入改为异步
  - **学习点：** tokio::fs、异步文件操作

- [ ] **T3.4 异步 Channel**
  - 将 `std::sync::mpsc` 替换为 `tokio::sync::mpsc`
  - 理解异步 channel 与同步 channel 的区别
  - **学习点：** tokio::sync::mpsc、异步生产者-消费者

- [ ] **T3.5 select! 多路复用**
  - 用 `tokio::select!` 同时监听：写操作 channel + 关闭信号 + 定时器
  - 优雅关闭 WAL 后台任务
  - **学习点：** select! 宏、多路复用、优雅关闭
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.6](docs/TECH_SOLUTION.md)

- [ ] **T3.6 异步测试**
  - 使用 `#[tokio::test]` 编写异步测试
  - 测试 TTL 过期、并发写入
  - **学习点：** 异步测试技巧

**阶段三验收：**

```rust
#[tokio::main]
async fn main() {
    let engine = Arc::new(Engine::new(Config::default())?);
    engine.put("key", Value::from("value"), Some(Duration::from_secs(1))).await?;
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_eq!(engine.get("key").await?, None);  // TTL 过期
}
```

---

## 阶段四：高级 Trait 与宏（第 3-4 周）

> 目标：掌握自定义 Iterator、运算符重载、Drop trait、声明式宏
> 补强 myapp：高级 trait、运算符重载、Drop、宏

### 任务清单

- [ ] **T4.1 自定义 Iterator**
  - 实现 `ScanIterator` 结构体，遍历指定前缀的键值对
  - 为其实现 `Iterator` trait（`type Item` 关联类型）
  - 支持 `.filter().map().collect()` 组合
  - **学习点：** Iterator trait、关联类型、迭代器组合链
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.7](docs/TECH_SOLUTION.md)

- [ ] **T4.2 运算符重载**
  - 为 `Value` 实现 `std::ops::Add`（合并操作）
  - String + String = 拼接，Integer + Integer = 累加，List + List = 拼接
  - 实现 `MERGE` 命令
  - **学习点：** std::ops trait、运算符重载模式
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.8](docs/TECH_SOLUTION.md)

- [ ] **T4.3 Drop trait**
  - 为 `Engine` 实现 `Drop` trait
  - drop 时：flush WAL、等待后台线程退出、打印统计
  - **学习点：** RAII、Drop trait、资源生命周期

- [ ] **T4.4 声明式宏**
  - 编写 `kv!` 宏：`kv!(engine, "key" => "value")` 语法糖
  - 编写 `log_kv!` 宏：带时间戳的日志输出
  - **学习点：** macro_rules!、宏语法、片段说明符
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.9](docs/TECH_SOLUTION.md)

**阶段四验收：**

```rust
// 运算符重载
let v1 = Value::from(10i64);
let v2 = Value::from(20i64);
assert_eq!(v1 + v2, Ok(Value::from(30i64)));

// 自定义 Iterator
for (key, value) in engine.scan("user:") {
    println!("{key} => {value}");
}

// 宏
kv!(engine, "name" => "RustKV", "count" => 42i64);
```

---

## 阶段五：CLI 与集成（第 4 周）

> 目标：整合所有模块，复用 myapp 的 CLI 技能，编写集成测试
> 复用 myapp：clap、serde、anyhow/thiserror、测试

### 任务清单

- [ ] **T5.1 CLI 命令定义**
  - 使用 clap derive 定义子命令（PUT/GET/DEL/KEYS/SCAN/MERGE/STATS/PERSIST）
  - 复用 myapp 的 CLI 设计模式
  - **复用技能：** clap derive、子命令设计

- [ ] **T5.2 错误处理**
  - 用 thiserror 定义 `KvError` 枚举
  - 用 anyhow 在 main 中统一处理
  - **复用技能：** thiserror + anyhow、生产零 unwrap

- [ ] **T5.3 集成测试**
  - 测试所有 CLI 命令的正常/异常路径
  - 并发写入测试
  - TTL 过期测试
  - **复用技能：** assert_cmd、tempfile、测试隔离

- [ ] **T5.4 基准测试（可选）**
  - 使用 criterion crate 编写基准测试
  - 测试单线程 vs 多线程 vs 异步性能对比
  - **学习点：** 性能测试、criterion

**阶段五验收：**

```bash
cargo test                    # 全部通过
rustkv put name RustKV
rustkv get name
rustkv put count 10
rustkv merge count 5          # → 15
rustkv scan "na"              # 前缀扫描
rustkv stats
```

---

## 进度追踪

| 任务 | 状态 | 学习点 | 备注 |
|------|------|--------|------|
| T1.1 | ⬜ | cargo init --lib | |
| T1.2 | ⬜ | 泛型枚举、From/Into | |
| T1.3 | ⬜ | Box 递归类型 | |
| T1.4 | ⬜ | RefCell、Rc、生命周期 | |
| T1.5 | ⬜ | 单元测试 | |
| T2.1 | ⬜ | Arc、Mutex | |
| T2.2 | ⬜ | thread、mpsc channel | |
| T2.3 | ⬜ | JoinHandle | |
| T2.4 | ⬜ | 错误恢复 | |
| T2.5 | ⬜ | Send/Sync | |
| T3.1 | ⬜ | async/await、tokio | |
| T3.2 | ⬜ | tokio::spawn、定时器 | |
| T3.3 | ⬜ | tokio::fs | |
| T3.4 | ⬜ | tokio::sync::mpsc | |
| T3.5 | ⬜ | select! 宏 | |
| T3.6 | ⬜ | 异步测试 | |
| T4.1 | ⬜ | Iterator、关联类型 | |
| T4.2 | ⬜ | std::ops、运算符重载 | |
| T4.3 | ⬜ | Drop trait | |
| T4.4 | ⬜ | macro_rules! | |
| T5.1 | ⬜ | clap（复用） | |
| T5.2 | ⬜ | thiserror + anyhow（复用） | |
| T5.3 | ⬜ | 集成测试（复用） | |
| T5.4 | ⬜ | criterion 基准测试 | |

**状态说明：** ⬜ 待开始 | 🔵 进行中 | ✅ 已完成 | ⏸ 暂停
