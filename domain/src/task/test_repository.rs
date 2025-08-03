use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::{Task, TaskId, TaskStatus, TaskRepository, Result};

/// In-memory task repository for testing purposes
#[derive(Debug, Default)]
pub struct InMemoryTaskRepository {
    tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_default_task() -> Self {
        let repo = Self::new();
        // Could add default task here if needed
        repo
    }
}

#[async_trait]
impl TaskRepository for InMemoryTaskRepository {
    async fn create(&self, task: Task) -> Result<()> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task.id.clone(), task);
        Ok(())
    }

    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        let mut task_list: Vec<Task> = tasks.values().cloned().collect();
        task_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(task_list)
    }

    async fn get_active_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        let mut active_tasks: Vec<Task> = tasks
            .values()
            .filter(|task| task.status != TaskStatus::Completed)
            .cloned()
            .collect();
        // Sort by creation time for consistent ordering
        active_tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(active_tasks)
    }

    async fn update(&self, task: Task) -> Result<()> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task.id.clone(), task);
        Ok(())
    }

    async fn delete(&self, id: TaskId) -> Result<bool> {
        let mut tasks = self.tasks.lock().unwrap();
        Ok(tasks.remove(&id).is_some())
    }

    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks
            .values()
            .filter(|task| tags.iter().any(|tag| task.tags.contains(tag)))
            .cloned()
            .collect())
    }

    async fn get_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks
            .values()
            .filter(|task| task.status == status)
            .cloned()
            .collect())
    }

    async fn exists(&self, id: TaskId) -> Result<bool> {
        let tasks = self.tasks.lock().unwrap();
        Ok(tasks.contains_key(&id))
    }
}