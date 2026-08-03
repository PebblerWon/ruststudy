mod models;
mod store;
use crate::models::{Task, Status, Priority};
use chrono::{DateTime, NaiveDate, Utc };

fn main() {
    let task = Task { 
        id:String::from("1"),
        title: "学习Rust所有权".to_string(),
        description:None,
        status: Status::InProgress,
        priority: Priority::High,
        due_date: Some(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap()),
        tags: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    println!("{}", task);
    let task = Task { 
        id:String::from("1"),
        title: "学习Rust所有权".to_string(),
        description:None,
        status: Status::InProgress,
        priority: Priority::High,
        due_date:None,
        tags: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    println!("{}", task);
}
