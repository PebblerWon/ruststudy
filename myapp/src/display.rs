use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, ContentArrangement, Table};

use crate::models::{Priority, Status, Task};

pub fn print_task_table(tasks: &[Task]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL) // 1. 加载 Unicode 全边框
        .set_content_arrangement(ContentArrangement::Dynamic) // 2. 自适应填满终端
        .set_header(vec!["ID", "标题", "状态", "优先级", "标签", "截止日期"]);

    for task in tasks {
        let due_str = task.due_date.map_or("--".to_string(), |d| d.to_string());
        let tag_str = if task.tags.is_empty() {
            "-".to_string()
        } else {
            task.tags.join(",")
        };
        table.add_row(vec![
            &task.id[..task.id.len().min(8)],
            &task.title,
            // &task.status.to_string(),
            // &task.priority.to_string(),
            &format_status(&task.status),
            &format_priority(&task.priority),
            &tag_str,
            &due_str,
        ]);
    }
    println!("{table}");
}

pub fn format_status(status: &Status) -> String {
    match status {
        Status::Done => "已完成".green().strikethrough(),
        Status::InProgress => "进行中".blue(),
        Status::Todo => "未完成".bright_black(),
    }
    .to_string()
}

pub fn format_priority(priority: &Priority) -> String {
    match priority {
        Priority::High => "高".red(),
        Priority::Medium => "中".yellow(),
        Priority::Low => "低".green(),
    }
    .to_string()
}

pub fn print_success(msg: &str) {
    println!("✓ {}", msg.green());
}

pub fn print_warning(msg: &str) {
    println!("⚠ {}", msg.yellow());
}

pub fn print_error(msg: &str) {
    eprintln!("✗ 错误：{}", msg.red());
}

pub fn print_info(msg: &str) {
    println!("{}", msg); // stdout，无前缀无颜色
}
