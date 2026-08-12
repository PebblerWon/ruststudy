# 第 9 章：增强功能——搜索、统计、CSV 导出

## 本章目标

- 实现关键字搜索（大小写不敏感）
- 实现统计面板（状态/优先级分布、完成率、逾期检测）
- 实现 CSV 导出（含 BOM、中文表头）
- 学习 `TaskCsvRow` 适配层设计

## 9.1 搜索功能

### 需求

按关键字搜索任务标题和描述，大小写不敏感。

### 实现

在 `service.rs` 中添加：

```rust
pub fn search_task(&self, keyword: &str) -> Result<Vec<Task>> {
    let tasks = self.store.load()?;
    let keyword_lower = keyword.to_lowercase();

    let res = tasks
        .into_iter()
        .filter(|i| {
            // 标题匹配
            let title_match = i.title.to_lowercase().contains(&keyword_lower);
            // 描述匹配（Option 安全处理）
            let desc_match = i
                .description
                .as_deref()
                .map_or(false, |d| d.to_lowercase().contains(&keyword_lower));
            title_match || desc_match
        })
        .collect();

    Ok(res)
}
```

### 关键语法点

| 语法 | 含义 |
|------|------|
| `to_lowercase()` | 转小写，实现大小写不敏感匹配 |
| `.contains()` | 子串包含检查 |
| `as_deref()` | `Option<String>` → `Option<&str>` |
| `map_or(false, \|d\| ...)` | `None` → false，`Some(d)` → 执行闭包 |
| `\|\|` | 逻辑或：标题或描述任一命中即保留 |

### 边界行为

| 场景 | 行为 |
|------|------|
| keyword 为空字符串 | `"".contains("")` 为 true，返回所有任务 |
| 标题和描述都命中 | 只返回一次（`filter` 不会重复） |
| 中文关键字 | `to_lowercase()` 对中文无影响，正常工作 |

## 9.2 统计功能

### 统计结构体

在 `models/task.rs` 中：

```rust
#[derive(Default, Debug)]
pub struct TaskStats {
    pub total: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub done: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub overdue: usize,        // 逾期任务数
    pub completion_rate: f64,  // 完成率 0.0~1.0
}
```

> **`#[derive(Default)]`**：所有 `usize`/`f64` 字段默认为 0/0.0。
> 可以用 `TaskStats { total: 5, ..Default::default() }` 只设置部分字段。

### 统计逻辑

```rust
pub fn get_stats(&self) -> Result<TaskStats> {
    let tasks = self.store.load()?;
    let total = tasks.len();

    let mut stats = TaskStats {
        total,
        ..Default::default()
    };

    // 提到循环外，避免每次迭代重复调用系统时间
    let today = Some(Utc::now().date_naive());

    for task in &tasks {
        // 按状态计数
        match task.status {
            Status::Done => stats.done += 1,
            Status::InProgress => stats.in_progress += 1,
            Status::Todo => stats.todo += 1,
        }
        // 按优先级计数
        match task.priority {
            Priority::High => stats.high += 1,
            Priority::Medium => stats.medium += 1,
            Priority::Low => stats.low += 1,
        }

        // 逾期判定：有截止日期 + 已过期 + 未完成
        let overdue = task.due_date.is_some()
            && task.due_date < today
            && task.status != Status::Done;
        if overdue {
            stats.overdue += 1;
        }
    }

    // 防除零
    stats.completion_rate = if total == 0 {
        0.0
    } else {
        stats.done as f64 / total as f64
    };

    Ok(stats)
}
```

### 逾期判定逻辑

```
逾期 = 有截止日期(is_some) + 截止日期早于今天(< today) + 状态不是已完成(!= Done)
```

| 场景 | 是否逾期 | 原因 |
|------|---------|------|
| due_date = Some(昨天), status = Todo | 是 | 过期且未完成 |
| due_date = Some(今天), status = Todo | 否 | 严格 `<`，等于不算 |
| due_date = None, status = Todo | 否 | `is_some()` 守卫排除 |
| due_date = Some(昨天), status = Done | 否 | `!= Done` 排除已完成 |
| due_date = Some(明天), status = Todo | 否 | 未到期 |

> **为什么 `is_some()` 守卫是必须的？**
> Rust 中 `None < Some(x)` 为 `true`！如果不先检查 `is_some()`，
> 没有截止日期的任务会被误判为逾期。

## 9.3 CSV 导出功能

### 9.3.1 适配结构体 TaskCsvRow

`Task` 不能直接序列化为 CSV，因为：
- `Option<NaiveDate>` → CSV 需要空串
- `Vec<String>` → CSV 需要拼接为单个字符串
- `DateTime<Utc>` → CSV 需要文本格式
- 枚举 → CSV 需要中文

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskCsvRow {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "标题")]
    pub title: String,
    #[serde(rename = "描述")]
    pub description: String,     // Option<String> → 空串表示 None
    #[serde(rename = "状态")]
    pub status: String,          // Status::to_string() → "待办"/"进行中"/"已完成"
    #[serde(rename = "优先级")]
    pub priority: String,        // Priority::to_string() → "低"/"中"/"高"
    #[serde(rename = "标签")]
    pub tags: String,            // Vec<String> → join(";")
    #[serde(rename = "截止日期")]
    pub due_date: String,        // Option<NaiveDate> → 空串
    #[serde(rename = "创建时间")]
    pub created_at: String,      // DateTime → RFC 3339 格式
    #[serde(rename = "更新时间")]
    pub updated_at: String,
}

impl From<&Task> for TaskCsvRow {
    fn from(t: &Task) -> Self {
        TaskCsvRow {
            id: t.id.clone(),
            title: t.title.clone(),
            description: t.description.clone().unwrap_or_default(),
            status: t.status.to_string(),
            priority: t.priority.to_string(),
            tags: t.tags.join(";"),
            due_date: t.due_date.map_or(String::new(), |d| d.to_string()),
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        }
    }
}
```

### 设计要点

| 决策 | 为什么 |
|------|--------|
| 与 `Task` 分离 | 不污染 Task 的 JSON serde 定义 |
| `#[serde(rename = "中文")]` | csv writer 自动写中文表头 |
| 同时 derive `Deserialize` | 测试可以反向解析验证列值 |
| tags 用 `;` 拼接 | 避免与 CSV 的 `,` 冲突 |
| `From<&Task>` | 借用转换，调用方保留 Task 所有权 |

### 9.3.2 导出逻辑

```rust
pub fn export_tasks(&self, format: &str) -> Result<String> {
    // 1. 校验格式
    if format.to_lowercase() != "csv" {
        return Err(TaskError::UnsupportedFormat(format.to_string()).into());
    }

    // 2. 加载任务
    let tasks = self.store.load()?;

    // 3. 构造 CSV writer
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)       // 自动写表头
        .from_writer(vec![]);    // 写入内存缓冲区

    // 4. 逐行序列化
    for task in tasks.iter() {
        wtr.serialize(TaskCsvRow::from(task))?;
    }

    // 5. 取出字节数据
    let bytes = wtr.into_inner()?;
    let csv_string = String::from_utf8(bytes)?;

    // 6. 前缀加 UTF-8 BOM（Excel 兼容性）
    Ok(format!("\u{FEFF}{}", csv_string))
}
```

### UTF-8 BOM 是什么？

BOM（Byte Order Mark）是 `\u{FEFF}` 字符，放在文件开头：

```
无 BOM：ID,标题,描述,...     → Excel 按 GBK 解码 → 中文乱码
有 BOM：\u{FEFF}ID,标题,...  → Excel 识别为 UTF-8 → 中文正常
```

> BOM 在文本编辑器中不可见，但 Excel 靠它判断编码。

### CSV 输出示例

```csv
ID,标题,描述,状态,优先级,标签,截止日期,创建时间,更新时间
abc12345,学习Rust,,待办,高,rust;学习,,2026-08-10T10:30:00+00:00,2026-08-10T10:30:00+00:00
def67890,写文档,需要完成,进行中,中,doc,2026-08-15,2026-08-09T08:00:00+00:00,2026-08-10T14:00:00+00:00
```

## 9.4 在 main.rs 中串联

搜索和统计的 main.rs 代码已在第 7 章展示。这里补充导出部分：

```rust
Commands::Export { format, output } => {
    let csv_data = service.export_tasks(&format)?;
    match output {
        Some(path) => {
            // 写入文件
            std::fs::write(&path, csv_data.as_bytes())
                .with_context(|| format!("写入文件失败：{}", path))?;
            print_success(&format!("已导出任务到{}", path));
        }
        None => {
            // 输出到终端
            print!("{csv_data}");
        }
    }
}
```

**设计要点**：Service 只负责生成 CSV 字符串，I/O（写文件 vs stdout）由 main.rs 决定。

## 9.5 验证

```bash
# 搜索
cargo run -- search "Rust"
cargo run -- search "rust"     # 大小写不敏感

# 统计
cargo run -- stats

# 导出
cargo run -- export -o tasks.csv
# 用 Excel 打开 tasks.csv，验证中文表头和内容正确

# 不支持的格式
cargo run -- export --format json
# ✗ 错误：不支持的导出格式：json
```

## 本章小结

| 概念 | 你学到了 |
|------|---------|
| `to_lowercase()` | 大小写不敏感搜索 |
| `map_or(false, ...)` | 安全处理 `Option` 内的匹配 |
| `#[derive(Default)]` | 结构体默认值初始化 |
| `date_naive()` | 获取无时区日期用于比较 |
| 适配层模式 | `TaskCsvRow` 分离 JSON 和 CSV 的序列化需求 |
| `#[serde(rename)]` | 自定义 CSV 表头名称 |
| UTF-8 BOM | Excel 中文兼容的关键 |
| `csv::WriterBuilder` | 内存中构建 CSV，自动写表头 |

---

[← 上一章](./08_testing.md) | [返回目录](./00_overview.md) | [下一章 →](./10_summary.md)

---

📧 联系作者：pebblerwon@qq.com
