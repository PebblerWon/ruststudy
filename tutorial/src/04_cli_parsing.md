# 第 4 章：CLI 参数解析（clap derive）

## 本章目标

- 用 `clap` derive 模式定义子命令
- 理解位置参数和可选参数
- 实现枚举类型的自动解析
- 自动生成 `--help` 帮助文档

## 4.1 clap 是什么？

`clap` 是 Rust 生态中最流行的命令行参数解析库。它的 derive 模式让你用结构体和枚举定义 CLI 接口，编译器自动生成解析逻辑和 `--help` 文本。

```bash
# 用户输入
taskflow add "学习Rust" -p high --due 2026-09-01

# clap 自动解析为
Commands::Add {
    title: "学习Rust",
    priority: Priority::High,
    due: Some("2026-09-01"),
    description: None,
    tag: vec![],
}
```

## 4.2 定义顶层结构

创建 `src/cli.rs`：

```rust
use clap::{Parser, Subcommand};
use crate::models::{Priority, Status};

/// TaskFlow 命令行入口
#[derive(Parser)]
#[command(
    version,
    name = "taskflow",
    about = "命令行任务管理工具",
    long_about = "TaskFlow —— 轻量级命令行任务管理工具\n\n\
        支持任务的增删改查、搜索、统计和导出。\n\
        数据存储在 ~/.taskflow/data.json，使用 UUID 作为任务 ID。",
    after_help = r#"使用示例:
        taskflow add "学习Rust" -p high
        taskflow list --status todo
        taskflow update <id> --status done
        taskflow delete <id> --force
        taskflow search "Rust"
        taskflow stats
        taskflow export -o tasks.csv"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
```

### 逐行解析

| 属性 | 作用 |
|------|------|
| `#[derive(Parser)]` | 启用 clap 的参数解析功能 |
| `version` | 自动从 Cargo.toml 读取版本号 |
| `name = "taskflow"` | 程序名（影响 help 输出） |
| `about` | `taskflow --help` 顶部的一行简介 |
| `long_about` | `taskflow --help` 的多行详细描述 |
| `after_help` | help 底部追加的使用示例 |
| `#[command(subcommand)]` | 标记这个字段是子命令枚举 |

> **doc comment 自动生成 help**：`/// TaskFlow 命令行入口` 这种 `///` 注释会被 clap 读取，
> 自动生成帮助文本。

## 4.3 定义子命令枚举

```rust
#[derive(Subcommand)]
pub enum Commands {
    /// 创建新任务
    Add {
        /// 任务标题
        title: String,
        /// 任务描述
        #[arg(short, long)]
        description: Option<String>,
        /// 优先级（high/medium/low，默认 medium）
        #[arg(short, long, default_value = "medium")]
        priority: Priority,
        /// 标签（逗号分隔，如 -t rust,study）
        #[arg(short, long, value_delimiter = ',')]
        tag: Vec<String>,
        /// 截止日期（YYYY-MM-DD 格式）
        #[arg(long)]
        due: Option<String>,
    },

    /// 列出任务
    List {
        /// 按状态筛选（todo/in_progress/done）
        #[arg(short, long)]
        status: Option<Status>,
        /// 按优先级筛选（high/medium/low）
        #[arg(short, long)]
        priority: Option<Priority>,
        /// 按标签筛选（模糊匹配）
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// 更新任务
    Update {
        /// 任务 ID（支持前缀匹配）
        id: String,
        /// 新标题
        #[arg(long)]
        title: Option<String>,
        /// 新状态（todo/in_progress/done）
        #[arg(long)]
        status: Option<Status>,
        /// 新优先级（high/medium/low）
        #[arg(long)]
        priority: Option<Priority>,
        /// 新标签（逗号分隔，替换原有标签）
        #[arg(short, long, value_delimiter = ',')]
        tag: Vec<String>,
    },

    /// 删除任务
    Delete {
        /// 任务 ID（支持前缀匹配）
        id: String,
        /// 跳过确认提示，直接删除
        #[arg(short, long)]
        force: bool,
    },

    /// 搜索任务
    Search {
        /// 搜索关键字（匹配标题和描述，大小写不敏感）
        keyword: String,
    },

    /// 查看统计
    Stats,

    /// 导出数据
    Export {
        /// 导出格式（csv，默认 csv）
        #[arg(long, default_value = "csv")]
        format: String,
        /// 输出文件路径（不指定则输出到终端）
        #[arg(short, long)]
        output: Option<String>,
    },
}
```

## 4.4 clap 属性详解

### `#[arg(...)]` 常用参数

| 参数 | 作用 | 示例 |
|------|------|------|
| `short` | 生成短选项（单字母，如 `-p`） | `#[arg(short, long)]` → `-p` 和 `--priority` |
| `long` | 生成长选项（如 `--priority`） | 同上 |
| `default_value` | 默认值 | `default_value = "medium"` |
| `value_delimiter` | 值分隔符 | `value_delimiter = ','` → `-t a,b` 解析为 `vec!["a", "b"]` |

### 类型与参数映射

| Rust 类型 | clap 行为 | 命令行示例 |
|-----------|----------|-----------|
| `String` | 必填位置参数 | `taskflow add "标题"` |
| `Option<String>` | 可选参数 | `--due 2026-09-01` 或不写 |
| `Option<Status>` | 可选枚举参数 | `--status done` |
| `Vec<String>` | 可重复/分隔的值 | `-t rust,study` |
| `bool` | 标志位（出现即为 true） | `--force` 或 `-f` |
| `Priority`（带 default_value） | 有默认值的参数 | 不写则为 `medium` |

### doc comment = help 文本

```rust
/// 任务标题
title: String,
```

clap 会把 `/// 任务标题` 作为 `title` 参数的帮助文本：

```
Arguments:
  <TITLE>  任务标题
```

## 4.5 枚举类型的自动解析

`Status` 和 `Priority` 都 derive 了 `clap::ValueEnum`，所以 clap 能自动把命令行字符串转为枚举：

```bash
# 用户输入
taskflow add "test" -p high

# clap 自动转换
"high" → Priority::High
```

转换规则：
- 变体名转为 `snake_case`（`InProgress` → `in_progress`）
- 用户输入不区分大小写
- 输入非法值时自动报错并列出可选项

```bash
$ taskflow add "test" -p invalid
error: invalid value 'invalid' for '--priority <PRIORITY>'
  [possible values: low, medium, high]
```

## 4.6 验证

暂时在 `main.rs` 中添加简单的解析验证：

```rust
mod cli;
mod display;
mod error;
mod models;
mod service;
mod store;

use clap::Parser;
use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();
    println!("解析到的命令：{:?}", cli.command);
}
```

运行测试：

```bash
# 查看自动生成的帮助
$ cargo run -- --help

TaskFlow —— 轻量级命令行任务管理工具

支持任务的增删改查、搜索、统计和导出。
数据存储在 ~/.taskflow/data.json，使用 UUID 作为任务 ID。

Usage: taskflow <COMMAND>

Commands:
  add     创建新任务
  list    列出任务
  update  更新任务
  delete  删除任务
  search  搜索任务
  stats   查看统计
  export  导出数据
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

使用示例:
  taskflow add "学习Rust" -p high
  ...

# 测试子命令帮助
$ cargo run -- add --help

创建新任务

Usage: taskflow add [OPTIONS] <TITLE>

Arguments:
  <TITLE>  任务标题

Options:
  -d, --description <DESCRIPTION>  任务描述
  -p, --priority <PRIORITY>        优先级（high/medium/low，默认 medium） [default: medium]
  -t, --tag <TAG>                  标签（逗号分隔，如 -t rust,study）
      --due <DUE>                  截止日期（YYYY-MM-DD 格式）
  -h, --help                       Print help

# 测试参数解析
$ cargo run -- add "学习Rust" -p high -t rust,study
解析到的命令：Add { title: "学习Rust", description: None, priority: High, tag: ["rust", "study"], due: None }
```

## 4.7 设计要点总结

| 设计决策 | 为什么 |
|---------|--------|
| 用 derive 而非 builder 模式 | 代码即文档，结构清晰，维护成本低 |
| 枚举做子命令 | `match` 穷尽匹配，新增子命令时编译器会提醒处理 |
| doc comment 做 help | 一处定义，两处使用（代码注释 + 帮助文本） |
| `value_delimiter = ','` | 用户不用重复写 `-t`，`-t a,b` 比 `-t a -t b` 更自然 |
| `default_value` | 减少用户输入，`-p` 不写就是 `medium` |

---

[← 上一章](./03_storage_layer.md) | [返回目录](./00_overview.md) | [下一章 →](./05_service_layer.md)

---

📧 联系作者：pebblerwon@qq.com
