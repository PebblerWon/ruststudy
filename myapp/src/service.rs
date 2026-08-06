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

    pub fn list_tasks(
        &self,
        status: Option<Status>,
        priority: Option<Priority>,
        tag: Option<&str>,
    ) -> Result<Vec<Task>> {
        let mut tasks = self.store.load()?;

        if let Some(s) = status {
            tasks = tasks.into_iter().filter(|t| t.status == s).collect();
        }
        if let Some(p) = priority {
            tasks = tasks.into_iter().filter(|t| t.priority == p).collect();
        }
        if let Some(arg_tag) = tag {
            tasks = tasks
                .into_iter()
                .filter(|t| {
                    (&t.tags)
                        .into_iter()
                        .any(|cur_tag| cur_tag.contains(arg_tag))
                })
                .collect();
        }
        Ok(tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::tests;
    use chrono::{NaiveDate, TimeZone, Utc};

    fn create_temp_services(test_name: &str) -> Result<TaskService> {
        let store = tests::temp_store(test_name);
        TaskService::with_store(store)
    }
    #[test]
    fn test_add_task() {
        let services = create_temp_services("service_add_task").unwrap();
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

    #[test]
    fn test_list_task() {
        let services = create_temp_services("service_list_task").unwrap();
        let tasks = [
            Task {
                id: "1".to_string(),
                title: format!("任务1"),
                description: None,
                status: Status::Todo,
                priority: Priority::Low,
                due_date: None,
                tags: vec!["1".to_string()],
                created_at: Utc.with_ymd_and_hms(2026, 1, 1, 1, 1, 1).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 1, 1, 1).unwrap(),
            },
            Task {
                id: "3".to_string(),
                title: format!("任务3"),
                description: None,
                status: Status::Done,
                priority: Priority::High,
                due_date: None,
                tags: vec!["2".to_string()],
                created_at: Utc.with_ymd_and_hms(2026, 1, 1, 1, 1, 1).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 1, 1, 1).unwrap(),
            },
            Task {
                id: "2".to_string(),
                title: format!("任务2"),
                description: None,
                status: Status::InProgress,
                priority: Priority::Medium,
                due_date: None,
                tags: vec!["3".to_string()],
                created_at: Utc.with_ymd_and_hms(2026, 1, 1, 1, 1, 1).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 1, 1, 1).unwrap(),
            },
        ];
        services.store.save(&tasks);

        let list1 = services.list_tasks(Some(Status::Done), None, None).unwrap();

        assert_eq!(list1.len(), 1);
        let list2 = services
            .list_tasks(None, Some(Priority::High), None)
            .unwrap();
        assert_eq!(list2.len(), 1);

        let list3 = services.list_tasks(None, None, Some("1")).unwrap();
        assert_eq!(list3.len(), 1);

        let list4 = services
            .list_tasks(Some(Status::InProgress), Some(Priority::Medium), Some("3"))
            .unwrap();
        assert_eq!(list4.len(), 1);

        tests::cleanup(&services.store);
    }
}
