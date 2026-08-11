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
