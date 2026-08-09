mod cli;
mod error;
mod models;
mod service;
mod store;

use crate::{
    cli::{Cli, Commands},
    service::TaskService,
};
use anyhow::{Context, Result};
use clap::Parser;

fn main() {
    if let Err(e) = run() {
        eprintln!("✗ 错误：{e:#}");
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
            println!("✓ 任务创建成功：{}", task);
        }

        Commands::List {
            status,
            priority,
            tag,
        } => {
            let tasks = service.list_tasks(status, priority, tag.as_deref())?;

            if tasks.is_empty() {
                println!("暂无任务")
            } else {
                for i in &tasks {
                    println!("{i}");
                }
            }
        }
        Commands::Update {
            id,
            title,
            status,
            priority,
        } => {
            let task =
                service.update_task(&id, title.as_deref(), status, priority, None, None, None)?;
            println!("✓ 任务已更新：{}", task);
        }
        Commands::Delete { id, force:_ } => {
            let deleted = service.delete_task(&id)?;

            println!("✓ 已删除任务：{} ({})", deleted.title, deleted.id);
        }
        // 阶段二/三实现
        Commands::Search { .. } | Commands::Stats | Commands::Export { .. } => {
            anyhow::bail!("该命令将在后续阶段实现");
        }
    }
    Ok(())
}
