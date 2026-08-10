# TaskFlow 技术方案

## 1. 技术栈

### 1.1 核心依赖

| 类别       | 库            | 版本         | 用途                        |
| ---------- | ------------- | ------------ | --------------------------- |
| CLI 解析   | `clap`        | 4.x (derive) | 命令行参数解析、子命令定义  |
| 序列化     | `serde`       | 1.x          | 数据模型序列化框架          |
| JSON       | `serde_json`  | 1.x          | JSON 格式读写               |
| 时间       | `chrono`      | 0.4.x        | 日期时间处理，带 serde 支持 |
| UUID       | `uuid`        | 1.x (v4)     | 任务唯一 ID 生成            |
| 终端颜色   | `colored`     | 2.x          | 终端彩色文本输出            |
| 表格       | `comfy-table` | 7.x          | 终端表格渲染                |
| CSV        | `csv`         | 1.x          | CSV 文件导出                |
| 错误(应用) | `anyhow`      | 1.x          | 应用层错误处理              |
| 错误(库)   | `thiserror`   | 1.x          | 库层错误类型定义            |
| 目录       | `dirs`        | 5.x          | 获取跨平台 home 目录        |

### 1.2 开发依赖

| 类别     | 库           | 版本 | 用途             |
| -------- | ------------ | ---- | ---------------- |
| 集成测试 | `assert_cmd` | 2.x  | CLI 集成测试框架 |
| 断言     | `predicates` | 3.x  | 测试断言辅助     |
| 临时目录 | `tempfile`   | 3.x  | 测试用临时目录   |

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

| 文件              | 职责                                           | 关键类型                              |
| ----------------- | ---------------------------------------------- | ------------------------------------- |
| `main.rs`         | 入口函数，解析 CLI，调用 service，处理错误输出 | `fn main()`                           |
| `cli.rs`          | 定义所有子命令和参数结构                       | `Cli`, `Commands` enum                |
| `models/task.rs`  | Task 数据模型                                  | `Task` struct                         |
| `models/enums.rs` | 状态和优先级枚举                               | `Status`, `Priority`                  |
| `store.rs`        | 数据持久化，JSON 文件操作                      | `Store` trait, `JsonFileStore`        |
| `service.rs`      | 业务逻辑，数据校验，调用 store                 | `TaskService`                         |
| `display.rs`      | 终端输出格式化                                 | `print_task_table()`, `print_stats()` |
| `error.rs`        | 错误类型定义                                   | `TaskError` enum                      |

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

#### 文件路径规范

| 项目                     | 值                                  |
| ------------------------ | ----------------------------------- |
| 数据目录                 | `~/.taskflow/`                      |
| 主数据文件               | `data.json`                         |
| 备份文件                 | `data.json.bak`                     |
| 完整路径示例 (Windows)   | `C:\Users\xyzq\.taskflow\data.json` |
| 完整路径示例 (Linux/Mac) | `/home/xyzq/.taskflow/data.json`    |

#### Store Trait 接口

```rust
pub trait Store {
    /// 加载所有任务
    fn load(&self) -> Result<Vec<Task>>;

    /// 保存所有任务（覆盖写入）
    fn save(&self, tasks: &[Task]) -> Result<()>;
}
```

#### JsonFileStore 结构

```rust
pub struct JsonFileStore {
    file_path: PathBuf,  // 例如: ~/.taskflow/data.json
}
```

#### 需要用到的 API

**目录与路径操作：**

| API                         | 用途               | 示例                      |
| --------------------------- | ------------------ | ------------------------- |
| `dirs::home_dir()`          | 获取用户 home 目录 | `Some("/home/xyzq")`      |
| `PathBuf::push()`           | 拼接路径           | `path.push(".taskflow")`  |
| `std::fs::create_dir_all()` | 递归创建目录       | 创建 `~/.taskflow/`       |
| `Path::exists()`            | 检查文件是否存在   | 判断 `data.json` 是否存在 |

**文件读写：**

| API                         | 用途                 | 示例                          |
| --------------------------- | -------------------- | ----------------------------- |
| `std::fs::read_to_string()` | 读取文件内容为字符串 | 读取 JSON                     |
| `std::fs::write()`          | 写入字符串到文件     | 写入 JSON                     |
| `std::fs::copy()`           | 复制文件（用于备份） | `data.json` → `data.json.bak` |

**JSON 序列化：**

| API                                   | 用途                    | 示例                       |
| ------------------------------------- | ----------------------- | -------------------------- |
| `serde_json::from_str::<Vec<Task>>()` | JSON 字符串 → Vec<Task> | 反序列化                   |
| `serde_json::to_string_pretty()`      | Vec<Task> → 格式化 JSON | 序列化（带缩进，便于阅读） |

#### 实现流程

**`new()` 构造函数：**

```
1. 获取 home_dir
2. 拼接路径: home/.taskflow/data.json
3. 确保目录存在: create_dir_all(".taskflow")
4. 返回 JsonFileStore { file_path }
```

**`load()` 加载：**

```
1. 检查文件是否存在
   - 不存在 → 返回 Ok(vec![])
   - 存在 → 继续
2. 读取文件内容: read_to_string()
3. 反序列化: serde_json::from_str()
4. 返回 Ok(tasks)
```

**`save()` 保存：**

```
1. 备份旧文件（如果存在）
   - copy("data.json", "data.json.bak")
2. 序列化: serde_json::to_string_pretty()
3. 写入文件: write()
4. 返回 Ok(())
```

#### 错误处理

| 场景               | 错误信息                    |
| ------------------ | --------------------------- |
| 无法获取 home 目录 | `"无法获取用户主目录"`      |
| 创建目录失败       | `"创建数据目录失败: {err}"` |
| 读取文件失败       | `"读取数据文件失败: {err}"` |
| JSON 解析失败      | `"数据文件格式错误: {err}"` |
| 写入文件失败       | `"写入数据文件失败: {err}"` |

#### 测试建议

| 测试场景   | 方法                           |
| ---------- | ------------------------------ |
| 空文件加载 | 文件不存在时返回空 Vec         |
| 正常读写   | 写入后读取，验证数据一致       |
| 备份机制   | 写入两次，验证 `.bak` 文件存在 |

测试时使用 `tempfile::tempdir()` 创建临时目录，避免污染真实数据。

**设计要点：**

- 使用 `trait` 抽象存储接口，测试时可注入 `MemoryStore`
- 文件不存在时返回空列表，不报错
- 写入前自动备份，防止数据丢失
- 使用 `dirs::home_dir()` 获取跨平台 home 路径

### 4.3 业务逻辑层 (`service.rs`)

#### 结构体定义

```rust
pub struct TaskService {
    store: JsonFileStore,
}
```

#### TaskStats 统计结构体

```rust
pub struct TaskStats {
    pub total: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub done: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub overdue: usize,       // 已逾期（due_date < 今天 且 status != Done）
    pub completion_rate: f64,  // 完成率 = done / total
}
```

#### 方法清单

| 方法              | 签名                                                             | 职责                       |
| ----------------- | ---------------------------------------------------------------- | -------------------------- |
| `new`             | `() -> Result<Self>`                                             | 构造 service，初始化 store |
| `add_task`        | `(title, desc, priority, tags, due) -> Result<Task>`             | 创建任务                   |
| `list_tasks`      | `(status, priority, tag) -> Result<Vec<Task>>`                   | 列出/筛选任务              |
| `update_task`     | `(id, title, status, priority, desc, tags, due) -> Result<Task>` | 更新任务                   |
| `delete_task`     | `(id) -> Result<Task>`                                           | 删除任务                   |
| `search_tasks`    | `(keyword) -> Result<Vec<Task>>`                                 | 搜索任务                   |
| `get_stats`       | `() -> Result<TaskStats>`                                        | 统计信息                   |
| `export_tasks`    | `(format, output) -> Result<String>`                             | 导出数据                   |
| `find_task_by_id` | `(tasks, id) -> Result<usize>`                                   | 内部辅助：按 ID 查找索引   |
| `validate_title`  | `(title) -> Result<()>`                                          | 内部辅助：校验标题         |
| `parse_due_date`  | `(due) -> Result<Option<NaiveDate>>`                             | 内部辅助：解析日期字符串   |

#### 方法详细设计

##### `new()` — 构造函数

```
输入：无
输出：Result<TaskService>
流程：
  1. 调用 JsonFileStore::new() 创建存储实例
  2. 返回 TaskService { store }
```

##### `add_task()` — 创建任务（对应 PRD F1）

```
输入：
  - title: &str           必填，1~100 字符
  - desc: Option<&str>    可选描述
  - priority: Priority    默认 medium
  - tags: Vec<String>     可选标签列表
  - due: Option<&str>     可选截止日期字符串 "YYYY-MM-DD"
输出：Result<Task>        返回创建成功的完整 Task
流程：
  1. 调用 validate_title(title) 校验标题
  2. 调用 parse_due_date(due) 解析截止日期，格式错误返回 TaskError::InvalidDate
  3. 生成 UUID v4 作为 id
  4. 构造 Task：
     - status = Status::Todo
     - created_at = Utc::now()
     - updated_at = Utc::now()
  5. store.load() 加载现有任务
  6. 追加新任务到 Vec
  7. store.save() 保存
  8. 返回新 Task
错误：
  - TaskError::EmptyTitle        标题为空
  - TaskError::TitleTooLong      标题超 100 字符
  - TaskError::InvalidDate       日期格式错误
```

##### `list_tasks()` — 列出任务（对应 PRD F2）

```
输入：
  - status: Option<Status>      按状态筛选
  - priority: Option<Priority>  按优先级筛选
  - tag: Option<&str>           按标签筛选（包含匹配）
输出：Result<Vec<Task>>
流程：
  1. store.load() 加载所有任务
  2. 依次应用筛选条件（全部为 Option，None 表示不过滤）：
     - status 不为 None → 保留 status 相等的任务
     - priority 不为 None → 保留 priority 相等的任务
     - tag 不为 None → 保留 tags 中包含该标签的任务
  3. 返回筛选后的 Vec
```

##### `update_task()` — 更新任务（对应 PRD F3）

```
输入：
  - id: &str                    任务 ID（支持前缀匹配，至少 1 字符）
  - title: Option<&str>         新标题
  - status: Option<Status>      新状态
  - priority: Option<Priority>  新优先级
  - desc: Option<&str>          新描述
  - tags: Option<Vec<String>>   新标签（覆盖）
  - due: Option<&str>           新截止日期
输出：Result<Task>              返回更新后的 Task
流程：
  1. store.load() 加载所有任务
  2. 调用 find_task_by_id() 查找目标索引，未找到返回 TaskError::NotFound
  3. 若 title 不为 None，调用 validate_title() 校验
  4. 若 due 不为 None，调用 parse_due_date() 解析
  5. 逐字段更新（仅更新 Some 的字段）：
     - title → task.title
     - status → task.status
     - priority → task.priority
     - desc → task.description（Some 设置, None 不修改）
     - tags → task.tags
     - due → task.due_date
  6. 更新 task.updated_at = Utc::now()
  7. store.save() 保存
  8. 返回更新后的 Task（clone）
错误：
  - TaskError::NotFound(id)      任务不存在
  - TaskError::EmptyTitle        新标题为空
  - TaskError::TitleTooLong      新标题超长
  - TaskError::InvalidDate       日期格式错误
```

##### `delete_task()` — 删除任务（对应 PRD F4）

```
输入：
  - id: &str   任务 ID（支持前缀匹配）
输出：Result<Task>  返回被删除的 Task
流程：
  1. store.load() 加载所有任务
  2. 调用 find_task_by_id() 查找目标索引，未找到返回 TaskError::NotFound
  3. 从 Vec 中 remove 该任务
  4. store.save() 保存
  5. 返回被删除的 Task
错误：
  - TaskError::NotFound(id)
```

##### `search_tasks()` — 搜索任务（对应 PRD F5）

```
输入：
  - keyword: &str   搜索关键字
输出：Result<Vec<Task>>
流程：
  1. store.load() 加载所有任务
  2. 对 title 和 description 都执行大小写不敏感的 contains 匹配：
     - title.to_lowercase().contains(keyword.to_lowercase())
     - description.as_deref().map_or(false, |d| d.to_lowercase().contains(keyword.to_lowercase()))
  3. 任一字段命中即保留
  4. 返回匹配结果

说明：
  - 大小写不敏感通过双方 to_lowercase() 实现，非 locale 敏感（ASCII 场景足够）
  - keyword 为空字符串时，"".contains("") 为 true，会返回所有任务（视为「列出全部」）
```

##### `get_stats()` — 统计信息（对应 PRD F6）

```
输入：无
输出：Result<TaskStats>
流程：
  1. store.load() 加载所有任务
  2. 统计：
     - total = tasks.len()
     - todo = 计数 status == Todo
     - in_progress = 计数 status == InProgress
     - done = 计数 status == Done
     - high/medium/low = 按 priority 计数
     - overdue = 计数 due_date < Utc::now().date() 且 status != Done
     - completion_rate = if total == 0 { 0.0 } else { done as f64 / total as f64 }
  3. 返回 TaskStats
```

##### `export_tasks()` — 导出数据（对应 PRD F8）

```
输入：
  - format: &str             导出格式，当前仅支持 "csv"
  - output: Option<&str>     输出文件路径，None 则输出到 stdout
输出：Result<String>         返回导出文件路径或内容
流程：
  1. store.load() 加载所有任务
  2. 使用 csv crate 序列化 tasks 为 CSV 格式
  3. 若 output 不为 None → 写入文件，返回文件路径
  4. 若 output 为 None → 返回 CSV 字符串
```

#### 内部辅助方法

##### `validate_title()` — 标题校验

```
输入：title: &str
输出：Result<()>
规则：
  - 空字符串 → TaskError::EmptyTitle
  - 长度 > 100 → TaskError::TitleTooLong
  - 否则 → Ok(())
```

##### `parse_due_date()` — 日期解析

```
输入：due: Option<&str>
输出：Result<Option<NaiveDate>>
规则：
  - None → Ok(None)
  - Some(s) → NaiveDate::parse_from_str(s, "%Y-%m-%d")
    - 成功 → Ok(Some(date))
    - 失败 → Err(TaskError::InvalidDate)
```

##### `find_task_by_id()` — ID 查找

```
输入：tasks: &[Task], id: &str
输出：Result<usize>  返回索引
规则：
  - 支持前缀匹配：task.id.starts_with(id)
  - 匹配 0 个 → TaskError::NotFound(id)
  - 匹配多个 → TaskError::AmbiguousId(id)（可选，增强体验）
  - 匹配 1 个 → Ok(index)
```

#### 错误类型（`error.rs`）

```rust
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

    #[error("ID 匹配到多个任务: {0}")]
    AmbiguousId(String),

    #[error("数据文件读取失败: {0}")]
    StoreLoadError(#[from] std::io::Error),

    #[error("数据解析失败: {0}")]
    ParseError(#[from] serde_json::Error),
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
use comfy_table::{
    presets::UTF8_FULL, Attribute, Cell, Color, ContentArrangement, Table,
};

pub fn print_task_table(tasks: &[Task]) {
    let mut table = Table::new();

    // 1. 加载 Unicode 全边框 preset（横线/竖线/交叉点）
    // 2. ContentArrangement::Dynamic 让表格按内容 + 终端宽度自动调整列宽
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["ID", "标题", "状态", "优先级", "标签", "截止日期"]);

    for task in tasks {
        let due_str = task.due_date.map_or("--".to_string(), |d| d.to_string());
        let tag_str = if task.tags.is_empty() {
            "-".to_string()
        } else {
            task.tags.join(",")
        };

        // 表格内颜色必须用 comfy_table 原生 Cell 样式 API，不能用 colored
        // 原因见下方"对齐约束"
        table.add_row(vec![
            Cell::new(&task.id[..task.id.len().min(8)]),    // UUID 安全截断：min(8, len)
            Cell::new(&task.title),
            status_cell(&task.status),                        // 返回带样式的 Cell
            priority_cell(&task.priority),
            Cell::new(&tag_str),
            Cell::new(&due_str),
        ]);
    }

    println!("{table}");
}

/// 状态单元格：用 comfy_table 原生 Color/Attribute 样式
fn status_cell(status: &Status) -> Cell {
    match status {
        Status::Done => {
            Cell::new("已完成").fg(Color::Green).add_attribute(Attribute::CrossedOut) // PRD F7：Done=绿色删除线
        }
        Status::InProgress => Cell::new("进行中").fg(Color::Blue),
        Status::Todo => Cell::new("未完成").fg(Color::DarkGrey),
    }
}

/// 优先级单元格：用 comfy_table 原生 Color 样式
fn priority_cell(priority: &Priority) -> Cell {
    match priority {
        Priority::High => Cell::new("高").fg(Color::Red),
        Priority::Medium => Cell::new("中").fg(Color::Yellow),
        Priority::Low => Cell::new("低").fg(Color::Green),
    }
}

/// 成功消息：自动加 ✓ 前缀 + 绿色（此处仍用 colored，非表格场景无对齐问题）
pub fn print_success(msg: &str) {
    println!("✓ {}", msg.green());
}

/// 错误消息：自动加 ✗ 前缀 + 红色 + 走 stderr
pub fn print_error(msg: &str) {
    eprintln!("✗ 错误：{}", msg.red());
}

/// 普通信息（无前缀无颜色）：用于「暂无任务」「未找到匹配任务」等中性状态
pub fn print_info(msg: &str) {
    println!("{}", msg);
}

/// 警告信息：仅黄色着色，无前缀（T3.4 输入校验时启用）
/// 注意：与 print_success/print_error 不同，警告前缀 ⚠ 由调用方自行拼接，
/// 因为不同场景的警告前缀可能不同（⚠ / ! / [WARN] 等）。
pub fn print_warning(msg: &str) {
    println!("{}", msg.yellow());
}
```

#### 设计与契约

| 函数               | 通道   | 前缀       | 颜色       | 调用方职责                               |
| ------------------ | ------ | ---------- | ---------- | ---------------------------------------- |
| `print_success`    | stdout | `✓ `       | 绿         | 传"主体内容"，前缀/颜色由 display 自动加 |
| `print_error`      | stderr | `✗ 错误：` | 红         | 同上                                     |
| `print_info`       | stdout | 无         | 无         | 同上                                     |
| `print_warning`    | stdout | 无         | 黄         | 调用方控制前缀（仅着色）                 |
| `print_task_table` | stdout | 无         | 表格列自带 | 传 `&[Task]`                             |

#### 对齐约束

##### 颜色规则（严格遵循 [PRD F7](docs/PRD.md)）

- 状态 Done = **绿色删除线**（`Color::Green` + `Attribute::CrossedOut`）
- 状态 InProgress = 蓝色（`Color::Blue`）
- 状态 Todo = 灰色（`Color::DarkGrey`）
- 优先级 High = 红、Medium = 黄、Low = 绿

##### 表格渲染约束

- **表格内颜色必须用 `comfy_table` 原生 `Cell` 样式 API**（`Cell::new(text).fg(Color::...)` / `.add_attribute(Attribute::...)`），**不能用 `colored` 库**
  - 原因：`colored` 库通过在字符串中嵌入 ANSI 转义码（`\x1b[...m`）实现着色，`comfy_table` 默认把转义码也算成字符宽度，导致彩色列被算得超宽、表格错位
  - 改用 `comfy_table` 原生 `Color` / `Attribute` 后，样式信息与文本内容分离，`comfy_table` 按可见字符算宽，表格正确对齐
  - `colored` 库仍保留给 `print_success` / `print_error` / `print_warning` 等非表格场景使用（单行输出无对齐问题）
- **必须用 `load_preset(UTF8_FULL)`** 加载 Unicode 全边框，否则表格无横线/竖线
- **必须用 `set_content_arrangement(ContentArrangement::Dynamic)`** 让表格按内容自适应列宽
- **ID 必须用 `min(8)` 防 panic**：mock 测试数据 id 可能很短（`"1"`、`"2"`），裸切片 `&id[..8]` 会 panic
- **CJK 字符宽度**：当前默认靠 `Dynamic` 自适应；若发现中文列依然错位（不同终端字体宽度不同），可加 `unicode-width` crate 用 `UnicodeWidthStr` 精确计算

##### `print_warning` 当前无 caller

- **T3.4 输入校验完善时启用**（如"标签超过 10 个"等警告）
- 注意：与 `print_success`/`print_error` 不同，警告前缀由调用方按需拼接

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

### 4.7 入口层与端到端串联（`main.rs` —— 对应 DEV_PLAN T1.6）

#### 入口职责

`main.rs` 是唯一串联所有模块的入口，但职责极薄，只做三件事：

1. 调用 `Cli::parse()` 解析命令行参数
2. 构造 `TaskService::new()?` 拿到业务层实例
3. 将 `Commands` 子命令 dispatch 到 service 对应方法并打印结果

不做业务校验、不做 IO、不做格式化（格式化下沉到后续 `display.rs`）。

#### 推荐结构

```rust
use anyhow::{Context, Result};
use clap::Parser;
use crate::{
    cli::{Cli, Commands},
    service::TaskService,
};

fn main() {
    if let Err(e) = run() {
        eprintln!("✗ 错误：{e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let service = TaskService::new().context("初始化任务服务失败")?;

    match cli.command {
        Commands::Add { title, description, priority, tag, due } => {
            let tags: Vec<&str> = tag.iter().map(String::as_str).collect();
            let task = service.add_task(&title, description.as_deref(), Some(priority), tags, due.as_deref())?;
            println!("✓ 任务创建成功：{}", task);
        }
        Commands::List { status, priority, tag } => {
            let tasks = service.list_tasks(status, priority, tag.as_deref())?;
            if tasks.is_empty() {
                println!("暂无任务");
            } else {
                for t in &tasks {
                    println!("{}", t);
                }
            }
        }
        Commands::Update { id, title, status, priority } => {
            let task = service.update_task(
                &id,
                title.as_deref(),
                status,
                priority,
                None,  // desc
                None,  // tags
                None,  // due
            )?;
            println!("✓ 任务已更新：{}", task);
        }
        Commands::Delete { id, force: _ } => {
            let deleted = service.delete_task(&id)?;
            println!("✓ 已删除任务：{} ({})", deleted.title, deleted.id);
        }
        Commands::Search { keyword } => {
            let res = service.search_task(&keyword)?;
            if res.is_empty() {
                println!("未找到匹配任务");
            } else {
                println!("✓ 搜索到 {} 条结果：", res.len());
                for t in &res {
                    println!("{t}");
                }
            }
        }
        // 阶段二/三实现
        Commands::Stats | Commands::Export { .. } => {
            anyhow::bail!("该命令将在后续阶段实现");
        }
    }
    Ok(())
}
```

#### Dispatch 映射表

| 子命令   | service 方法                                                | 备注                                                 |
| -------- | ----------------------------------------------------------- | ---------------------------------------------------- |
| `Add`    | `add_task(title, desc, priority, tags, due)`                | T1.6 已就绪                                          |
| `List`   | `list_tasks(status, priority, tag)`                         | T1.6 已就绪                                          |
| `Update` | `update_task(id, title, status, priority, desc, tags, due)` | T1.6 仅传入 cli 提供的字段，未提供的传 `None`        |
| `Delete` | `delete_task(id)`                                           | T1.6 不实现交互确认（见 T3.1），`--force` 仅作为占位 |
| `Search` | `search_task(keyword)`                                      | T2.2 已实现：大小写不敏感，命中 title 或 description |
| `Stats`  | （stub）`anyhow::bail!("未实现")`                           | T2.4 实现                                            |
| `Export` | （stub）`anyhow::bail!("未实现")`                           | T3.2 实现                                            |

#### 输出规范

`main.rs` 通过 `display::*` 函数统一所有输出，前缀/颜色/通道由 display 内部负责，main 只传"主体内容"。

| 场景                                  | 调用函数                                                   | 实际输出                   | 通道   |
| ------------------------------------- | ---------------------------------------------------------- | -------------------------- | ------ |
| 单条任务操作成功（Add/Update/Delete） | `print_success(&format!(...))`                             | `✓ <消息>：<Task Display>` | stdout |
| `list` 有结果                         | `print_task_table(&tasks)`                                 | 表格                       | stdout |
| `list` 无结果                         | `print_info("暂无任务")`                                   | `暂无任务`                 | stdout |
| `search` 命中                         | `println!("搜索到 N 条结果：")` + `print_task_table(&res)` | 计数 + 表格                | stdout |
| `search` 无结果                       | `print_info("未找到匹配任务")`                             | `未找到匹配任务`           | stdout |
| 业务错误（TaskError）                 | 经 `?` 上抛到 main → `print_error(&format!("{e:#}"))`      | `✗ 错误：<anyhow chain>`   | stderr |
| 系统错误（io/json）                   | 同上                                                       | 同上                       | stderr |
| 输入校验警告（T3.4 启用）             | `print_warning(&format!("⚠ {msg}"))`                       | `⚠ <消息>`                 | stdout |

前缀/颜色约定（display 内部硬编码）：

| 类型 | 前缀                            | 颜色 |
| ---- | ------------------------------- | ---- |
| 成功 | `✓ `（U+2713 + 半角空格）       | 绿   |
| 错误 | `✗ 错误：`（U+2717 + 半角空格） | 红   |
| 信息 | 无                              | 无   |
| 警告 | `⚠ `（U+26A0 + 半角空格）       | 黄   |

> **历史推迟项（已落地）**
>
> - ~~不引入 `comfy-table` 渲染~~ → T2.3 已接入
> - ~~不引入 `colored` 上色~~ → T2.3 已接入
> - ~~不使用 `display.rs` 模块~~ → 已接入 main.rs
> - 不实现 `Delete` 的交互确认 → T3.1
> - `print_warning` 无 caller → T3.4 输入校验时启用

#### 错误处理收敛

- 顶层统一格式：`fn run() -> anyhow::Result<()>`，`main` 只负责 `if let Err` 打印 + `exit(1)`
- 业务错误经由 `TaskError`（`thiserror`）产生，自动通过 `#[from]` 转为 `anyhow::Error`
- 退出码：成功 `0`；任何错误路径 `1`（后续可细分，但不强制）
- `TaskError` 当前已覆盖：
  - `NotFound(id)` — `update`/`delete` 找不到
  - `AmbiguousId(id)` — 前缀匹配命中多条
  - `EmptyTitle` / `TitleTooLong` — `add`/`update` 标题校验
  - `InvalidDate` — `due` 解析失败
  - `StoreLoadError(io)` / `ParseError(json)` — 存储层兜底
- 现阶段允许的 `.unwrap()`：
  - `Uuid::new_v4()`（系统调用，不可能失败）
  - `Cli::parse()` 之外的 `unwrap` 一律禁止；如有，标记 TODO 等 T3.3 清理

#### 手动验收脚本

与 DEV_PLAN § 阶段一验收 对齐，并补充异常路径：

```bash
# 正常路径
cargo run -- add "学习Rust" -p high
cargo run -- list
cargo run -- update <id前缀> --status done
cargo run -- delete <id前缀>
cargo run -- list                 # 确认删除生效

# 异常路径（期待非 0 退出 + 中文提示）
cargo run -- add ""               # ✗ 错误：标题不能为空
cargo run -- update xx --status done   # ✗ 错误：任务不存在：xx
# 日期格式错误：expect ✗ 错误：日期格式错误，请使用 YYYY-MM-DD 格式
cargo run -- add "测试日期" --due 2099/01/01
# 日期合法（对照）：expect ✓ 任务创建成功
cargo run -- add "测试日期" --due 2099-01-01
```

> `<id前缀>` 取自 `list` 输出中 UUID 前 8 位。

#### 完成判定（DoD）

- 上述 8 条命令全部按预期输出，**无 panic**
- `~/.taskflow/data.json` 内容随命令正确变化（`cat` 可验证）
- 错误路径输出中文友好提示且退出码非 0
- `cargo build` 无 warning（建议项）
- 单元测试与既有 service/store/models 测试不回归：`cargo test` 全绿

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

| 模块      | 测试内容                       |
| --------- | ------------------------------ |
| `models`  | 序列化/反序列化、Display 输出  |
| `store`   | 文件读写、空文件处理、备份逻辑 |
| `service` | 标题校验、CRUD 逻辑、筛选逻辑  |

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
- **表格内不能用 `colored` 着色**：`colored` 嵌入的 ANSI 转义码会被 `comfy_table` 计入字符宽度导致列错位，表格内颜色须用 `comfy_table` 原生 `Cell` 样式 API（`Color` / `Attribute`）

### 7.3 调试技巧

- 使用 `dbg!()` 宏快速调试值
- 使用 `serde_json::to_string_pretty()` 查看 JSON 数据
- 用 `cargo run -- add "test"` 快速测试
