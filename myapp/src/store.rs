use crate::error::TaskError;
use crate::models::Task;
use anyhow::{Ok, Result};
use serde_json::{from_str, to_string_pretty};
use std::fs::{create_dir_all, read_to_string};
use std::path::PathBuf;

pub trait Store {
    fn load(&self) -> Result<Vec<Task>>;
    fn save(&self, tasks: &[Task]) -> Result<()>;
}

pub struct JsonFileStore {
    pub(crate) file_path: PathBuf,
}

impl JsonFileStore {
    pub fn new() -> Result<JsonFileStore> {
        if let Some(home_dir) = dirs::home_dir() {
            let task_dir = home_dir.join(".taskflow");
            create_dir_all(&task_dir)?;
            let data_path = task_dir.join("data.json");
            Ok(JsonFileStore {
                file_path: data_path,
            })
        } else {
            return Err(TaskError::HomeDirNotFound.into());
        }
    }

    #[cfg(test)]
    pub(crate) fn with_path(path: PathBuf) -> Self {
        JsonFileStore { file_path: path }
    }
}

impl Store for JsonFileStore {
    fn load(&self) -> Result<Vec<Task>> {
        let path = &self.file_path;
        if path.exists() {
            let str = read_to_string(&path)?;
            let b: Vec<Task> = from_str(&str)?;
            Ok(b)
        } else {
            Ok(vec![])
        }
    }
    fn save(&self, tasks: &[Task]) -> Result<()> {
        let json = to_string_pretty(tasks)?;
        let back_path = self
            .file_path
            .parent()
            .ok_or(TaskError::HomeDirNotFound)?
            .join("data.json.bak");
        if self.file_path.exists() {
            std::fs::copy(&self.file_path, &back_path)?;
        }

        std::fs::write(&self.file_path, &json).inspect_err(|_| {
            let _ = std::fs::copy(&back_path, &self.file_path);
        })?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::models::{Priority, Status, Task};
    use crate::store::{JsonFileStore, Store};
    use chrono::{TimeZone, Utc};
    use std::fs;

    pub fn mock_task(id: &str) -> Task {
        Task {
            id: id.to_string(),
            title: format!("任务{}", id),
            description: None,
            status: Status::Todo,
            priority: Priority::High,
            due_date: None,
            tags: vec![],
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 1, 1, 1).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 1, 1, 1).unwrap(),
        }
    }

    pub fn temp_store(test_name: &str) -> JsonFileStore {
        let path = std::env::temp_dir();
        let data_dir = path.join("taskflow_test").join(test_name);
        // 先清理上次残留的目录
        let _ = fs::remove_dir_all(&data_dir);
        fs::create_dir_all(&data_dir).unwrap();
        let file_path = data_dir.join("test_data.json");
        JsonFileStore::with_path(file_path)
    }

    pub fn cleanup(store: &JsonFileStore) {
        if let Some(dir) = store.file_path.parent() {
            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn testJsonFileStore() {
        let a = JsonFileStore::new().unwrap();
        let b = a.file_path.to_str().unwrap();

        let home_dir = dirs::home_dir().unwrap();
        let data_path = home_dir.join(".taskflow").join("data.json");
        let c = data_path.to_str().unwrap();
        assert_eq!(b, c);
    }

    #[test]
    fn test_load_file_no_exists() {
        let store = temp_store("load_file_no_exists");
        assert!(!store.file_path.exists());
        assert!(store.load().unwrap().is_empty());
        cleanup(&store);
    }

    #[test]
    fn test_load_valid_json() {
        let store = temp_store("load_valid_json");

        let tasks = [mock_task("1"), mock_task("2")];
        store.save(&tasks).unwrap();

        let loaded = store.load().unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "1");
        assert_eq!(loaded[1].id, "2");
        cleanup(&store);
    }

    #[test]
    fn test_load_invalid_json() {
        let store = temp_store("load_invalid_json");
        fs::write(&store.file_path, "这不是合法的json").unwrap();
        let result = store.load();
        assert!(result.is_err());
        cleanup(&store);
    }

    #[test]
    fn test_save_and_read_back() {
        let store = temp_store("save_and_read_back");
        let tasks = [mock_task("a"), mock_task("b"), mock_task("c")];
        store.save(&tasks).unwrap();

        let loaded = store.load().unwrap();

        assert_eq!(loaded.len(), 3);
        cleanup(&store);
    }
}
