# RustKV 产品需求说明书（PRD）

## 1. 项目概述

**产品名称：** RustKV — 嵌入式并发键值存储引擎

**产品定位：** 一个轻量级的本地键值存储引擎，提供多数据类型、TTL、WAL 持久化和并发访问能力。作为 Rust 进阶学习项目，刻意覆盖泛型、智能指针、多线程、异步编程、高级 trait 和宏等 myapp 未涉及的核心特性。

**核心价值：**
- 嵌入式库 + CLI 双模式，无需独立部署服务
- 支持多数据类型（字符串、整数、列表、哈希）
- 线程安全 + 异步双模式并发
- WAL 持久化 + 快照双保障

**学习定位：**
本项目是 myapp (TaskFlow) 的进阶延续。myapp 覆盖了 CLI 开发、serde、错误处理、测试等基础技能；RustKV 刻意设计用于填补泛型、智能指针、并发、异步、高级 trait、宏等缺口。

## 2. 目标用户

- Rust 学习者（项目所有者）
- 已完成 CLI 项目（myapp），掌握 struct/enum/trait/serde/clap/anyhow 基础
- 希望补强泛型、智能指针、并发、异步、高级 trait 等进阶技能

## 3. 功能需求

### 3.1 核心功能（P0）

#### F1：基础键值操作
- `PUT <key> <value> [--ttl <seconds>]`：存储键值对，可选 TTL
- `GET <key>`：获取值
- `DEL <key>`：删除键值对
- `KEYS <pattern>`：按前缀匹配列出键名

#### F2：多数据类型
- 字符串（String）：基础键值
- 整数（Integer）：支持数值操作
- 列表（List）：自定义 LinkedList，`LPUSH`/`LPOP`/`LRANGE`
- 哈希（Hash）：`HSET`/`HGET`/`HDEL`

#### F3：WAL 持久化
- 所有写操作写入 Write-Ahead Log
- 后台线程异步刷盘
- 启动时自动恢复
- 支持手动触发快照

### 3.2 增强功能（P1）

#### F4：TTL 过期管理
- `PUT` 时设置 `--ttl <seconds>`
- 后台定时清理过期键
- TTL 过期的键不可 `GET`

#### F5：并发访问
- 线程安全存储（`Arc<Mutex>`）
- 多线程并发读写
- 异步模式（tokio）

#### F6：范围扫描
- `SCAN <prefix>`：前缀扫描
- 自定义 `Iterator` 实现
- 支持游标式分批返回

#### F7：合并操作
- `MERGE <key> <value>`：将新值合并到已有值
- 字符串：追加；整数：累加；列表：拼接；哈希：覆盖同名 field
- 通过运算符重载（`std::ops::Add`）实现

### 3.3 扩展功能（P2）

#### F8：快照与恢复
- `PERSIST`：触发全量快照
- 快照文件序列化为 JSON
- 启动时加载最新快照 + 回放 WAL

#### F9：统计信息
- `STATS`：显示存储概况（键数量、类型分布、WAL 大小）

## 4. 数据模型

### 4.1 Value 类型

```
Value {
    String(String)           // 字符串值
    Integer(i64)             // 整数值
    List(LinkedList)         // 列表（自定义链表，Box 递归类型）
    Hash(HashMap<String, String>)  // 哈希表
}
```

### 4.2 Entry 结构

```
Entry {
    value: Value
    created_at: Instant
    expires_at: Option<Instant>  // None = 永不过期
}
```

### 4.3 存储结构演进

```
Phase 1: RefCell<HashMap>           // 单线程，内部可变性
Phase 2: Arc<Mutex<HashMap>>        // 多线程，线程安全
Phase 3: Arc<Mutex<HashMap>> + tokio // 异步，非阻塞 IO
```

## 5. 非功能需求

| 类别 | 要求 |
|------|------|
| 并发 | 支持多线程并发读写 |
| 持久化 | WAL + 快照双保障 |
| 性能 | 10000 条键值对，PUT < 1ms |
| 跨平台 | Windows / macOS / Linux |
| 错误处理 | 所有操作返回 Result，不 panic |
| 学习覆盖 | 覆盖泛型、智能指针、线程、异步、高级 trait、宏 |

## 6. 约束与假设

- 不涉及网络编程（嵌入式库 + CLI，非 Server）
- 不实现复杂事务
- 数据量上限 100000 条
- 单进程使用，不涉及分布式

## 7. 学习目标映射

| 功能模块 | 覆盖的 Rust 特性 | myapp 是否涉及 |
|---------|-----------------|---------------|
| Value 泛型枚举 | 泛型、enum 高级用法 | 部分（简单 enum） |
| LinkedList | Box、递归类型 | ❌ |
| 配置共享 (Rc) | Rc 引用计数 | ❌ |
| 单线程缓存 (RefCell) | 内部可变性 | ❌ |
| 线程安全存储 | Arc、Mutex | ❌ |
| WAL 后台线程 | std::thread、JoinHandle | ❌ |
| 写操作队列 | mpsc Channel | ❌ |
| TTL 定时清理 | async/await、tokio::spawn | ❌ |
| 异步文件 IO | tokio::fs | ❌ |
| 异步通道 | tokio::sync::mpsc | ❌ |
| 多路复用 | select! 宏 | ❌ |
| 范围扫描 | 自定义 Iterator、关联类型 | ❌ |
| 合并操作 | 运算符重载 (std::ops) | ❌ |
| 类型转换 | From/Into 手动实现 | 部分（#[from] 自动） |
| 资源清理 | Drop trait | ❌ |
| 命令定义 | macro_rules! | ❌ |
| 显式生命周期 | 函数签名标注 | ❌ |
