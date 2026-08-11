use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::{Priority, Status};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            Some(d) => {
                write!(
                    f,
                    "({})[{}] {} ({}) - {}",
                    &self.id[..8],
                    self.priority,
                    self.title,
                    self.status,
                    self.due_date.unwrap().format("%Y-%m-%d")
                )
            }
            _ => {
                write!(
                    f,
                    "({})[{}] {} ({})",
                    &self.id[..8],
                    self.priority,
                    self.title,
                    self.status
                )
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct TaskStats {
    pub total: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub done: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub overdue: usize,
    pub completion_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    #[test]
    fn testSe() {
        let task = Task {
            id: String::from("1"),
            title: "学习Rust所有权".to_string(),
            description: None,
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
    fn testDe() {
        let a =  "{\"id\":\"1\",\"title\":\"学习Rust所有权\",\"description\":null,\"status\":\"in_progress\",\"priority\":\"high\",\"tags\":[],\"due_date\":\"2026-08-15\",\"created_at\":\"2026-08-15T10:30:00Z\",\"updated_at\":\"2026-08-15T10:30:00Z\"}";
        let b: Task = serde_json::from_str(a).unwrap();
        assert_eq!(b.priority, Priority::High);
        assert_eq!(b.status, Status::InProgress);
        assert_eq!(
            b.due_date,
            Some(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap())
        );
        assert_eq!(
            b.created_at,
            Utc.with_ymd_and_hms(2026, 8, 15, 10, 30, 0).unwrap()
        );
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskCsvRow {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "标题")]
    pub title: String,
    #[serde(rename = "描述")]
    pub description: String,
    #[serde(rename = "状态")]
    pub status: String,
    #[serde(rename = "优先级")]
    pub priority: String,
    #[serde(rename = "标签")]
    pub tags: String,
    #[serde(rename = "截止日期")]
    pub due_date: String,
    #[serde(rename = "创建时间")]
    pub created_at: String,
    #[serde(rename = "更新时间")]
    pub updated_at: String,
}
impl From<&Task> for TaskCsvRow {
    fn from(t: &Task) -> Self {
        TaskCsvRow {
            id: t.id.clone(),
            title: t.title.clone(),
            description: t.description.clone().unwrap_or_default(),
            status: t.status.to_string(),
            priority: t.priority.to_string(),
            tags: t.tags.join(";"),
            due_date: t.due_date.map_or(String::new(), |d| d.to_string()),
            created_at: t.created_at.to_rfc3339(),
            updated_at: t.updated_at.to_rfc3339(),
        }
    }
}
