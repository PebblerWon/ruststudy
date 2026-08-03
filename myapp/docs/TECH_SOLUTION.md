# TaskFlow 技术方案

## 1. 技术栈

### 1.1 核心依赖

| 类别 | 库 | 版本 | 用途 |
|------|------|------|------|
| CLI 解析 | `clap` | 4.x (derive) | 命令行参数解析、子命令定义 |
| 序列化 | `serde` | 1.x | 数据模型序列化框架 |
| JSON | `serde_json` | 1.x | JSON 格式读写 |
| 时间 | `chrono` | 0.4.x | 日期时间处理，带 serde 支持 |
| UUID | `uuid` | 1.x (v4) | 任务唯一 ID 生成 |
| 终端颜色 | `colored` | 2.x | 终端彩色文本输出 |
| 表格 | `comfy-table` | 7.x | 终端表格渲染 |
| CSV | `csv` | 1.x | CSV 文件导出 |
| 错误(应用) | `anyhow` | 1.x | 应用层错误处理 |
| 错误(库) | `thiserror` | 1.x | 库层错误类型定义 |
| 目录 | `dirs` | 5.x | 获取跨平台 home 目录 |

### 1.2 开发依赖

| 类别 | 库 | 版本 | 用途 |
|------|------|------|------|
| 集成测试 | `assert_cmd` | 2.x | CLI 集成测试框架 |
| 断言 | `predicates` | 3.x | 测试断言辅助 |
| 临时目录 | `tempfile` | 3.x | 测试用临时目录 |

### 1.3 Cargo.toml 配置参考

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

---

## 2. 项目结构

```
taskflow/
├── Cargo.toml
├── Cargo.lock
├── docs/
│   ├── PRD.md              # 产品需求说明书
│   ├── DEV_PLAN.md         # 开发计划
│   └── TECH_SOLUTION.md    # 技术方案（本文件）
├── src/
│   ├── main.rs             # 程序入口：解析CLI，调度执行
│   ├── cli.rs              # clap 子命令和参数定义
│   ├── models/
│   │   ├── mod.rs          # 模块导出
│   │   ├── task.rs         # Task 结构体定义
│   │   └── enums.rs        # Status, Priority 枚举
│   ├── store.rs            # 存储层：JSON 文件读写
│   ├── service.rs          # 业务逻辑层：CRUD、搜索、统计
│   ├── display.rs          # 展示层：表格渲染、颜色输出
│   └── error.rs            # 自定义错误类型
└── tests/
    └── cli_test.rs         # CLI 集成测试
```

### 各文件职责

| 文件 | 职责 | 关键类型 |
|------|------|---------|
| `main.rs` | 入口函数，解析 CLI，调用 service，处理错误输出 | `fn main()` |
| `cli.rs` | 定义所有子命令和参数结构 | `Cli`, `Commands` enum |
| `models/task.rs` | Task 数据模型 | `Task` struct |
| `models/enums.rs` | 状态和优先级枚举 | `Status`, `Priority` |
| `store.rs` | 数据持久化，JSON 文件操作 | `Store` trait, `JsonFileStore` |
| `service.rs` | 业务逻辑，数据校验，调用 store | `TaskService` |
| `display.rs` | 终端输出格式化 | `print_task_table()`, `print_stats()` |
| `error.rs` | 错误类型定义 | `TaskError` enum |

---

## 3. 架构设计

### 3.1 分层架构

```
┌─────────────────────────────────────────────────┐
│                    CLI 层                        │
│  cli.rs (参数定义)  +  main.rs (调度)            │
│  职责：解析参数 → 调用 Service → 调用 Display    │
└─────────────────────┬───────────────────────────┘
                      │ 调用
                      ▼
┌─────────────────────────────────────────────────┐
│                  业务逻辑层                       │
│  service.rs                                      │
│  职责：数据校验、业务规则、组合存储操作            │
└─────────────────────┬───────────────────────────┘
                      │ 调用
                      ▼
┌─────────────────────────────────────────────────┐
│                   存储层                         │
│  store.rs                                        │
│  职责：数据持久化，JSON 文件读写                  │
└─────────────────────┬───────────────────────────┘
                      │ 读写
                      ▼
              ┌──────────────┐
              │  JSON 文件    │
              │ ~/.taskflow/  │
              │  data.json   │
              └──────────────┘

┌─────────────────────────────────────────────────┐
│                   展示层                         │
│  display.rs                                      │
│  职责：表格渲染、颜色输出、格式化                  │
└─────────────────────────────────────────────────┘
```

### 3.2 数据流

```
用户输入命令
    │
    ▼
cli.rs: clap 解析参数 → Commands 枚举
    │
    ▼
main.rs: match Commands → 调用 service 对应方法
    │
    ▼
service.rs: 校验参数 → 调用 store → 返回结果
    │
    ▼
store.rs: 读写 JSON 文件 → 返回 Vec<Task> 或 Task
    │
    ▼
display.rs: 格式化输出到终端
```

---

## 4. 核心模块设计

### 4.1 数据模型 (`models/`)

```rust
// enums.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
}

// task.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**设计要点：**
- `id` 使用 `String` 存储 UUID，方便 JSON 序列化和用户输入
- `Status` 和 `Priority` 使用 `serde(rename_all = "snake_case")` 保证 JSON 可读性
- `DateTime<Utc>` 统一使用 UTC 时间，避免时区问题
- 所有字段 `pub`，因为这是数据载体，不需要封装

### 4.2 存储层 (`store.rs`)

```rust
// 定义 trait 接口，方便测试和未来扩展
pub trait Store {
    fn load(&self) -> Result<Vec<Task>>;
    fn save(&self, tasks: &[Task]) -> Result<()>;
}

pub struct JsonFileStore {
    file_path: PathBuf,
}

impl JsonFileStore {
    pub fn new() -> Result<Self> { ... }
    // 获取数据目录：~/.taskflow/
    fn data_dir() -> Result<PathBuf> { ... }
    // 写入前备份
    fn backup(&self) -> Result<()> { ... }
}

impl Store for JsonFileStore {
    fn load(&self) -> Result<Vec<Task>> {
        // 文件不存在 → 返回空 Vec
        // 文件存在 → 读取并反序列化
    }
    fn save(&self, tasks: &[Task]) -> Result<()> {
        // 先备份旧文件
        // 序列化并写入
    }
}
```

**设计要点：**
- 使用 `trait` 抽象存储接口，测试时可注入 `MemoryStore`
- 文件不存在时返回空列表，不报错
- 写入前自动备份，防止数据丢失
- 使用 `dirs::home_dir()` 获取跨平台 home 路径

### 4.3 业务逻辑层 (`service.rs`)

```rust
pub struct TaskService {
    store: JsonFileStore,
}

impl TaskService {
    pub fn new() -> Result<Self> { ... }

    // 创建任务
    pub fn add_task(&self, title: &str, desc: Option<&str>,
                    priority: Priority, tags: Vec<String>,
                    due: Option<NaiveDate>) -> Result<Task> {
        // 1. 校验标题长度
        // 2. 生成 UUID
        // 3. 设置 created_at, updated_at
        // 4. 默认 status = Todo
        // 5. 加载 → 追加 → 保存
        // 6. 返回新任务
    }

    // 列出任务（支持筛选）
    pub fn list_tasks(&self, status: Option<Status>,
                      priority: Option<Priority>,
                      tag: Option<&str>) -> Result<Vec<Task>> { ... }

    // 更新任务
    pub fn update_task(&self, id: &str, ...) -> Result<Task> { ... }

    // 删除任务
    pub fn delete_task(&self, id: &str) -> Result<Task> { ... }

    // 搜索任务
    pub fn search_tasks(&self, keyword: &str) -> Result<Vec<Task>> { ... }

    // 统计
    pub fn get_stats(&self) -> Result<TaskStats> { ... }
}
```

### 4.4 CLI 定义 (`cli.rs`)

```rust
#[derive(Parser)]
#[command(name = "taskflow", about = "命令行任务管理工具")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 创建新任务
    Add {
        /// 任务标题
        title: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long, default_value = "medium")]
        priority: Priority,
        #[arg(short, long, value_delimiter = ',')]
        tag: Vec<String>,
        #[arg(long)]
        due: Option<String>,
    },
    /// 列出任务
    List {
        #[arg(short, long)]
        status: Option<Status>,
        #[arg(short, long)]
        priority: Option<Priority>,
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// 更新任务
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        status: Option<Status>,
        // ... 其他可选字段
    },
    /// 删除任务
    Delete {
        id: String,
        #[arg(short, long)]
        force: bool,
    },
    /// 搜索任务
    Search {
        keyword: String,
    },
    /// 查看统计
    Stats,
    /// 导出数据
    Export {
        #[arg(long, default_value = "csv")]
        format: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}
```

### 4.5 展示层 (`display.rs`)

```rust
use colored::Colorize;
use comfy_table::Table;

pub fn print_task_table(tasks: &[Task]) {
    let mut table = Table::new();
    table.set_header(vec!["ID", "标题", "状态", "优先级", "标签", "截止日期"]);
    for task in tasks {
        table.add_row(vec![
            &task.id[..8],  // 只显示 UUID 前 8 位
            &task.title,
            format_status(&task.status),  // 带颜色
            format_priority(&task.priority),  // 带颜色
            &task.tags.join(", "),
            &task.due_date.map_or("-".into(), |d| d.to_string()),
        ]);
    }
    println!("{table}");
}

pub fn print_success(msg: &str) {
    println!("{}", msg.green());
}

pub fn print_error(msg: &str) {
    eprintln!("{}", msg.red());
}
```

### 4.6 错误处理 (`error.rs`)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("任务不存在: {0}")]
    NotFound(String),

    #[error("标题不能为空")]
    EmptyTitle,

    #[error("标题长度不能超过 100 个字符")]
    TitleTooLong,

    #[error("日期格式错误，请使用 YYYY-MM-DD 格式")]
    InvalidDate,

    #[error("数据文件读取失败: {0}")]
    StoreLoadError(#[from] std::io::Error),

    #[error("数据解析失败: {0}")]
    ParseError(#[from] serde_json::Error),
}
```

---

## 5. 关键技术点说明

### 5.1 为什么选 JSON 而不是 SQLite？
- 初学者无需学习 SQL
- `serde` 是 Rust 生态核心技能
- 文件可直接查看和手动编辑
- 1000 条任务性能完全足够

### 5.2 为什么用 UUID 而不是自增 ID？
- 无需维护计数器
- 删除后无 ID 冲突风险
- 用户只需输入前 8 位即可定位（显示截断）

### 5.3 为什么用 trait 抽象 Store？
- 测试时可注入内存实现，不依赖文件系统
- 未来可扩展其他存储后端（如 SQLite）
- 符合 Rust 的 trait 最佳实践

### 5.4 错误处理策略
- `error.rs` 用 `thiserror` 定义业务错误类型
- `main.rs` 用 `anyhow::Result` 统一捕获和输出
- 所有错误路径输出友好提示，不暴露内部细节

---

## 6. 测试策略

### 6.1 单元测试

| 模块 | 测试内容 |
|------|---------|
| `models` | 序列化/反序列化、Display 输出 |
| `store` | 文件读写、空文件处理、备份逻辑 |
| `service` | 标题校验、CRUD 逻辑、筛选逻辑 |

### 6.2 集成测试

```rust
// tests/cli_test.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_add_and_list() {
    let mut cmd = Command::cargo_bin("taskflow").unwrap();
    cmd.arg("add").arg("测试任务")
       .assert()
       .success()
       .stdout(predicate::str::contains("创建成功"));
}
```

### 6.3 测试数据隔离
- 使用 `tempfile` 创建临时目录
- 通过环境变量 `TASKFLOW_DATA_DIR` 覆盖默认路径
- 每个测试独立，互不影响

---

## 7. 实现建议

### 7.1 推荐实现顺序
1. 先跑通 `models` → 确保数据结构正确
2. 再实现 `store` → 确保数据能持久化
3. 然后 `cli` → 定义好接口
4. 接着 `service` → 串联逻辑
5. 最后 `display` → 美化输出

### 7.2 常见陷阱提醒
- `serde` 的 `rename_all` 要统一，否则 JSON 字段名不一致
- `chrono` 的 `NaiveDate` 解析需要用 `NaiveDate::parse_from_str`
- `uuid` 的 v4 feature 必须在 Cargo.toml 中显式开启
- Windows 路径分隔符用 `PathBuf` 处理，不要硬编码 `/`
- `colored` 在 Windows 终端可能需要启用 ANSI 支持

### 7.3 调试技巧
- 使用 `dbg!()` 宏快速调试值
- 使用 `serde_json::to_string_pretty()` 查看 JSON 数据
- 用 `cargo run -- add "test"` 快速测试
