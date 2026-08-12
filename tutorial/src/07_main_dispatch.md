# 第 7 章：入口串联与调度

## 本章目标

- 在 `main.rs` 中串联所有模块
- 用 `match` 分发子命令到对应 service 方法
- 实现删除确认的交互流程
- 统一错误处理和退出码

## 7.1 main.rs 的职责

`main.rs` 的职责极薄，只做三件事：

1. 解析 CLI 参数
2. 构造 Service 实例
3. 分发命令 → 调用 Service → 调用 Display

**不做**业务校验、不做 IO、不做格式化。

## 7.2 完整实现

```rust
mod cli;
mod display;
mod error;
mod models;
mod service;
mod store;

use crate::{
    cli::{Cli, Commands},
    display::{
        print_error, print_info, print_stats, print_success, print_task_table, print_warning,
    },
    service::TaskService,
};
use anyhow::{Context, Result};
use clap::Parser;

fn main() {
    if let Err(e) = run() {
        print_error(&format!("{e:#}"));
        std::process::exit(1)
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let service = TaskService::new().context("初始化任务服务失败")?;

    match cli.command {
        Commands::Add {
            title,
            description,
            priority,
            tag,
            due,
        } => {
            let tags: Vec<&str> = tag.iter().map(String::as_str).collect();
            let desc = description.as_deref();
            let task = service.add_task(&title, desc, Some(priority), tags, due.as_deref())?;
            print_success(&format!("任务创建成功：{}", task));
        }

        Commands::List {
            status,
            priority,
            tag,
        } => {
            let tasks = service.list_tasks(status, priority, tag.as_deref())?;
            if tasks.is_empty() {
                print_info("暂无任务");
            } else {
                print_task_table(&tasks);
            }
        }

        Commands::Update {
            id,
            title,
            status,
            priority,
            tag,
        } => {
            let tags: Vec<&str> = tag.iter().map(String::as_str).collect();
            let task =
                service.update_task(&id, title.as_deref(), status, priority, None, tags, None)?;
            print_success(&format!("任务已更新：{}", task));
        }

        Commands::Delete { id, force } => {
            if !force {
                let task = service.get_task_by_id(&id)?;
                print_warning(&format!(
                    "确认删除任务 \"{}\" ({})?(y/n)",
                    task.title,
                    &task.id[..task.id.len().min(8)]
                ));

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "y" {
                    print_info("已取消删除");
                    return Ok(());
                }
            }
            let deleted = service.delete_task(&id)?;
            print_success(&format!("已删除任务：{} ({})", deleted.title, deleted.id));
        }

        Commands::Search { keyword } => {
            let res = service.search_task(keyword.as_str())?;
            if res.is_empty() {
                print_info("未找到匹配任务");
            } else {
                println!("搜索到 {} 条结果：", res.len());
                print_task_table(&res);
            }
        }

        Commands::Stats => {
            let task_stats = service.get_stats()?;
            print_stats(&task_stats);
        }

        Commands::Export { format, output } => {
            let csv_data = service.export_tasks(&format)?;
            match output {
                Some(path) => {
                    std::fs::write(&path, csv_data.as_bytes())
                        .with_context(|| format!("写入文件失败：{}", path))?;
                    print_success(&format!("已导出任务到{}", path));
                }
                None => {
                    print!("{csv_data}");
                }
            }
        }
    }
    Ok(())
}
```

## 7.3 逐段解析

### 7.3.1 错误处理入口

```rust
fn main() {
    if let Err(e) = run() {
        print_error(&format!("{e:#}"));
        std::process::exit(1)
    }
}
```

| 语法 | 含义 |
|------|------|
| `if let Err(e) = run()` | 如果 `run()` 返回错误，绑定到 `e` |
| `{e:#}` | anyhow 的 alternate Display，打印完整错误链 |
| `std::process::exit(1)` | 以退出码 1 退出（表示失败） |

> **为什么 `main` 不直接返回 `Result`？**
> 虽然 Rust 支持 `fn main() -> Result<()>`，但那样输出的错误信息不够友好。
> 自己处理可以控制格式（`print_error` 带红色前缀）。

### 7.3.2 context() 附加上下文

```rust
let service = TaskService::new().context("初始化任务服务失败")?;
```

`anyhow::Context::context()` 给错误附加额外信息。如果 `new()` 失败，
错误链会变成：

```
初始化任务服务失败: 无法获取父目录
```

### 7.3.3 类型转换

```rust
let tags: Vec<&str> = tag.iter().map(String::as_str).collect();
let desc = description.as_deref();
```

**为什么需要转换？**

- CLI 解析出的 `tag` 是 `Vec<String>`（拥有所有权）
- `service.add_task` 接收 `Vec<&str>`（借用，更灵活）
- `as_deref()` 把 `Option<String>` 转为 `Option<&str>`

### 7.3.4 删除确认流程

```rust
Commands::Delete { id, force } => {
    if !force {
        // 1. 预览任务（提前暴露 NotFound 错误）
        let task = service.get_task_by_id(&id)?;

        // 2. 显示确认提示
        print_warning(&format!(
            "确认删除任务 \"{}\" ({})?(y/n)",
            task.title,
            &task.id[..task.id.len().min(8)]
        ));

        // 3. 读取用户输入
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        // 4. 判定
        if input.trim().to_lowercase() != "y" {
            print_info("已取消删除");
            return Ok(());
        }
    }
    // 5. 执行删除
    let deleted = service.delete_task(&id)?;
    print_success(&format!("已删除任务：{} ({})", deleted.title, deleted.id));
}
```

**交互流程：**

```
taskflow delete a1b2c3d4
    ↓
⚠ 确认删除任务 "学习Rust" (a1b2c3d4)?(y/n)
    ↓
用户输入 y → 执行删除 → ✓ 已删除任务：学习Rust (a1b2c3d4-...)
用户输入 n → 已取消删除
--force    → 跳过确认，直接删除
```

> **`task.id.len().min(8)`**：安全截断。如果 ID 不足 8 位（测试 mock 数据），
> 直接 `&id[..8]` 会 panic。用 `min(8)` 取较小值避免越界。

## 7.4 Dispatch 映射表

| 子命令 | 调用的 service 方法 | 输出函数 |
|--------|-------------------|---------|
| `Add` | `add_task(...)` | `print_success` |
| `List` | `list_tasks(...)` | `print_task_table` 或 `print_info` |
| `Update` | `update_task(...)` | `print_success` |
| `Delete` | `get_task_by_id` + `delete_task` | `print_warning` + `print_success` |
| `Search` | `search_task(...)` | `print_task_table` 或 `print_info` |
| `Stats` | `get_stats()` | `print_stats` |
| `Export` | `export_tasks(...)` | `print_success` 或 `print!` |

## 7.5 验证

```bash
# 完整流程测试
cargo run -- add "学习Rust" -p high
cargo run -- list
cargo run -- update <id前8位> --status done
cargo run -- delete <id前8位>
# 输入 y 确认

# 异常路径
cargo run -- add ""
# ✗ 错误：标题不能为空

cargo run -- update xx --status done
# ✗ 错误：任务不存在：xx
```

## 7.6 错误处理收敛总结

```
service 层：抛出 TaskError（精确的业务错误）
    ↓ 通过 #[from] 自动转为 anyhow::Error
main 层：用 ? 传播所有错误到 run() 的返回值
    ↓
main()：捕获 Err → print_error 格式化输出 → exit(1)
```

| 层 | 错误类型 | 处理方式 |
|----|---------|---------|
| `error.rs` | `TaskError`（thiserror） | 定义精确的业务错误 |
| `service.rs` | 返回 `anyhow::Result` | 用 `?` 传播，`TaskError` 自动转换 |
| `main.rs` | `anyhow::Result` | 顶层 `if let Err` 统一输出 |

## 本章小结

| 概念 | 你学到了 |
|------|---------|
| `match` 分发 | 枚举穷尽匹配，新增子命令时编译器提醒 |
| `?` 传播 | 一行 `?` 替代大段 try/catch |
| `.context()` | 给错误附加上下文信息 |
| `{e:#}` | anyhow alternate Display，打印完整错误链 |
| 交互式确认 | `stdin().read_line()` 读取用户输入 |
| 安全截断 | `.len().min(8)` 避免越界 panic |

---

[← 上一章](./06_display_layer.md) | [返回目录](./00_overview.md) | [下一章 →](./08_testing.md)

---

📧 联系作者：pebblerwon@qq.com
