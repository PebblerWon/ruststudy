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

- [x] **T1.1 项目初始化** ✅
  - `cargo init --lib` 创建 library 项目
  - 配置 `Cargo.toml`（tokio/clap 先注释，后续阶段启用）
  - 建立项目目录结构
  - **产出：** 可编译的空项目骨架
  - **技术方案：** 见 [TECH_SOLUTION.md § 1.3](docs/TECH_SOLUTION.md)

- [x] **T1.2 Value 类型定义** ✅
  - 定义泛型 `Value` 枚举：String / Integer / List / Hash
  - 为 `Value` 实现 `Display` trait
  - 手动实现 `From<&str>` / `From<i64>` / `From<String>`（非 derive）
  - **学习点：** 泛型枚举、From/Into trait 手动实现
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.1](docs/TECH_SOLUTION.md)

- [x] **T1.3 LinkedList 实现** ✅
  - 用 `Box<Node>` 实现递归链表类型
  - 实现 `push` / `pop` / `len` / `is_empty` 方法
  - 实现 `Display` trait
  - **学习点：** Box 智能指针、递归类型、为何递归类型必须用 Box
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.2](docs/TECH_SOLUTION.md)

- [x] **T1.4 Engine 基础（单线程版）** ✅
  - 定义 `Engine` 结构体，持有 `RefCell<HashMap<String, Entry>>`
  - 实现 `put` / `get` / `del` / `keys` 方法（返回 Result）
  - 用 `Rc<Config>` 共享配置
  - 定义 `Entry` 结构体（含 TTL 字段）
  - **学习点：** RefCell 内部可变性、Rc 引用计数、生命周期限制
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.4](docs/TECH_SOLUTION.md)（Entry 见 § 4.3）
  - **完成摘要：** 已落地并演进至 Phase 2（RefCell→Arc/Mutex、Rc→Arc，方法统一返回 Result）

- [x] **T1.5 单元测试** ✅
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

- [x] **T2.1 Engine 线程安全改造** ✅
  - 将 `RefCell<HashMap>` 改为 `Arc<Mutex<HashMap>>`
  - 所有操作方法加锁：`let store = self.store.lock()?;`
  - 理解 Mutex 的 RAII 锁释放
  - **学习点：** Arc 引用计数（线程安全版）、Mutex 互斥锁
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.5](docs/TECH_SOLUTION.md)
  - **完成摘要：** 已落地，`get`/`len` 等统一返回 `Result`，锁中毒经 `KvError::LockPoisoned` 传播

- [x] **T2.2 WAL 写前日志** ✅
  - 定义 `WalOp` 枚举（Put / Del）
  - 用 `mpsc::channel` 创建写操作队列
  - 后台线程消费 channel，写入 WAL 文件
  - **学习点：** std::thread::spawn、mpsc channel、生产者-消费者模式、Drop 优雅关闭
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.6](docs/TECH_SOLUTION.md)
  - **完成摘要：** 已落地并集成进 Engine（`create_dir_all` + `data_dir/wal.log`）；`append` 错误经 `?` 传播；实现 `Drop` 优雅关闭（`Option<Sender>` + `take()` 先关 channel 再 `join`，规避 drop-order 死锁），cargo test 全过

- [ ] **T2.3 后台快照线程** ❌ 已取消

  > 快照功能不做（TECH_SOLUTION 未提供快照方案，PRD 已移除 F8）。持久化收敛为「WAL + 启动回放」；JoinHandle 学习点已由 WAL 后台任务覆盖。

- [ ] **T2.4 启动恢复（仅 WAL 回放）**
  - 启动时回放 WAL 日志恢复状态
  - 处理 TTL 语义：按 Put 记录中的 ttl_secs 重算 expires_at
  - **学习点：** 错误恢复、文件 IO 组合
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.6](docs/TECH_SOLUTION.md)

- [ ] **T2.5 并发测试** 🔵
  - 多线程并发 PUT/GET 测试 ✅（test_concurrent_put_with_wal：并发 WAL 追加 + 并发插入）
  - 验证数据一致性 ✅（断言键数量）
  - 尝试跨线程发送 `Rc`（观察编译器报错，理解 Send/Sync）——待做
  - **学习点：** Send/Sync trait、并发测试技巧
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.5](docs/TECH_SOLUTION.md)

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

- [x] **T3.1 tokio 集成** ✅
  - 启用 tokio 依赖（features = ["full"]）✅
  - 将 WAL 后台线程改为 `tokio::spawn` + async task ✅（wal_tokio.rs）
  - `#[tokio::main]` 标注 main——随 T5 CLI 落地（main.rs 尚未接入）
  - **学习点：** async/await 基础、Future trait、async fn
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.5](docs/TECH_SOLUTION.md)（async 版对比）

- [x] **T3.2 TTL 过期管理** ✅
  - PUT 时记录 `expires_at` ✅
  - `tokio::spawn` 后台定时清理任务（`tokio::time::sleep` + `select!` 监听关闭信号）✅
  - retain 谓词反转已修复 ✅（TTL 过期测试归入 T3.6）
  - **学习点：** tokio::spawn、异步定时器、async channel
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.7](docs/TECH_SOLUTION.md)

- [x] **T3.3 异步文件 IO** ✅
  - WAL 写入改为 `tokio::fs` 异步 IO ✅（快照写入项随快照取消）
  - **学习点：** tokio::fs、异步文件操作

- [x] **T3.4 异步 Channel** ✅
  - 将 `std::sync::mpsc` 替换为 `tokio::sync::mpsc` ✅（有界 channel 1024，背压）
  - 理解异步 channel 与同步 channel 的区别 ✅（同步版 wal.rs 保留作教学对照）
  - **学习点：** tokio::sync::mpsc、异步生产者-消费者

- [x] **T3.5 select! 多路复用** ✅
  - 用 `tokio::select!` 同时监听：TTL 定时器 + 关闭信号（WAL 优雅关闭经 close() 关 channel 实现）
  - **学习点：** select! 宏、多路复用、优雅关闭
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.7](docs/TECH_SOLUTION.md)

- [ ] **T3.6 异步测试** 🔵
  - 使用 `#[tokio::test]` 编写异步测试 ✅（engine / 并发 WAL / scan 共 3 个）
  - 测试 TTL 过期——待补（T3.2 修复后一并覆盖）
  - **学习点：** 异步测试技巧

- [ ] **T3.7 WAL 持久性升级：L1 ack 式回执**
  - channel 载荷从 `WalOp` 升级为 `(WalOp, oneshot::Sender<Result<(), KvError>>)`
  - 后台任务写盘完成后才回执，`append` 挂起直到回执
  - 补充测试：`put` 返回 Ok 后 wal.log 立即包含该 op（无需 close）
  - **学习点：** oneshot channel、确认与落盘绑定、持久性分级（L0→L3）
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.6](docs/TECH_SOLUTION.md)（持久性分级与 L1 实现骨架）

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

- [x] **T4.1 自定义 Iterator** ✅
  - 实现 `ScanIterator` 结构体，遍历指定前缀的键值对 ✅
  - 为其实现 `Iterator` trait（`type Item` 关联类型）✅
  - 支持 `.filter().map().collect()` 组合 ✅（test_scan 验证）
  - **学习点：** Iterator trait、关联类型、迭代器组合链
  - **技术方案：** 见 [TECH_SOLUTION.md § 4.8](docs/TECH_SOLUTION.md)

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
  - 使用 clap derive 定义子命令（PUT/GET/DEL/KEYS/SCAN/MERGE/STATS）
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

| 任务 | 状态 | 学习点                     | 备注                                       |
| ---- | ---- | -------------------------- | ------------------------------------------ |
| T1.1 | ✅   | cargo init --lib           | 可编译骨架已建立                           |
| T1.2 | ✅   | 泛型枚举、From/Into        | Display + 3 个 From 手动实现               |
| T1.3 | ✅   | Box 递归类型               | push/pop/len/is_empty/Display              |
| T1.4 | ✅   | RefCell、Rc、生命周期      | 已演进至 Phase 2（Arc/Mutex）              |
| T1.5 | ✅   | 单元测试                   | 3 个测试全过                               |
| T2.1 | ✅   | Arc、Mutex                 | Result 统一错误传播，锁中毒经 KvError 传播 |
| T2.2 | ✅   | thread、mpsc channel       | Drop 优雅关闭，规避 drop-order 死锁        |
| T2.3 | ❌   | JoinHandle                 | 已取消：快照功能不做                       |
| T2.4 | ⬜   | 错误恢复                   | 范围调整为仅 WAL 回放                      |
| T2.5 | 🔵   | Send/Sync                  | 并发 WAL 测试已有，Rc 跨线程实验未做       |
| T3.1 | ✅   | async/await、tokio         | Engine async 化，WAL 迁移至 wal_tokio      |
| T3.2 | ✅   | tokio::spawn、定时器       | 谓词反转已修复；过期测试归入 T3.6          |
| T3.3 | ✅   | tokio::fs                  | WAL 异步写入                               |
| T3.4 | ✅   | tokio::sync::mpsc          | 有界 channel（1024）                       |
| T3.5 | ✅   | select! 宏                 | TtlManager 多路复用                        |
| T3.6 | 🔵   | 异步测试                   | 3 个 #[tokio::test]，TTL 过期测试缺失      |
| T3.7 | ⬜   | oneshot channel、ack 回执  | ack 式 WAL，方案见 TECH_SOLUTION §4.6      |
| T4.1 | ✅   | Iterator、关联类型         | ScanIterator + engine.scan                 |
| T4.2 | ⬜   | std::ops、运算符重载       |                                            |
| T4.3 | ⬜   | Drop trait                 |                                            |
| T4.4 | ⬜   | macro_rules!               |                                            |
| T5.1 | ⬜   | clap（复用）               |                                            |
| T5.2 | ⬜   | thiserror + anyhow（复用） |                                            |
| T5.3 | ⬜   | 集成测试（复用）           |                                            |
| T5.4 | ⬜   | criterion 基准测试         |                                            |

**状态说明：** ⬜ 待开始 | 🔵 进行中 | ✅ 已完成 | ⏸ 暂停 | ❌ 已取消
