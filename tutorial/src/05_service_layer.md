# 第 5 章：业务逻辑层（Service CRUD）

## 本章目标

- 实现 `TaskService` 结构体
- 完成增删改查（CRUD）核心逻辑
- 实现数据校验（标题、日期、标签）
- 理解 UUID 生成和时间戳处理

## 5.1 Service 层的职责

Service 层是"业务规则"的守护者：

```
CLI 层：只管解析参数
Service 层：校验数据 → 组装 Task → 调用 Store → 返回结果
Store 层：只管读写文件
```

## 5.2 定义 TaskService

创建 `src/service.rs`：

```rust
use crate::{
    error::TaskError,
    models::{Priority, Status, Task},
    store::{JsonFileStore, Store},
};
use anyhow::Result;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

pub struct TaskService {
    pub store: JsonFileStore,
}

impl TaskService {
    /// 构造函数：创建 store 实例
    pub fn new() -> Result<Self> {
        let store = JsonFileStore::new()?;
        Ok(TaskService { store })
    }

    /// 测试用：注入自定义 store
    #[cfg(test)]
    pub fn with_store(store: JsonFileStore) -> Result<Self> {
        Ok(TaskService { store })
    }
}
```

> **`#[cfg(test)]`**：这个函数只在 `cargo test` 时编译。生产代码看不到它，
> 但测试可以用它注入临时目录的 store，不污染真实数据。

## 5.3 数据校验

### 标题校验

```rust
impl TaskService {
    pub fn validate_title(title: &str) -> Result<()> {
        if title.is_empty() {
            return Err(TaskError::EmptyTitle.into());
        }
        if title.chars().count() > 100 {
            return Err(TaskError::TitleTooLong.into());
        }
        Ok(())
    }
}
```

> **为什么用 `chars().count()` 而不是 `.len()`？**
> `.len()` 返回的是**字节数**，不是字符数。一个中文字符占 3 字节。
> `"中文".len()` = 6，但 `"中文".chars().count()` = 2。
> 用户期望的是"100 个字符"，不是"100 个字节"。

### 日期校验

```rust
    pub fn validate_due_date(due: Option<&str>) -> Result<Option<NaiveDate>> {
        match due {
            None => Ok(None),
            Some(s) => {
                let date = NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|_| TaskError::InvalidDate)?;
                Ok(Some(date))
            }
        }
    }
```

`NaiveDate::parse_from_str` 是严格解析：
- `"2026-09-01"` → Ok
- `"2026/09/01"` → Err（格式不对）
- `"2026-02-30"` → Err（日期不存在）

### 标签校验

```rust
    pub fn validate_tags(tags: &[&str]) -> Result<()> {
        if tags.len() > 10 {
            return Err(TaskError::TooManyTags.into());
        }
        Ok(())
    }
```

## 5.4 添加任务（Create）

```rust
    pub fn add_task(
        &self,
        title: &str,
        desc: Option<&str>,
        priority: Option<Priority>,
        tags: Vec<&str>,
        due: Option<&str>,
    ) -> Result<Task> {
        // 1. 校验
        Self::validate_title(title)?;
        Self::validate_tags(&tags)?;
        let due_date = Self::validate_due_date(due)?;

        // 2. 构造 Task
        let id = Uuid::new_v4();  // 生成随机 UUID
        let description = desc.map(|d| d.to_string());

        let task = Task {
            id: id.to_string(),
            title: title.to_string(),
            description,
            status: Status::Todo,          // 新任务默认"待办"
            priority: priority.unwrap_or(Priority::Medium),  // 默认中优先级
            tags: tags.iter().map(|s| s.to_string()).collect(),
            due_date,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // 3. 加载现有任务 → 追加 → 保存
        let mut tasks = self.store.load()?;
        tasks.push(task.clone());
        self.store.save(&tasks)?;

        Ok(task)
    }
```

### 关键语法点

| 语法 | 含义 |
|------|------|
| `Uuid::new_v4()` | 生成随机 UUID（如 `a1b2c3d4-e5f6-...`），本身返回 `Uuid` 不是 `Result` |
| `desc.map(\|d\| d.to_string())` | `Option<&str>` → `Option<String>` |
| `priority.unwrap_or(...)` | `Option<Priority>` → `Priority`，None 时用默认值 |
| `tags.iter().map(\|s\| s.to_string()).collect()` | `Vec<&str>` → `Vec<String>` |
| `task.clone()` | 因为 push 会转移所有权，先 clone 一份用于返回 |

## 5.5 列出任务（Read）

```rust
    pub fn list_tasks(
        &self,
        status: Option<Status>,
        priority: Option<Priority>,
        tag: Option<&str>,
    ) -> Result<Vec<Task>> {
        let mut tasks = self.store.load()?;

        // 链式筛选：每个条件独立过滤
        if let Some(s) = status {
            tasks = tasks.into_iter().filter(|t| t.status == s).collect();
        }
        if let Some(p) = priority {
            tasks = tasks.into_iter().filter(|t| t.priority == p).collect();
        }
        if let Some(arg_tag) = tag {
            tasks = tasks
                .into_iter()
                .filter(|t| t.tags.iter().any(|cur_tag| cur_tag.contains(arg_tag)))
                .collect();
        }

        Ok(tasks)
    }
```

> **`if let Some(x) = ...`**：只在值存在时执行代码块，等价于 `match` 但更简洁。
>
> **`.filter(|t| ...)`**：迭代器适配器，保留满足条件的元素。
> `.any(|tag| ...)` 检查标签列表中是否有任何一个包含关键字。

## 5.6 更新任务（Update）

```rust
    pub fn update_task(
        &self,
        id: &str,
        title: Option<&str>,
        status: Option<Status>,
        priority: Option<Priority>,
        desc: Option<&str>,
        tags: Vec<&str>,
        due: Option<&str>,
    ) -> Result<Task> {
        let mut tasks = self.store.load()?;
        let update_index = Self::find_task_by_id(&tasks, id)?;

        // get_mut 获取可变引用
        let update_item = tasks
            .get_mut(update_index)
            .ok_or_else(|| TaskError::NotFound(id.to_string()))?;

        // 只更新 Some 的字段
        if let Some(t) = title {
            Self::validate_title(t)?;
            update_item.title = t.to_string();
        }
        if let Some(s) = status {
            update_item.status = s;
        }
        if let Some(p) = priority {
            update_item.priority = p;
        }
        if let Some(d) = desc {
            update_item.description = Some(d.to_string());
        }
        if !tags.is_empty() {
            Self::validate_tags(&tags)?;
            update_item.tags = tags.iter().map(|s| s.to_string()).collect();
        }
        if due.is_some() {
            let date = Self::validate_due_date(due)?;
            update_item.due_date = date;
        }

        update_item.updated_at = Utc::now();
        let updated_task = update_item.clone();
        self.store.save(&tasks)?;
        Ok(updated_task)
    }
```

> **`get_mut()` + `ok_or_else()`**：
> `get_mut(index)` 返回 `Option<&mut T>`。用 `ok_or_else` 把 `None` 转为 `Err`。
> `ok_or_else` 使用闭包（惰性求值）——只在 `None` 时才执行 `|| TaskError::NotFound(...)`，
> 避免每次都构造错误对象。

## 5.7 删除任务（Delete）

```rust
    pub fn delete_task(&self, id: &str) -> Result<Task> {
        let mut tasks = self.store.load()?;
        let del_index = Self::find_task_by_id(&tasks, id)?;

        let deleted_task = tasks.remove(del_index);  // 从 Vec 中移除并返回
        self.store.save(&tasks)?;
        Ok(deleted_task)
    }
```

## 5.8 内部辅助方法

### 按 ID 查找

```rust
    pub fn find_task_by_id(tasks: &Vec<Task>, id: &str) -> Result<usize> {
        let mut found: Option<usize> = None;

        for (i, v) in tasks.iter().enumerate() {
            if v.id.starts_with(id) {
                if found.is_some() {
                    return Err(TaskError::AmbiguousId(id.to_string()).into());
                }
                found = Some(i);
            }
        }

        match found {
            Some(i) => Ok(i),
            None => Err(TaskError::NotFound(id.to_string()).into()),
        }
    }
```

**设计要点：**

- **前缀匹配**：`starts_with(id)` 让用户只需输入 UUID 的前几位
- **多义检测**：如果多个任务匹配同一前缀，返回 `AmbiguousId` 错误
- **返回索引**：而非引用，避免生命周期问题

### 预览任务（供删除确认用）

```rust
    pub fn get_task_by_id(&self, id: &str) -> Result<Task> {
        let tasks = self.store.load()?;
        let i = Self::find_task_by_id(&tasks, id)?;
        Ok(tasks[i].clone())
    }
```

## 5.9 验证

在 `main.rs` 中临时添加：

```rust
fn main() {
    let service = TaskService::new().unwrap();

    // 测试添加
    let task = service.add_task("测试任务", None, None, vec![], None).unwrap();
    println!("创建了：{}", task);

    // 测试列表
    let tasks = service.list_tasks(None, None, None).unwrap();
    println!("共 {} 个任务", tasks.len());
}
```

```bash
cargo run
# 应该输出创建成功和任务数量
```

## 本章小结

| 概念 | 你学到了 |
|------|---------|
| `Uuid::new_v4()` | 生成唯一 ID |
| `Option` 模式 | `map()`、`unwrap_or()`、`if let Some()` |
| 迭代器 | `.iter().filter().collect()` 链式处理 |
| `get_mut()` | 获取可变引用修改 Vec 中的元素 |
| `ok_or_else()` | 惰性错误构造，避免不必要的计算 |
| `chars().count()` | 按字符计数（非字节），正确处理中文 |
| 前缀匹配 | `starts_with` 让用户不用输入完整 UUID |

---

[← 上一章](./04_cli_parsing.md) | [返回目录](./00_overview.md) | [下一章 →](./06_display_layer.md)

---

📧 联系作者：pebblerwon@qq.com
