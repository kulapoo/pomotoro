use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use super::types::{Task, TaskId, TaskStatus};

pub type TaskRepository = Arc<dyn TaskRepositoryTrait + Send + Sync>;

#[async_trait::async_trait]
pub trait TaskRepositoryTrait {
    async fn create(&self, task: Task) -> Result<(), TaskError>;
    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>, TaskError>;
    async fn get_all(&self) -> Result<Vec<Task>, TaskError>;
    async fn get_active_tasks(&self) -> Result<Vec<Task>, TaskError>;
    async fn update(&self, task: Task) -> Result<(), TaskError>;
    async fn delete(&self, id: TaskId) -> Result<bool, TaskError>;
    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>, TaskError>;
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Task not found")]
    NotFound,
    #[error("Task already exists")]
    AlreadyExists,
    #[error("Invalid task data: {0}")]
    #[allow(dead_code)]
    InvalidData(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

pub struct InMemoryTaskRepository {
    tasks: RwLock<HashMap<TaskId, Task>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        let mut tasks = HashMap::new();
        
        let default_task = Task::new_default();
        tasks.insert(default_task.id, default_task);
        
        Self {
            tasks: RwLock::new(tasks),
        }
    }

    pub fn with_default_task() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TaskRepositoryTrait for InMemoryTaskRepository {
    async fn create(&self, task: Task) -> Result<(), TaskError> {
        let mut tasks = self.tasks.write().map_err(|e| TaskError::Storage(e.to_string()))?;
        
        if tasks.contains_key(&task.id) {
            return Err(TaskError::AlreadyExists);
        }
        
        tasks.insert(task.id, task);
        Ok(())
    }

    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>, TaskError> {
        let tasks = self.tasks.read().map_err(|e| TaskError::Storage(e.to_string()))?;
        Ok(tasks.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Task>, TaskError> {
        let tasks = self.tasks.read().map_err(|e| TaskError::Storage(e.to_string()))?;
        let mut task_list: Vec<Task> = tasks.values().cloned().collect();
        task_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(task_list)
    }

    async fn get_active_tasks(&self) -> Result<Vec<Task>, TaskError> {
        let all_tasks = self.get_all().await?;
        Ok(all_tasks
            .into_iter()
            .filter(|task| matches!(task.status, TaskStatus::Active | TaskStatus::Queued))
            .collect())
    }

    async fn update(&self, task: Task) -> Result<(), TaskError> {
        let mut tasks = self.tasks.write().map_err(|e| TaskError::Storage(e.to_string()))?;
        
        if !tasks.contains_key(&task.id) {
            return Err(TaskError::NotFound);
        }
        
        tasks.insert(task.id, task);
        Ok(())
    }

    async fn delete(&self, id: TaskId) -> Result<bool, TaskError> {
        let mut tasks = self.tasks.write().map_err(|e| TaskError::Storage(e.to_string()))?;
        Ok(tasks.remove(&id).is_some())
    }

    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>, TaskError> {
        let all_tasks = self.get_all().await?;
        Ok(all_tasks
            .into_iter()
            .filter(|task| {
                tags.iter().any(|tag| task.tags.contains(tag))
            })
            .collect())
    }
}