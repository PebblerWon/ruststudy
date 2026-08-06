use crate::{
    error::TaskError,
    models::{Priority, Status, Task},
    store::{JsonFileStore, Store},
};
use anyhow::{Ok, Result};
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

pub struct TaskService {
    pub store: JsonFileStore,
}

impl TaskService {
    pub fn new() -> Result<Self> {
        let store = JsonFileStore::new()?;
        Ok(TaskService { store })
    }

    #[cfg(test)]
    pub fn with_store(store: JsonFileStore) -> Result<Self> {
        Ok(TaskService { store })
    }
    pub fn validate_title(title: &str) -> Result<()> {
        if title.is_empty() {
            return Err(TaskError::EmptyTitle.into());
        }
        if title.len() > 100 {
            return Err(TaskError::TitleTooLong.into());
        }
        Ok(())
    }

    pub fn validate_due_date(due: Option<&str>) -> Result<Option<NaiveDate>> {
        match due {
            None => Ok(None),
            Some(s) => {
                let date =
                    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| TaskError::InvalidDate)?;
                Ok(Some(date))
            }
        }
    }
    pub fn add_task(
        &self,
        title: &str,
        desc: Option<&str>,
        priority: Option<Priority>,
        tags: Vec<&str>,
        due: Option<&str>,
    ) -> Result<Task> {
        Self::validate_title(title)?;
        let due_date = Self::validate_due_date(due)?;
        let id = Uuid::new_v4();
        let description = match desc {
            Some(d) => Some(d.to_string()),
            None => None,
        };
        let task = Task {
            id: id.to_string(),
            title: title.to_string(),
            description,
            status: Status::Todo,
            priority: priority.unwrap_or(Priority::Medium),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            due_date,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let mut tasks = self.store.load()?;
        tasks.push(task.clone());

        self.store.save(&tasks)?;

        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::tests;

    fn create_temp_services() -> Result<TaskService> {
        let store = tests::temp_store("service");
        TaskService::with_store(store)
    }
    #[test]
    fn test_add_task() {
        let services = create_temp_services().unwrap();
        let t = services
            .add_task("测试", Some("测试desc"), None, vec!["1"], None)
            .unwrap();
        assert_eq!(t.title, "测试");
        assert!(!t.id.is_empty());
        let tasks = services.store.load().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, t.id);
        tests::cleanup(&services.store);
    }
}
