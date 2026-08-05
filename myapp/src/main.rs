mod models;
mod store;
mod cli;
use crate::{cli::Cli, models::{Priority, Status, Task}, store::{JsonFileStore,Store}};
use chrono::{NaiveDate, Utc };
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    let task1 = Task { 
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
    println!("{}", task1);
    let task2 = Task { 
        id:String::from("2"),
        title: "学习Rust所有权".to_string(),
        description:None,
        status: Status::InProgress,
        priority: Priority::High,
        due_date:None,
        tags: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    println!("{}", task2);
    let t = JsonFileStore::new().unwrap();
    let r = t.load().unwrap();
    println!("{:?}",r);
    let tasks = vec![task1,task2];
    t.save(&tasks).unwrap();
    
}
