# 第 2 章：数据模型定义

## 本章目标

- 用 `enum` 定义状态和优先级
- 用 `struct` 定义 Task 结构体
- 用 `serde` 实现 JSON 序列化/反序列化
- 实现 `Display` trait 进行友好展示
- 编写单元测试验证正确性

## 2.1 Rust 基础概念速览

在开始写代码之前，先了解本章涉及的 Rust 核心概念：

### struct（结构体）

```rust
struct Point {
    x: f64,
    y: f64,
}
```

类似其他语言的 class，但只有数据，没有方法。方法通过 `impl` 块单独定义。

### enum（枚举）

```rust
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
```

Rust 的 enum 比大多数语言的 enum 强大得多——每个变体可以携带数据（本章暂不涉及）。

### derive 宏

```rust
#[derive(Debug, Clone)]
struct Foo;
```

`#[derive(...)]` 让编译器自动为结构体/枚举实现指定的 trait。常见的有：
- `Debug`：支持 `{:?}` 格式化（调试打印）
- `Clone`：支持 `.clone()` 深拷贝
- `PartialEq`：支持 `==` 比较
- `Serialize`/`Deserialize`：serde 序列化/反序列化

### Option<T>

```rust
let maybe_name: Option<String> = Some("Alice".to_string());
let no_name: Option<String> = None;
```

Rust 没有 null！用 `Option<T>` 表示"可能有值，也可能没有"。`Some(v)` 表示有值，`None` 表示无值。

## 2.2 定义枚举：Status 和 Priority

创建 `src/models/enums.rs`：

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// 任务状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,        // 待办
    InProgress,  // 进行中
    Done,        // 已完成
}
```

### 逐行解析

| 代码 | 作用 |
|------|------|
| `#[derive(Debug)]` | 支持 `{:?}` 调试打印 |
| `#[derive(Clone)]` | 支持 `.clone()` 复制 |
| `#[derive(PartialEq)]` | 支持 `==` 比较（筛选时需要） |
| `#[derive(Serialize, Deserialize)]` | serde 自动序列化/反序列化 |
| `#[derive(clap::ValueEnum)]` | 让 clap 能直接从命令行字符串解析为枚举 |
| `#[serde(rename_all = "snake_case")]` | JSON 中使用 `snake_case`（如 `in_progress`），而非默认的 `InProgress` |

> **为什么需要 `clap::ValueEnum`？**
> 用户在命令行输入 `--status done`，clap 需要知道如何把字符串 `"done"` 转为 `Status::Done`。
> `ValueEnum` 自动实现了这个转换（要求变体名全小写匹配）。

### 实现 Display trait

```rust
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let d = match self {
            Status::Todo => "待办",
            Status::InProgress => "进行中",
            Status::Done => "已完成",
        };
        write!(f, "{}", d)
    }
}
```

**什么是 `Display` trait？**

`Display` 定义了用 `{}` 格式化时的输出内容。没有它，`println!("{}", status)` 会编译报错。

```rust
// 有了 Display 实现后：
let s = Status::Done;
println!("{}", s);  // 输出: 已完成
```

**`match` 是什么？**

`match` 是 Rust 的模式匹配，类似其他语言的 `switch`，但更强大。编译器会检查你是否覆盖了所有可能的情况（穷尽匹配），漏掉一个都编译不过。

### Priority 枚举

```rust
/// 任务优先级
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let d = match self {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        };
        write!(f, "{}", d)
    }
}
```

结构完全一样。`Display` 输出中文，方便终端展示。

## 2.3 定义 Task 结构体

创建 `src/models/task.rs`：

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use super::{Priority, Status};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,                    // UUID v4，自动生成
    pub title: String,                 // 任务标题，必填
    pub description: Option<String>,   // 任务描述，可选
    pub status: Status,                // 待办/进行中/已完成
    pub priority: Priority,            // 低/中/高
    pub tags: Vec<String>,             // 标签列表，可以为空
    pub due_date: Option<NaiveDate>,   // 截止日期，可选
    pub created_at: DateTime<Utc>,     // 创建时间
    pub updated_at: DateTime<Utc>,     // 更新时间
}
```

### 字段类型解读

| 字段 | 类型 | 为什么选这个类型 |
|------|------|-----------------|
| `id` | `String` | UUID 转字符串存储，方便 JSON 序列化和用户输入前缀匹配 |
| `title` | `String` | 必填文本 |
| `description` | `Option<String>` | 可选文本——`None` 表示没填，`Some("...")` 表示有内容 |
| `status` | `Status` | 自定义枚举，限定只能是三个值之一 |
| `priority` | `Priority` | 同上 |
| `tags` | `Vec<String>` | 可变长度的字符串列表，`vec![]` 表示空 |
| `due_date` | `Option<NaiveDate>` | 可选日期。`NaiveDate` 是"不含时区"的日期类型 |
| `created_at` | `DateTime<Utc>` | 带时区的日期时间，统一用 UTC 避免时区问题 |
| `updated_at` | `DateTime<Utc>` | 同上 |

> **为什么 `id` 用 `String` 而不是 `Uuid`？**
> 因为要存 JSON。虽然 `uuid` crate 也支持 serde，但存为 String 更直观，
> 且用户输入 ID 前缀时方便做 `starts_with` 匹配。

## 2.4 实现 Task 的 Display

```rust
impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.due_date {
            Some(d) => {
                write!(
                    f,
                    "({})[{}] {} ({}) - {}",
                    &self.id[..8],
                    self.priority,
                    self.title,
                    self.status,
                    d.format("%Y-%m-%d")
                )
            }
            _ => {
                write!(
                    f,
                    "({})[{}] {} ({})",
                    &self.id[..8],
                    self.priority,
                    self.title,
                    self.status
                )
            }
        }
    }
}
```

**关键语法点：**

- `&self.id[..8]`：取 ID 字符串的前 8 个字符（UUID 太长，只显示前 8 位）
- `self.priority`：因为 Priority 实现了 Display，所以 `{}` 会自动调用
- `d.format("%Y-%m-%d")`：chrono 的日期格式化
- `match self.due_date`：有截止日期就多显示 `- 2026-09-01`，没有就不显示

输出效果：

```
(a1b2c3d4)[高] 学习Rust所有权 (待办) - 2026-09-01
(a1b2c3d4)[高] 学习Rust所有权 (待办)
```

## 2.5 模块导出

创建 `src/models/mod.rs`：

```rust
mod enums;
mod task;

pub use enums::{Priority, Status};
pub use task::{Task, TaskCsvRow, TaskStats};
```

**模块系统解读：**

- `mod enums;` 告诉 Rust 去找 `enums.rs` 文件
- `pub use` 把内部模块的类型"提升"到 `models` 层级
- 这样其他文件就可以写 `use crate::models::{Status, Priority}` 而不是 `use crate::models::enums::Status`

## 2.6 编写单元测试

在 `enums.rs` 底部添加测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_serialize() {
        let a = Status::Todo;
        let res = serde_json::to_string(&a).unwrap();
        assert_eq!(res, "\"todo\"");

        let b = Status::InProgress;
        let res = serde_json::to_string(&b).unwrap();
        assert_eq!(res, "\"in_progress\"");
    }

    #[test]
    fn test_priority_serialize() {
        let a = Priority::High;
        let res = serde_json::to_string(&a).unwrap();
        assert_eq!(res, "\"high\"");
    }
}
```

在 `task.rs` 底部添加测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_task_serialize() {
        let task = Task {
            id: String::from("abc12345-1234-1234-1234-123456789abc"),
            title: "学习Rust".to_string(),
            description: None,
            status: Status::Todo,
            priority: Priority::High,
            tags: vec!["rust".to_string()],
            due_date: Some(NaiveDate::from_ymd_opt(2026, 9, 1).unwrap()),
            created_at: Utc.with_ymd_and_hms(2026, 8, 12, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 8, 12, 10, 0, 0).unwrap(),
        };

        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("\"title\":\"学习Rust\""));
        assert!(json.contains("\"status\":\"todo\""));
        assert!(json.contains("\"priority\":\"high\""));
    }

    #[test]
    fn test_task_deserialize() {
        let json = r#"{
            "id": "abc12345",
            "title": "测试任务",
            "description": null,
            "status": "in_progress",
            "priority": "medium",
            "tags": [],
            "due_date": "2026-09-01",
            "created_at": "2026-08-12T10:00:00Z",
            "updated_at": "2026-08-12T10:00:00Z"
        }"#;

        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.title, "测试任务");
        assert_eq!(task.status, Status::InProgress);
        assert_eq!(task.priority, Priority::Medium);
        assert_eq!(task.due_date, Some(NaiveDate::from_ymd_opt(2026, 9, 1).unwrap()));
    }
}
```

### 测试代码解读

| 语法 | 含义 |
|------|------|
| `#[cfg(test)]` | 只在 `cargo test` 时编译这个模块 |
| `#[test]` | 标记一个函数为测试函数 |
| `.unwrap()` | 测试中遇到 `Result` 直接解包——如果出错就 panic，测试失败 |
| `assert_eq!(a, b)` | 断言 a 等于 b，不等则 panic |
| `Utc.with_ymd_and_hms(...)` | chrono 构造 UTC 时间的方法，返回 `Result` 所以要 `unwrap` |
| `r#"{...}"#` | Rust 原始字符串字面量，不需要转义引号 |

## 2.7 验证

```bash
cargo test
```

你应该看到所有测试通过：

```
running 4 tests
test models::enums::tests::test_status_serialize ... ok
test models::enums::tests::test_priority_serialize ... ok
test models::task::tests::test_task_serialize ... ok
test models::task::tests::test_task_deserialize ... ok

test result: ok. 4 passed; 0 failed
```

## 本章小结

| 概念 | 你学到了 |
|------|---------|
| `struct` | 数据载体，字段用 `pub` 公开访问 |
| `enum` | 限定取值范围，配合 `match` 穷尽分析 |
| `derive` | 编译器自动实现 trait，减少样板代码 |
| `serde` | 一行 `#[derive(Serialize, Deserialize)]` 搞定 JSON |
| `Option<T>` | 替代 null，强制你处理"可能没有值"的情况 |
| `Display` | 自定义 `{}` 格式化输出 |
| `#[cfg(test)]` | 条件编译，测试代码不影响生产构建 |

---

[← 上一章](./01_project_setup.md) | [返回目录](./00_overview.md) | [下一章 →](./03_storage_layer.md)

---

📧 联系作者：pebblerwon@qq.com
