use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use pomotoro_domain::{Task, TaskId, TaskStatus, TaskRepository, Result, Error};
use crate::infrastructure::persistence::TaskDto;
use async_trait::async_trait;
use serde_json;

/// File-based task repository that demonstrates proper DTO usage
/// This shows how infrastructure layer should handle serialization
/// while keeping domain objects pure
pub struct FileTaskRepository {
    tasks_file: PathBuf,
}

impl FileTaskRepository {
    pub fn new(tasks_file: PathBuf) -> Self {
        Self { tasks_file }
    }

    /// Load tasks from file using DTOs for deserialization
    fn load_tasks(&self) -> Result<HashMap<TaskId, Task>> {
        if !self.tasks_file.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.tasks_file)
            .map_err(|e| Error::RepositoryError { 
                message: format!("Failed to read tasks file: {}", e) 
            })?;

        let task_dtos: Vec<TaskDto> = serde_json::from_str(&content)
            .map_err(|e| Error::RepositoryError { 
                message: format!("Failed to deserialize tasks: {}", e) 
            })?;

        let mut tasks = HashMap::new();
        for dto in task_dtos {
            let task = Task::try_from(dto)
                .map_err(|e| Error::RepositoryError { 
                    message: format!("Failed to convert DTO to Task: {:?}", e) 
                })?;
            tasks.insert(task.id, task);
        }

        Ok(tasks)
    }

    /// Save tasks to file using DTOs for serialization
    fn save_tasks(&self, tasks: &HashMap<TaskId, Task>) -> Result<()> {
        let task_dtos: Vec<TaskDto> = tasks
            .values()
            .map(|task| TaskDto::from(task.clone()))
            .collect();

        let content = serde_json::to_string_pretty(&task_dtos)
            .map_err(|e| Error::RepositoryError { 
                message: format!("Failed to serialize tasks: {}", e) 
            })?;

        let mut file = fs::File::create(&self.tasks_file)
            .map_err(|e| Error::RepositoryError { 
                message: format!("Failed to create tasks file: {}", e) 
            })?;

        file.write_all(content.as_bytes())
            .map_err(|e| Error::RepositoryError { 
                message: format!("Failed to write tasks file: {}", e) 
            })?;

        file.sync_all()
            .map_err(|e| Error::RepositoryError { 
                message: format!("Failed to sync tasks file: {}", e) 
            })?;

        Ok(())
    }
}

#[async_trait]
impl TaskRepository for FileTaskRepository {
    async fn create(&self, task: Task) -> Result<()> {
        let mut tasks = self.load_tasks()?;
        
        if tasks.contains_key(&task.id) {
            return Err(Error::TaskNotFound { 
                id: task.id.to_string() 
            });
        }
        
        tasks.insert(task.id, task);
        self.save_tasks(&tasks)?;
        Ok(())
    }

    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        let mut task_list: Vec<Task> = tasks.values().cloned().collect();
        task_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(task_list)
    }

    async fn get_active_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks
            .values()
            .filter(|task| matches!(task.status, TaskStatus::Active | TaskStatus::Queued))
            .cloned()
            .collect())
    }

    async fn update(&self, task: Task) -> Result<()> {
        let mut tasks = self.load_tasks()?;
        
        if !tasks.contains_key(&task.id) {
            return Err(Error::TaskNotFound { 
                id: task.id.to_string() 
            });
        }
        
        tasks.insert(task.id, task);
        self.save_tasks(&tasks)?;
        Ok(())
    }

    async fn delete(&self, id: TaskId) -> Result<bool> {
        let mut tasks = self.load_tasks()?;
        let removed = tasks.remove(&id).is_some();
        if removed {
            self.save_tasks(&tasks)?;
        }
        Ok(removed)
    }

    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks
            .values()
            .filter(|task| tags.iter().any(|tag| task.tags.contains(tag)))
            .cloned()
            .collect())
    }

    async fn get_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        let tasks = self.load_tasks()?;
        Ok(tasks
            .values()
            .filter(|task| task.status == status)
            .cloned()
            .collect())
    }

    async fn exists(&self, id: TaskId) -> Result<bool> {
        let tasks = self.load_tasks()?;
        Ok(tasks.contains_key(&id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::TaskDefaults;
    use tempfile::tempdir;

    #[tokio::test]
    async fn should_save_and_load_tasks_with_dtos() {
        let temp_dir = tempdir().unwrap();
        let tasks_file = temp_dir.path().join("tasks.json");
        let repo = FileTaskRepository::new(tasks_file);

        let defaults = TaskDefaults::default();
        let task = Task::new("Test Task".to_string(), 4, &defaults).unwrap();
        let task_id = task.id;

        // Create and save task
        repo.create(task.clone()).await.unwrap();

        // Load and verify task
        let loaded_task = repo.get_by_id(task_id).await.unwrap().unwrap();
        assert_eq!(loaded_task.name, "Test Task");
        assert_eq!(loaded_task.max_sessions, 4);

        // Verify file contains DTO format, not domain objects
        let file_content = fs::read_to_string(&repo.tasks_file).unwrap();
        assert!(file_content.contains("\"id\":"));
        assert!(file_content.contains("\"name\":"));
        // Should contain serialized TaskConfigDto, not TaskConfig methods
        assert!(file_content.contains("work_duration"));
    }

    #[tokio::test] 
    async fn should_handle_dto_conversion_errors() {
        let temp_dir = tempdir().unwrap();
        let tasks_file = temp_dir.path().join("invalid_tasks.json");
        
        // Write invalid DTO to file
        fs::write(&tasks_file, r#"[{"id": "invalid-id", "name": ""}]"#).unwrap();
        
        let repo = FileTaskRepository::new(tasks_file);
        let result = repo.get_all().await;
        
        // Should handle DTO conversion errors gracefully
        assert!(result.is_err());
    }
}