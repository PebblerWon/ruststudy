use crate::{
    error::TaskError,
    models::{Priority, Status, Task, TaskCsvRow, TaskStats},
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
        if title.chars().count() > 100 {
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

    pub fn validate_tags(tags: &[&str]) -> Result<()> {
        let len = tags.len();
        if len > 10 {
            return Err(TaskError::TooManyTags.into());
        }
        Ok(())
    }

    pub fn find_task_by_id(tasks: &Vec<Task>, id: &str) -> Result<usize> {
        let mut found: Option<usize> = None;
        for (i, v) in tasks.iter().enumerate() {
            if v.id.starts_with(id) {
                if found.is_some() {
                    return Err(TaskError::AmbiguousId(id.to_string()).into());
                } else {
                    found = Some(i);
                }
            }
        }
        match found {
            Some(f) => Ok(f),
            None => Err(TaskError::NotFound(id.to_string()).into()),
        }
    }
    pub fn get_task_by_id(&self, id: &str) -> Result<Task> {
        let tasks = self.store.load()?;
        let i = Self::find_task_by_id(&tasks, id)?;
        Ok(tasks[i].clone())
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
        Self::validate_tags(&tags)?;
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
        tags: Vec<&str>,
        due: Option<&str>,
    ) -> Result<Task> {
        let mut tasks = self.store.load()?;
        let update_index = Self::find_task_by_id(&tasks, id)?;

        let update_item = tasks
            .get_mut(update_index)
            .ok_or_else(|| TaskError::NotFound(id.to_string()))?;
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
        if !tags.is_empty() {
            Self::validate_tags(&tags)?;
            update_item.tags = tags.iter().map(|s| s.to_string()).collect();
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

    pub fn search_task(&self, keyword: &str) -> Result<Vec<Task>> {
        let tasks = self.store.load()?;
        let keyword_lower = keyword.to_lowercase();

        let res = tasks
            .into_iter()
            .filter(|i| {
                let desc_match = i
                    .description
                    .as_deref()
                    .map_or(false, |d| d.to_lowercase().contains(&keyword_lower));
                i.title.to_lowercase().contains(&keyword_lower) || desc_match
            })
            .collect();
        Ok(res)
    }

    pub fn get_stats(&self) -> Result<TaskStats> {
        let tasks = self.store.load()?;

        let total = tasks.len();
        let mut stats = TaskStats {
            total,
            ..Default::default()
        };

        let today = Some(Utc::now().date_naive());
        for task in &tasks {
            match task.status {
                Status::Done => stats.done += 1,
                Status::InProgress => stats.in_progress += 1,
                Status::Todo => stats.todo += 1,
            }
            match task.priority {
                Priority::High => stats.high += 1,
                Priority::Medium => stats.medium += 1,
                Priority::Low => stats.low += 1,
            }

            let overdue =
                task.due_date.is_some() && task.due_date < today && task.status != Status::Done;

            if overdue {
                stats.overdue += 1;
            }
        }

        let completion_rate = if total == 0 {
            0.0
        } else {
            stats.done as f64 / total as f64
        };
        stats.completion_rate = completion_rate;

        Ok(stats)
    }
    pub fn export_tasks(&self, format: &str) -> Result<String> {
        if format.to_lowercase() != "csv" {
            return Err(TaskError::UnsupportedFormat(format.to_string()).into());
        }
        let tasks = self.store.load()?;
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(vec![]);
        for i in tasks.iter() {
            wtr.serialize(TaskCsvRow::from(i))?;
        }
        let a = wtr.into_inner()?;
        let res = String::from_utf8(a)?;
        Ok(format!("\u{FEFF}{}", res))
    }
}

#[cfg(test)]
mod tests {
    use std::{assert_eq, vec};

    use super::*;
    use crate::store::tests;
    use chrono::{Duration, NaiveDate, TimeZone, Utc};

    fn create_temp_services(test_name: &str) -> Result<TaskService> {
        let store = tests::temp_store(test_name);
        TaskService::with_store(store)
    }
    fn mock_tasks() -> Vec<Task> {
        return vec![
            Task {
                id: "1".to_string(),
                title: format!("任务1"),
                description: Some("任务1".to_string()),
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
                vec!["2", "3"],
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

    #[test]
    fn test_search_task() {
        let services = create_temp_services("test_search_task").unwrap();

        let mock_data = mock_tasks();
        let _ = services.store.save(&mock_data);

        services
            .add_task(
                "务1",
                Some("任务1描述"),
                Some(Priority::Low),
                vec!["tag1"],
                None,
            )
            .unwrap();
        services
            .add_task("abc", Some("DEF"), Some(Priority::Low), vec!["tag1"], None)
            .unwrap();

        let search_title_res = services.search_task("务1").unwrap();
        assert_eq!(search_title_res.len(), 2);

        let search_desc_res = services.search_task("描述").unwrap();
        assert_eq!(search_desc_res.len(), 1);

        let search_none_res = services.search_task("无数据").unwrap();
        assert!(search_none_res.is_empty());

        let search_ignorecase_res = services.search_task("AB").unwrap();
        assert_eq!(search_ignorecase_res.len(), 1);

        let search_ignorecase_res = services.search_task("def").unwrap();
        assert_eq!(search_ignorecase_res.len(), 1);

        tests::cleanup(&services.store);
    }

    #[test]
    fn test_stats() {
        let services = create_temp_services("test_stats").unwrap();

        let stats1 = services.get_stats().unwrap();
        assert_eq!(stats1.completion_rate, 0.0);

        let today = Utc::now().date_naive();
        let todaystr = today.format("%Y-%m-%d").to_string();
        let yesterday = today - Duration::days(1);
        let yesterdaystr = yesterday.format("%Y-%m-%d").to_string();
        let tomorrow = today + Duration::days(1);
        let tomorrowstr = tomorrow.format("%Y-%m-%d").to_string();

        services
            .add_task("1", None, Some(Priority::High), vec![], None)
            .unwrap();
        services
            .add_task("2", None, Some(Priority::Medium), vec![], None)
            .unwrap();
        let three = services
            .add_task("3", None, Some(Priority::Low), vec![], None)
            .unwrap();
        let four = services
            .add_task("4", None, Some(Priority::Low), vec![], None)
            .unwrap();

        services
            .update_task(
                &three.id,
                None,
                Some(Status::InProgress),
                None,
                None,
                vec![],
                Some(&yesterdaystr),
            )
            .unwrap();
        services
            .update_task(&four.id, None, Some(Status::Done), None, None, vec![], None)
            .unwrap();

        let stats = services.get_stats().unwrap();
        let tasks = services.store.load().unwrap();

        assert_eq!(stats.total, tasks.len());
        assert_eq!(stats.todo, 2);
        assert_eq!(stats.in_progress, 1);
        assert_eq!(stats.done, 1);
        assert_eq!(stats.high, 1);
        assert_eq!(stats.medium, 1);
        assert_eq!(stats.low, 2);
        assert_eq!(stats.overdue, 1);
        assert_eq!(stats.completion_rate, 1.0 / 4.0);

        services
            .update_task(
                &three.id,
                None,
                Some(Status::Todo),
                None,
                None,
                vec![],
                Some(&todaystr),
            )
            .unwrap();

        let stats = services.get_stats().unwrap();
        assert_eq!(stats.overdue, 0);

        services
            .update_task(
                &three.id,
                None,
                Some(Status::Todo),
                None,
                None,
                vec![],
                None,
            )
            .unwrap();

        let stats = services.get_stats().unwrap();
        assert_eq!(stats.overdue, 0);

        services
            .update_task(
                &three.id,
                None,
                Some(Status::Done),
                None,
                None,
                vec![],
                Some(&yesterdaystr),
            )
            .unwrap();

        let stats = services.get_stats().unwrap();
        assert_eq!(stats.overdue, 0);

        services
            .update_task(
                &three.id,
                None,
                Some(Status::Todo),
                None,
                None,
                vec![],
                Some(&tomorrowstr),
            )
            .unwrap();

        let stats = services.get_stats().unwrap();
        assert_eq!(stats.overdue, 0);

        tests::cleanup(&services.store);
    }

    #[test]
    fn test_export() {
        let services = create_temp_services("test_export").unwrap();

        let a = services.export_tasks("csv").unwrap();

        assert_eq!(a.lines().count(), 1);
        assert!(a.starts_with("\u{FEFF}"));

        let b = services.export_tasks("json");
        assert!(b.is_err());

        let t = services
            .add_task(
                "title1",
                None,
                Some(Priority::High),
                vec!["tag1", "tag2", "tag3"],
                None,
            )
            .unwrap();
        let c = services.export_tasks("csv").unwrap();
        assert_eq!(c.lines().count(), 2);

        let mut lines = c.lines();
        let l1 = lines.next().unwrap();
        assert!(l1.starts_with("\u{FEFF}"));
        assert!(l1.contains("ID,标题,描述,状态,优先级,标签,截止日期,创建时间,更新时间"));
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(c.as_bytes());
        let row: TaskCsvRow = rdr.deserialize().next().unwrap().unwrap();

        assert_eq!(row.description, "");
        assert_eq!(row.tags, "tag1;tag2;tag3");
        assert_eq!(row.due_date, "");
        assert_eq!(row.created_at, t.created_at.to_rfc3339());
        assert_eq!(row.updated_at, t.updated_at.to_rfc3339());

        tests::cleanup(&services.store);
    }

    #[test]
    fn test_validate_title() {
        // 空标题
        assert!(TaskService::validate_title("").is_err());

        // 恰好 100 字符
        let s100: String = "a".repeat(100);
        assert!(TaskService::validate_title(&s100).is_ok());

        // 101 字符
        let s101: String = "a".repeat(101);
        assert!(TaskService::validate_title(&s101).is_err());

        // 34 个中文字符 = 102 字节，验证按字符数而非字节数
        let s_cn = "中".repeat(34);
        assert_eq!(s_cn.len(), 102);
        assert!(TaskService::validate_title(&s_cn).is_ok());
    }

    #[test]
    fn test_validate_due_date() {
        // None
        assert!(TaskService::validate_due_date(None).unwrap().is_none());

        // 合法日期
        let d = TaskService::validate_due_date(Some("2026-08-15")).unwrap();
        assert_eq!(d, Some(NaiveDate::from_ymd_opt(2026, 8, 15).unwrap()));

        // 非法格式
        assert!(TaskService::validate_due_date(Some("2026/08/15")).is_err());

        // 不存在的日期
        assert!(TaskService::validate_due_date(Some("2026-02-30")).is_err());

        // 纯乱码
        assert!(TaskService::validate_due_date(Some("invalid")).is_err());
    }

    #[test]
    fn test_validate_tags() {
        // 0 条
        assert!(TaskService::validate_tags(&[]).is_ok());

        // 恰好 10 条
        let ten: Vec<&str> = vec!["t"; 10];
        assert!(TaskService::validate_tags(&ten).is_ok());

        // 11 条
        let eleven: Vec<&str> = vec!["t"; 11];
        assert!(TaskService::validate_tags(&eleven).is_err());
    }

    #[test]
    fn test_add_task_too_many_tags() {
        let services = create_temp_services("service_add_too_many_tags").unwrap();
        let eleven: Vec<String> = vec!["t".to_string(); 11];
        let tags: Vec<&str> = eleven.iter().map(String::as_str).collect();
        let r = services.add_task("test", None, None, tags, None);
        assert!(r.is_err());
        tests::cleanup(&services.store);
    }

    #[test]
    fn test_update_task_too_many_tags() {
        let services = create_temp_services("service_update_too_many_tags").unwrap();
        let mock_data = mock_tasks();
        let _ = services.store.save(&mock_data);

        let update_id = &mock_data[0].id;
        let eleven: Vec<String> = vec!["t".to_string(); 11];
        let tags: Vec<&str> = eleven.iter().map(String::as_str).collect();
        let r = services.update_task(update_id, None, None, None, None, tags, None);
        assert!(r.is_err());
        tests::cleanup(&services.store);
    }
}
