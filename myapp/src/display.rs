use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Attribute, Cell, Color, ContentArrangement, Table};

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
            Cell::new(&task.id[..task.id.len().min(8)]),
            Cell::new(&task.title),
            status_cell(&task.status),
            priority_cell(&task.priority),
            Cell::new(&tag_str),
            Cell::new(&due_str),
        ]);
    }
    println!("{table}");
}

fn status_cell(status: &Status) -> Cell {
    match status {
        Status::Done => {
            Cell::new("已完成").fg(Color::Green).add_attribute(Attribute::CrossedOut)
        }
        Status::InProgress => Cell::new("进行中").fg(Color::Blue),
        Status::Todo => Cell::new("未完成").fg(Color::DarkGrey),
    }
}

fn priority_cell(priority: &Priority) -> Cell {
    match priority {
        Priority::High => Cell::new("高").fg(Color::Red),
        Priority::Medium => Cell::new("中").fg(Color::Yellow),
        Priority::Low => Cell::new("低").fg(Color::Green),
    }
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
