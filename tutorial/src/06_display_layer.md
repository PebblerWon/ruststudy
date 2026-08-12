# 第 6 章：展示层——表格与彩色输出

## 本章目标

- 用 `comfy-table` 渲染 Unicode 表格
- 用 `colored` 实现终端彩色输出
- 理解"表格内不能用 colored"的陷阱
- 实现统一的消息输出规范

## 6.1 两个库，两种场景

| 库 | 用途 | 用在哪里 |
|----|------|---------|
| `comfy-table` | 表格渲染 | 任务列表、统计面板 |
| `colored` | 文本着色 | 成功/错误/警告消息 |

> **重要规则：表格内颜色必须用 `comfy-table` 原生 API，不能用 `colored`！**
> 原因见 [6.5 节](#65-陷阱表格内不能用-colored)。

## 6.2 表格渲染基础

创建 `src/display.rs`：

```rust
use colored::Colorize;
use comfy_table::{
    presets::UTF8_FULL, Attribute, Cell, Color, ContentArrangement, Table,
};
use crate::models::{Priority, Status, Task, TaskStats};

pub fn print_task_table(tasks: &[Task]) {
    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)                        // 1. Unicode 全边框
        .set_content_arrangement(ContentArrangement::Dynamic)  // 2. 自适应列宽
        .set_header(vec!["ID", "标题", "状态", "优先级", "标签", "截止日期"]);

    for task in tasks {
        let due_str = task.due_date.map_or("--".to_string(), |d| d.to_string());
        let tag_str = if task.tags.is_empty() {
            "-".to_string()
        } else {
            task.tags.join(",")
        };

        table.add_row(vec![
            Cell::new(&task.id[..task.id.len().min(8)]),  // 安全截断 ID
            Cell::new(&task.title),
            status_cell(&task.status),       // 带颜色的状态 Cell
            priority_cell(&task.priority),   // 带颜色的优先级 Cell
            Cell::new(&tag_str),
            Cell::new(&due_str),
        ]);
    }

    println!("{table}");
}
```

### 关键 API 解读

| API | 作用 |
|-----|------|
| `load_preset(UTF8_FULL)` | 加载 Unicode 边框样式（横线、竖线、交叉点） |
| `ContentArrangement::Dynamic` | 根据终端宽度和内容自动调整列宽 |
| `Cell::new(text)` | 创建一个表格单元格 |
| `.fg(Color::Red)` | 设置前景色（文字颜色） |
| `.add_attribute(Attribute::CrossedOut)` | 添加删除线效果 |

### 输出效果

```
╭──────────┬────────────────┬────────┬────────┬───────────┬────────────╮
│ ID       │ 标题           │ 状态   │ 优先级 │ 标签      │ 截止日期   │
├──────────┼────────────────┼────────┼────────┼───────────┼────────────┤
│ a1b2c3d4 │ 学习Rust所有权 │ 未完成 │ 高     │ rust,study│ 2026-09-01 │
│ b2c3d4e5 │ 写周报         │ 已完成 │ 中     │ -         │ --         │
╰──────────┴────────────────┴────────┴────────┴───────────┴────────────╯
```

## 6.3 带颜色的单元格

```rust
/// 状态单元格：不同状态不同颜色
fn status_cell(status: &Status) -> Cell {
    match status {
        Status::Done => Cell::new("已完成")
            .fg(Color::Green)
            .add_attribute(Attribute::CrossedOut),  // 绿色 + 删除线
        Status::InProgress => Cell::new("进行中").fg(Color::Blue),   // 蓝色
        Status::Todo => Cell::new("未完成").fg(Color::DarkGrey),     // 灰色
    }
}

/// 优先级单元格
fn priority_cell(priority: &Priority) -> Cell {
    match priority {
        Priority::High => Cell::new("高").fg(Color::Red),      // 红色
        Priority::Medium => Cell::new("中").fg(Color::Yellow), // 黄色
        Priority::Low => Cell::new("低").fg(Color::Green),     // 绿色
    }
}
```

**颜色规则（来自 PRD F7）：**

| 状态 | 颜色 | 附加效果 |
|------|------|---------|
| Done | 绿色 | 删除线 |
| InProgress | 蓝色 | 无 |
| Todo | 灰色 | 无 |

| 优先级 | 颜色 |
|--------|------|
| High | 红色 |
| Medium | 黄色 |
| Low | 绿色 |

## 6.4 消息输出函数

```rust
/// 成功消息：✓ 前缀 + 绿色 → stdout
pub fn print_success(msg: &str) {
    println!("✓ {}", msg.green());
}

/// 错误消息：✗ 前缀 + 红色 → stderr
pub fn print_error(msg: &str) {
    eprintln!("✗ 错误：{}", msg.red());
}

/// 警告消息：⚠ 前缀 + 黄色 → stdout
pub fn print_warning(msg: &str) {
    println!("⚠ {}", msg.yellow());
}

/// 信息消息：无前缀无颜色 → stdout
pub fn print_info(msg: &str) {
    println!("{}", msg);
}
```

### stdout vs stderr

| 函数 | 输出流 | 为什么 |
|------|--------|--------|
| `print_success` | stdout | 正常输出，可以被重定向到文件 |
| `print_error` | **stderr** | 错误信息即使用户重定向了 stdout 也要能看到 |
| `print_warning` | stdout | 警告是正常流程的一部分 |
| `print_info` | stdout | 普通信息 |

> **为什么要区分 stdout/stderr？**
> ```bash
> # 用户想把任务列表导出到文件
> taskflow list > tasks.txt
> # 如果此时有错误，错误应该显示在终端（stderr），而不是写进 tasks.txt
> ```

## 6.5 陷阱：表格内不能用 colored

**错误做法：**

```rust
// ❌ 错误：在表格内使用 colored
use colored::Colorize;
table.add_row(vec![
    Cell::new("已完成".green()),  // ANSI 转义码会被计入字符宽度！
]);
```

**为什么？**

`colored` 通过在字符串中嵌入 ANSI 转义码实现着色：

```
正常文本: "已完成"     → 3 个字符
着色文本: "\x1b[32m已完成\x1b[0m" → comfy-table 认为是 3 + 9 + 4 = 16 个字符
```

`comfy-table` 默认按字符串长度计算列宽，ANSI 转义码被算进去了，导致列宽超宽、表格错位。

**正确做法：**

```rust
// ✅ 正确：用 comfy-table 原生 Cell 样式 API
Cell::new("已完成").fg(Color::Green)
```

`comfy-table` 的 `Color` 和 `Attribute` 是样式信息，与文本内容分离，
计算列宽时只算可见字符。

## 6.6 统计面板

```rust
pub fn print_stats(stats: &TaskStats) {
    // 1. 概览行
    let rate = format!("{:.1}%", stats.completion_rate * 100.0);
    println!("总任务数：{}    已完成率：{}", stats.total, rate);

    // 2. 状态分布表
    let mut status_table = Table::new();
    status_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["状态", "数量", "占比"]);

    status_table.add_row(vec![
        status_cell(&Status::Todo),
        Cell::new(stats.todo),
        Cell::new(format_pct(stats.todo, stats.total)),
    ]);
    status_table.add_row(vec![
        status_cell(&Status::InProgress),
        Cell::new(stats.in_progress),
        Cell::new(format_pct(stats.in_progress, stats.total)),
    ]);
    status_table.add_row(vec![
        status_cell(&Status::Done),
        Cell::new(stats.done),
        Cell::new(format_pct(stats.done, stats.total)),
    ]);
    println!("{status_table}");

    // 3. 优先级分布表
    let mut prio_table = Table::new();
    prio_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["优先级", "数量"]);
    prio_table.add_row(vec![priority_cell(&Priority::High), Cell::new(stats.high)]);
    prio_table.add_row(vec![priority_cell(&Priority::Medium), Cell::new(stats.medium)]);
    prio_table.add_row(vec![priority_cell(&Priority::Low), Cell::new(stats.low)]);
    println!("{prio_table}");

    // 4. 逾期提示
    if stats.overdue > 0 {
        print_warning(&format!("逾期任务：{} 个", stats.overdue));
    }
}

/// 计算占比字符串，防除零
fn format_pct(part: usize, total: usize) -> String {
    if total == 0 {
        "0.0%".to_string()
    } else {
        format!("{:.1}%", part as f64 / total as f64 * 100.0)
    }
}
```

### 输出效果

```
总任务数：5    已完成率：40.0%
╭────────┬──────┬───────╮
│ 状态   │ 数量 │ 占比  │
├────────┼──────┼───────┤
│ 未完成 │ 2    │ 40.0% │
│ 进行中 │ 1    │ 20.0% │
│ 已完成 │ 2    │ 40.0% │
╰────────┴──────┴───────╯
╭────────┬──────╮
│ 优先级 │ 数量 │
├────────┼──────┤
│ 高     │ 1    │
│ 中     │ 3    │
│ 低     │ 1    │
╰────────┴──────╯
⚠ 逾期任务：1 个
```

## 6.7 输出规范总结

| 函数 | 通道 | 前缀 | 颜色 | 场景 |
|------|------|------|------|------|
| `print_success` | stdout | `✓ ` | 绿 | 操作成功 |
| `print_error` | stderr | `✗ 错误：` | 红 | 操作失败 |
| `print_warning` | stdout | `⚠ ` | 黄 | 删除确认 |
| `print_info` | stdout | 无 | 无 | 中性信息 |
| `print_task_table` | stdout | 无 | 表格列自带 | 任务列表 |
| `print_stats` | stdout | 无 | 表格列自带 | 统计面板 |

## 本章小结

| 概念 | 你学到了 |
|------|---------|
| `comfy-table` | Unicode 表格渲染、自适应列宽 |
| `Cell` 样式 API | 表格内用原生 Color/Attribute 着色 |
| `colored` | 非表格场景的文本着色 |
| stdout vs stderr | 错误走 stderr，正常输出走 stdout |
| 陷阱 | 表格内禁用 colored（ANSI 转义码导致列宽错位） |

---

[← 上一章](./05_service_layer.md) | [返回目录](./00_overview.md) | [下一章 →](./07_main_dispatch.md)

---

📧 联系作者：pebblerwon@qq.com
