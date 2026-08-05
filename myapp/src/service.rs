use anyhow::{Result,Ok};
use chrono::NaiveDate;

use crate::{error::TaskError, models::Priority, store::JsonFileStore};

pub struct TaskService {
    store: JsonFileStore,
}

impl TaskService {
    pub fn new() -> Result<Self> {
        let store = JsonFileStore::new()?;
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

    pub fn validate_due_date(due:Option<&str>)-> Result<Option<NaiveDate>>{
        match due {
            None => Ok(None),
            Some(s) => {
                let date = NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|_| TaskError::InvalidDate)?;
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

        Ok(Task {
            id: uuid::
        })
    }
}
