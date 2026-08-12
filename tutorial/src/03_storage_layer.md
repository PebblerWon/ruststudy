# 第 3 章：存储层与错误处理

## 本章目标

- 用 `thiserror` 定义业务错误类型
- 用 `trait` 抽象存储接口
- 实现 `JsonFileStore`：JSON 文件读写
- 实现自动备份机制
- 理解 Rust 错误处理的哲学

## 3.1 Rust 错误处理哲学

Rust 没有 try/catch。它用两种 `Result` 类型处理错误：

```rust
// 标准库的 Result
enum Result<T, E> {
    Ok(T),    // 成功，携带返回值
    Err(E),   // 失败，携带错误
}
```

### 本项目采用双层错误方案

| 层级 | 库 | 用途 |
|------|-----|------|
| 库层（`error.rs`） | `thiserror` | 定义精确的业务错误类型 |
| 应用层（`main.rs`） | `anyhow` | 统一捕获、传播、输出 |

```
service.rs 抛出 TaskError::NotFound("abc")
    ↓ 通过 #[from] 自动转为 anyhow::Error
main.rs 用 {e:#} 打印完整错误链
```

## 3.2 定义错误类型（error.rs）

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("任务不存在：{0}")]
    NotFound(String),

    #[error("标题不能为空")]
    EmptyTitle,

    #[error("标题长度不能超过 100 个字符")]
    TitleTooLong,

    #[error("日期格式错误，请使用 YYYY-MM-DD 格式")]
    InvalidDate,

    #[error("ID 匹配到多个任务：{0}")]
    AmbiguousId(String),

    #[error("数据文件读取失败：{0}")]
    StoreLoadError(#[from] std::io::Error),

    #[error("数据解析失败：{0}")]
    ParseError(#[from] serde_json::Error),

    #[error("不支持的导出格式：{0}")]
    UnsupportedFormat(String),

    #[error("无法获取父目录")]
    HomeDirNotFound,

    #[error("最多支持10条标签")]
    TooManyTags,
}
```

### 逐行解析

| 语法 | 含义 |
|------|------|
| `#[derive(Error)]` | thiserror 自动实现 `std::error::Error` trait |
| `#[error("...")]` | 定义该错误的 `Display` 输出（用户看到的消息） |
| `{0}` | 引用枚举变体中第一个字段的值 |
| `#[from]` | 自动生成 `From<io::Error> for TaskError`，让 `?` 运算符自动转换 |

> **`#[from]` 的魔力**：当函数返回 `Result<T, TaskError>` 时，遇到 `io::Error` 可以用 `?` 直接传播，
> 编译器会自动调用 `TaskError::from(io_error)` 转换类型。

## 3.3 定义 Store trait

在 `src/store.rs` 中：

```rust
use crate::error::TaskError;
use crate::models::Task;
use anyhow::Result;
use serde_json::{from_str, to_string_pretty};
use std::fs::{create_dir_all, read_to_string};
use std::path::PathBuf;

/// 存储层接口——定义"存储"应该做什么，不关心怎么做
pub trait Store {
    /// 加载所有任务
    fn load(&self) -> Result<Vec<Task>>;

    /// 保存所有任务（覆盖写入）
    fn save(&self, tasks: &[Task]) -> Result<()>;
}
```

### 为什么要用 trait？

```rust
// 生产环境：用 JsonFileStore
let store = JsonFileStore::new()?;

// 测试环境：可以注入 MemoryStore（不碰文件系统）
struct MemoryStore { data: Vec<Task> }
impl Store for MemoryStore { ... }
```

trait 让代码面向接口编程，测试时可以替换实现。

## 3.4 实现 JsonFileStore

### 3.4.1 构造函数

```rust
pub struct JsonFileStore {
    pub(crate) file_path: PathBuf,  // 例如: ~/.taskflow/data.json
}

impl JsonFileStore {
    pub fn new() -> Result<JsonFileStore> {
        // 支持通过环境变量覆盖数据目录（测试时需要）
        let data_dir = std::env::var("TASKFLOW_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|_| dirs::home_dir().ok_or(TaskError::HomeDirNotFound))
            .map(|d| d.join(".taskflow"))?;

        // 确保目录存在
        create_dir_all(&data_dir)?;

        let data_path = data_dir.join("data.json");
        Ok(JsonFileStore {
            file_path: data_path,
        })
    }
}
```

### 逐行解析

```rust
std::env::var("TASKFLOW_DATA_DIR")   // 读环境变量，返回 Result<String>
    .map(PathBuf::from)               // Ok(path) → Ok(PathBuf)
    .or_else(|_|                       // 如果环境变量不存在
        dirs::home_dir()              // 获取 home 目录
            .ok_or(TaskError::HomeDirNotFound)  // Option → Result
    )
    .map(|d| d.join(".taskflow"))?    // 拼接 .taskflow 子目录
```

> **`?` 运算符**：如果结果是 `Err`，立即返回错误；如果是 `Ok`，取出值继续。
> 这是 Rust 错误传播的核心语法。

### 3.4.2 load() 方法

```rust
impl Store for JsonFileStore {
    fn load(&self) -> Result<Vec<Task>> {
        let path = &self.file_path;

        if path.exists() {
            let content = read_to_string(&path)?;  // 读文件 → String
            let tasks: Vec<Task> = from_str(&content)?;  // JSON → Vec<Task>
            Ok(tasks)
        } else {
            Ok(vec![])  // 文件不存在 → 返回空列表，不报错
        }
    }
}
```

**数据流：**

```
文件存在？
  ├─ 是 → 读取文本 → serde_json 解析 → Vec<Task>
  └─ 否 → 返回空 Vec（首次使用时文件还不存在）
```

> **为什么文件不存在不报错？**
> 第一次运行 `taskflow add` 时，`~/.taskflow/data.json` 还不存在。
> 返回空列表是合理的——"没有任务"不是错误。

### 3.4.3 save() 方法

```rust
    fn save(&self, tasks: &[Task]) -> Result<()> {
        // 1. 序列化为格式化的 JSON
        let json = to_string_pretty(tasks)?;

        // 2. 计算备份路径
        let back_path = self
            .file_path
            .parent()
            .ok_or(TaskError::HomeDirNotFound)?
            .join("data.json.bak");

        // 3. 如果旧文件存在，先备份
        if self.file_path.exists() {
            std::fs::copy(&self.file_path, &back_path)?;
        }

        // 4. 写入新文件
        std::fs::write(&self.file_path, &json).inspect_err(|_| {
            // 写入失败时尝试恢复备份
            let _ = std::fs::copy(&back_path, &self.file_path);
        })?;

        Ok(())
    }
```

**安全机制：**

```
写入前：data.json → data.json.bak（备份旧数据）
写入失败：自动从 .bak 恢复
```

> **`inspect_err()`**：Rust 1.76+ 的方法，在 `Err` 时执行副作用（如日志、恢复），但不改变 `Result` 本身。

## 3.5 完整文件一览

```
~/.taskflow/
├── data.json        ← 主数据文件
└── data.json.bak    ← 上次写入前的备份
```

`data.json` 内容示例：

```json
[
  {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "title": "学习Rust",
    "description": null,
    "status": "todo",
    "priority": "high",
    "tags": ["rust", "study"],
    "due_date": "2026-09-01",
    "created_at": "2026-08-12T10:00:00Z",
    "updated_at": "2026-08-12T10:00:00Z"
  }
]
```

## 3.6 编写单元测试

在 `store.rs` 底部添加：

```rust
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::models::{Priority, Status, Task};
    use chrono::{TimeZone, Utc};
    use std::fs;

    /// 创建一个用于测试的 mock Task
    pub fn mock_task(id: &str) -> Task {
        Task {
            id: id.to_string(),
            title: format!("任务{}", id),
            description: None,
            status: Status::Todo,
            priority: Priority::High,
            due_date: None,
            tags: vec![],
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 1, 1, 1).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 1, 1, 1).unwrap(),
        }
    }

    /// 创建使用临时目录的 store（不污染真实数据）
    pub fn temp_store(test_name: &str) -> JsonFileStore {
        let path = std::env::temp_dir();
        let data_dir = path.join("taskflow_test").join(test_name);
        let _ = fs::remove_dir_all(&data_dir);  // 清理上次残留
        fs::create_dir_all(&data_dir).unwrap();
        let file_path = data_dir.join("test_data.json");
        JsonFileStore { file_path }
    }

    pub fn cleanup(store: &JsonFileStore) {
        if let Some(dir) = store.file_path.parent() {
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn test_load_file_not_exists() {
        let store = temp_store("load_not_exists");
        assert!(!store.file_path.exists());
        assert!(store.load().unwrap().is_empty());
        cleanup(&store);
    }

    #[test]
    fn test_save_and_load() {
        let store = temp_store("save_and_load");

        let tasks = vec![mock_task("1"), mock_task("2")];
        store.save(&tasks).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "1");
        assert_eq!(loaded[1].id, "2");

        cleanup(&store);
    }

    #[test]
    fn test_backup_created() {
        let store = temp_store("backup_test");

        // 第一次写入
        store.save(&[mock_task("a")]).unwrap();
        // 第二次写入（应该触发备份）
        store.save(&[mock_task("b")]).unwrap();

        let bak_path = store.file_path.parent().unwrap().join("data.json.bak");
        assert!(bak_path.exists());

        cleanup(&store);
    }

    #[test]
    fn test_load_invalid_json() {
        let store = temp_store("invalid_json");
        fs::write(&store.file_path, "这不是json").unwrap();
        assert!(store.load().is_err());
        cleanup(&store);
    }
}
```

### 测试要点

| 测试 | 验证什么 |
|------|---------|
| `test_load_file_not_exists` | 文件不存在时返回空列表 |
| `test_save_and_load` | 写入后能正确读回 |
| `test_backup_created` | 第二次写入时自动创建 `.bak` |
| `test_load_invalid_json` | 损坏的 JSON 返回错误而非 panic |

## 3.7 验证

```bash
cargo test
```

确保所有测试通过。

## 本章小结

| 概念 | 你学到了 |
|------|---------|
| `thiserror` | 用 derive 宏自动生成错误类型的 `Error` + `Display` 实现 |
| `#[from]` | 自动类型转换，让 `?` 运算符无缝传播错误 |
| `trait` | 定义接口，实现多态——测试时可替换 |
| `PathBuf` | 跨平台路径操作，不要硬编码 `/` 或 `\` |
| `serde_json` | `from_str` 反序列化、`to_string_pretty` 序列化 |
| 文件操作 | `read_to_string`、`write`、`copy`、`create_dir_all` |
| `inspect_err` | 在错误传播前执行副作用（如恢复备份） |

---

[← 上一章](./02_data_model.md) | [返回目录](./00_overview.md) | [下一章 →](./04_cli_parsing.md)

---

📧 联系作者：pebblerwon@qq.com
