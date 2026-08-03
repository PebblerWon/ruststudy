use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{NaiveDate, DateTime, Utc};

use super::{Status, Priority};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Task {
  pub id: String,
  pub title: String,
  pub description: Option<String>,
  pub status: Status,
  pub priority: Priority,
  pub tags: Vec<String>,
  pub due_date: Option<NaiveDate>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl fmt::Display for Task {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.due_date {
        Some(d)=>{
          write!(
            f,
            "[{}] {} ({}) - {}",
            self.priority,self.title,self.status,self.due_date.unwrap().format("%Y-%m-%d")
          )
        },
        _=>{
          write!(
            f,
            "[{}] {} ({})",
            self.priority,self.title,self.status
          )
        }
    }
    
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::TimeZone;
  #[test]
  fn testSe(){
    let task = Task { 
        id:String::from("1"),
        title: "学习Rust所有权".to_string(),
        description:None,
        status: Status::InProgress,
        priority: Priority::High,
        due_date: Some(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap()),
        tags: vec![],
        created_at: Utc.with_ymd_and_hms(2026, 8, 15, 10, 30, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 8, 15, 10, 30, 0).unwrap(),
        // ... 其他字段
    };
    assert_eq!(serde_json::to_string(&task).unwrap(),
       "{\"id\":\"1\",\"title\":\"学习Rust所有权\",\"description\":null,\"status\":\"in_progress\",\"priority\":\"high\",\"tags\":[],\"due_date\":\"2026-08-15\",\"created_at\":\"2026-08-15T10:30:00Z\",\"updated_at\":\"2026-08-15T10:30:00Z\"}"
    )
  }

  #[test]
  fn testDe(){
    let a =  "{\"id\":\"1\",\"title\":\"学习Rust所有权\",\"description\":null,\"status\":\"in_progress\",\"priority\":\"high\",\"tags\":[],\"due_date\":\"2026-08-15\",\"created_at\":\"2026-08-15T10:30:00Z\",\"updated_at\":\"2026-08-15T10:30:00Z\"}";
    let b:Task = serde_json::from_str(a).unwrap();
    assert_eq!(b.priority, Priority::High);
    assert_eq!(b.status, Status::InProgress);
    assert_eq!(b.due_date,Some(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap()));
    assert_eq!(b.created_at,Utc.with_ymd_and_hms(2026, 8, 15, 10, 30, 0).unwrap());
  }
}