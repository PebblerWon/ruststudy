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

    pub fn find_task_by_id(tasks: &Vec<Task>, id: &str) -> Result<usize> {
        let s = tasks.iter().position(|t| t.id.starts_with(id));
        let mut p: Option<usize> = None;
        for (i, v) in tasks.iter().enumerate() {
            if v.id.starts_with(id) {
                if p.is_some() {
                    return Err(TaskError::AmbiguousId(id.to_string()).into());
                } else {
                    p = Some(i);
                }
            }
        }
        if s.is_none() {
            return Err(TaskError::NotFound(id.to_string()).into());
        } else {
            return Ok(p.unwrap());
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

    pub fn delete_task(&self, id: &str) -> Result<Task> {
        let mut tasks = self.store.load()?;
        let del_index = Self::find_task_by_id(&tasks, id)?;

        let deleted_task = tasks.remove(del_index);

        self.store.save(&tasks)?;
        Ok(deleted_task)
    }

    pub fn update_task(
        &self,
        id: &str,
        title: Option<&str>,
        status: Option<Status>,
        priority: Option<Priority>,
        desc: Option<&str>,
        tags: Option<Vec<String>>,
        due: Option<&str>,
    ) -> Result<Task> {
        let mut tasks = self.store.load()?;
        let update_index = Self::find_task_by_id(&tasks, id)?;

        let update_item = tasks.get_mut(update_index).unwrap();
        if let Some(t) = title {
            Self::validate_title(t)?;
            update_item.title = t.to_string();
        }
        if let Some(t) = status {
            update_item.status = t;
        }
        if let Some(p) = priority {
            update_item.priority = p;
        }
        if let Some(d) = desc {
            update_item.description = Some(d.to_string());
        }
        if let Some(t) = tags {
            update_item.tags = t;
        }
        if due.is_some() {
            let date = Self::validate_due_date(due)?;
            update_item.due_date = date;
        }

        update_item.updated_at = Utc::now();
        let updated_task = update_item.clone();
        self.store.save(&tasks)?;
        Ok(updated_task)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Display};

    use super::*;
    use crate::store::tests;
    use chrono::{NaiveDate, TimeZone, Utc};

    fn create_temp_services(test_name: &str) -> Result<TaskService> {
        let store = tests::temp_store(test_name);
        TaskService::with_store(store)
    }
    fn mock_tasks() -> Vec<Task> {
        return vec![
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
        let mock_data = mock_tasks();
        let _ = services.store.save(&mock_data);

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

    #[test]

    fn test_del_task() {
        let services = create_temp_services("service_del_task").unwrap();
        let mock_data = mock_tasks();
        let _ = services.store.save(&mock_data);

        let del_item = &mock_data[1];
        let deleted_item = services.delete_task(&del_item.id).unwrap();

        let new_tasks = services.store.load().unwrap();

        assert_eq!(new_tasks.len(), 2);
        assert_eq!(deleted_item.id, del_item.id);

        tests::cleanup(&services.store);
    }

    #[test]
    fn test_update_task() {
        let services = create_temp_services("service_update_task").unwrap();
        let mock_data = mock_tasks();
        let _ = services.store.save(&mock_data);

        let update_id = &mock_data.get(0).unwrap().id;

        services
            .update_task(
                update_id.as_str(),
                Some("1"),
                Some(Status::Done),
                Some(Priority::Low),
                Some("desc"),
                Some(vec!["2".to_string(), "3".to_string()]),
                Some("2026-12-31"),
            )
            .unwrap();
        let tasks = services.store.load().unwrap();
        let updated_index = TaskService::find_task_by_id(&tasks, update_id.as_str()).unwrap();
        let updated_item = &tasks[updated_index];

        assert_eq!(updated_item.title, "1");
        assert_eq!(updated_item.status, Status::Done);
        assert_eq!(updated_item.priority, Priority::Low);
        assert_eq!(updated_item.description, Some(String::from("desc")));
        assert_eq!(updated_item.tags, vec!["2".to_string(), "3".to_string()]);
        assert_eq!(
            updated_item.due_date.unwrap(),
            NaiveDate::parse_from_str("2026-12-31", "%Y-%m-%d").unwrap()
        );
        assert!(updated_item.updated_at > mock_data[0].updated_at);
        assert_eq!(updated_item.created_at, mock_data[0].created_at);

        tests::cleanup(&services.store);
    }
}
