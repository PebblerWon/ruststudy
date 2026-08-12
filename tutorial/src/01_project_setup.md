# 第 1 章：项目初始化与架构设计

## 本章目标

- 用 `cargo new` 创建项目
- 配置所有依赖（Cargo.toml）
- 建立项目目录结构
- 理解分层架构设计

## 1.1 创建项目

```bash
cargo new taskflow
cd taskflow
```

这会生成：

```
taskflow/
├── Cargo.toml
├── src/
│   └── main.rs
└── target/
```

## 1.2 配置 Cargo.toml

打开 `Cargo.toml`，添加本项目需要的所有依赖：

```toml
[package]
name = "taskflow"
version = "0.1.0"
edition = "2021"
description = "A command-line task management tool"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
colored = "2"
comfy-table = "7"
csv = "1"
anyhow = "1"
thiserror = "1"
dirs = "5"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

### 依赖速查表

| 库 | 用途 | 为什么需要 |
|----|------|-----------|
| `clap` | 命令行参数解析 | 自动生成 `--help`、参数校验、子命令 |
| `serde` | 序列化框架 | Rust 结构体 ↔ JSON 的自动转换 |
| `serde_json` | JSON 读写 | 数据持久化到 JSON 文件 |
| `chrono` | 日期时间 | 任务创建时间、截止日期 |
| `uuid` | UUID 生成 | 每个任务的唯一标识 |
| `colored` | 终端颜色 | 成功/错误/警告的彩色输出 |
| `comfy-table` | 终端表格 | 任务列表的表格渲染 |
| `csv` | CSV 导出 | 将任务导出为 Excel 可读的 CSV |
| `anyhow` | 应用层错误 | 顶层统一错误处理和传播 |
| `thiserror` | 库层错误 | 定义业务错误类型 |
| `dirs` | 目录路径 | 跨平台获取 `~` 主目录 |

> **新手提示**：`features = ["derive"]` 启用宏自动派生功能，忘记开启是常见错误。

## 1.3 建立目录结构

创建以下文件和目录：

```
taskflow/
├── Cargo.toml
├── src/
│   ├── main.rs             # 程序入口
│   ├── cli.rs              # CLI 参数定义
│   ├── models/
│   │   ├── mod.rs          # 模块导出
│   │   ├── task.rs         # Task 结构体
│   │   └── enums.rs        # Status、Priority 枚举
│   ├── store.rs            # 存储层
│   ├── service.rs          # 业务逻辑层
│   ├── display.rs          # 展示层
│   └── error.rs            # 错误类型
└── tests/
    └── cli_test.rs         # 集成测试
```

可以先创建空文件，每个文件写一行注释占位：

```rust
// src/cli.rs
// TODO: CLI 参数定义

// src/error.rs
// TODO: 错误类型

// src/models/mod.rs
// TODO: 模块导出

// src/models/task.rs
// TODO: Task 结构体

// src/models/enums.rs
// TODO: 枚举定义

// src/store.rs
// TODO: 存储层

// src/service.rs
// TODO: 业务逻辑

// src/display.rs
// TODO: 展示层
```

## 1.4 配置 main.rs 的模块声明

在 `src/main.rs` 中声明所有模块：

```rust
mod cli;
mod display;
mod error;
mod models;
mod service;
mod store;

fn main() {
    println!("TaskFlow 启动成功！");
}
```

运行验证：

```bash
cargo run
# 输出: TaskFlow 启动成功！
```

## 1.5 理解分层架构

本项目采用经典的**分层架构**，每层职责单一，上层调用下层：

```
┌──────────────────────────────────────────┐
│               CLI 层                      │
│  cli.rs（参数定义）+ main.rs（调度）       │
│  职责：解析参数 → 调用 Service → 展示结果  │
├──────────────────────────────────────────┤
│               业务逻辑层                   │
│  service.rs                              │
│  职责：数据校验、业务规则、组合存储操作      │
├──────────────────────────────────────────┤
│               存储层                      │
│  store.rs                                │
│  职责：数据持久化，JSON 文件读写           │
├──────────────────────────────────────────┤
│               数据层                      │
│  models/                                 │
│  职责：定义数据结构（Task、Status 等）      │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│               展示层（独立）               │
│  display.rs                              │
│  职责：表格渲染、颜色输出                   │
└──────────────────────────────────────────┘
```

### 数据流

```
用户输入命令
    ↓
cli.rs: clap 解析参数 → Commands 枚举
    ↓
main.rs: match Commands → 调用 service 对应方法
    ↓
service.rs: 校验参数 → 调用 store → 返回结果
    ↓
store.rs: 读写 JSON 文件 → 返回 Vec<Task>
    ↓
display.rs: 格式化输出到终端
```

### 为什么要分层？

| 好处 | 说明 |
|------|------|
| **职责清晰** | 每个文件只做一件事，容易定位问题 |
| **可测试** | 存储层可以用内存 mock 替换，不需要真实文件 |
| **可替换** | 未来换 SQLite 只需改 `store.rs`，其他层不动 |
| **易理解** | 新手可以逐层阅读，不用一次理解全部代码 |

## 1.6 验证

```bash
cargo build
```

确保编译通过、无 warning。

## 本章小结

- 创建了项目并配置了 11 个依赖 + 3 个开发依赖
- 建立了 8 个源文件的目录结构
- 理解了分层架构：CLI → Service → Store → Models，Display 独立

---

[← 返回教程目录](./00_overview.md) | [下一章：数据模型 →](./02_data_model.md)

---

📧 联系作者：pebblerwon@qq.com
