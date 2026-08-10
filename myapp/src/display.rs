use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Attribute, Cell, Color, ContentArrangement, Table};

use crate::models::{Priority, Status, Task, TaskStats};

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

pub fn print_stats(stats: &TaskStats) {
    let rate = format!("{:.1}%", stats.completion_rate * 100.0);
    let total = stats.total;
    println!("总任务数：{}    已完成率：{}", total, rate);
    let mut status_table = Table::new();

    status_table
        .load_preset(UTF8_FULL) // 1. 加载 Unicode 全边框
        .set_content_arrangement(ContentArrangement::Dynamic) // 2. 自适应填满终端
        .set_header(vec!["状态", "数量", "占比"]);

    status_table.add_row(vec![
        status_cell(&Status::Todo),
        Cell::new(stats.todo),
        Cell::new(format_pct(stats.todo, total)),
    ]);
    status_table.add_row(vec![
        status_cell(&Status::InProgress),
        Cell::new(stats.in_progress),
        Cell::new(format_pct(stats.in_progress, total)),
    ]);
    status_table.add_row(vec![
        status_cell(&Status::Done),
        Cell::new(stats.done),
        Cell::new(format_pct(stats.done, total)),
    ]);
    println!("{status_table}");
    let mut prio_table = Table::new();
    prio_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["优先级", "数量"]);
    prio_table.add_row(vec![priority_cell(&Priority::High), Cell::new(stats.high)]);
    prio_table.add_row(vec![
        priority_cell(&Priority::Medium),
        Cell::new(stats.medium),
    ]);
    prio_table.add_row(vec![priority_cell(&Priority::Low), Cell::new(stats.low)]);
    println!("{prio_table}");

    if stats.overdue > 0 {
        print_warning(&format!("逾期任务：{} 个", stats.overdue));
    }
}

fn format_pct(part: usize, total: usize) -> String {
    if total == 0 {
        "0.0%".to_string()
    } else {
        format!("{:.1}%", part as f64 / total as f64 * 100.0)
    }
}
fn status_cell(status: &Status) -> Cell {
    match status {
        Status::Done => Cell::new("已完成")
            .fg(Color::Green)
            .add_attribute(Attribute::CrossedOut),
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
