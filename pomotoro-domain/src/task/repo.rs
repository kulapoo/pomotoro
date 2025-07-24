use async_trait::async_trait;
use crate::{Task, TaskId, TaskStatus, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn create(&self, task: Task) -> Result<()>;
    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>>;
    async fn get_all(&self) -> Result<Vec<Task>>;
    async fn get_active_tasks(&self) -> Result<Vec<Task>>;
    async fn update(&self, task: Task) -> Result<()>;
    async fn delete(&self, id: TaskId) -> Result<bool>;
    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>>;
    async fn get_by_status(&self, status: TaskStatus) -> Result<Vec<Task>>;
    async fn exists(&self, id: TaskId) -> Result<bool>;
}

// Test implementation for TaskRepository - available for use in tests
#[derive(Default)]
pub struct InMemoryTaskRepository {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tasks(tasks: Vec<Task>) -> Self {
        let mut task_map = HashMap::new();
        for task in tasks {
            let task_id = task.id.clone();
            task_map.insert(task_id, task);
        }
        Self {
            tasks: Arc::new(RwLock::new(task_map)),
        }
    }
}

#[async_trait]
impl TaskRepository for InMemoryTaskRepository {
    async fn create(&self, task: Task) -> Result<()> {
        let mut tasks = self.tasks.write().unwrap();
        if tasks.contains_key(&task.id) {
            return Err(crate::Error::TaskNotFound { 
                id: task.id.to_string() 
            });
        }
        let task_id = task.id.clone();
        tasks.insert(task_id, task);
        Ok(())
    }

    async fn get_by_id(&self, id: TaskId) -> Result<Option<Task>> {
        let tasks = self.tasks.read().unwrap();
        Ok(tasks.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().unwrap();
        Ok(tasks.values().cloned().collect())
    }

    async fn get_active_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().unwrap();
        Ok(tasks
            .values()
            .filter(|task| matches!(task.status, TaskStatus::Active | TaskStatus::Queued))
            .cloned()
            .collect())
    }

    async fn update(&self, task: Task) -> Result<()> {
        let mut tasks = self.tasks.write().unwrap();
        if !tasks.contains_key(&task.id) {
            return Err(crate::Error::TaskNotFound { 
                id: task.id.to_string() 
            });
        }
        let task_id = task.id.clone();
        tasks.insert(task_id, task);
        Ok(())
    }

    async fn delete(&self, id: TaskId) -> Result<bool> {
        let mut tasks = self.tasks.write().unwrap();
        Ok(tasks.remove(&id).is_some())
    }

    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().unwrap();
        Ok(tasks
            .values()
            .filter(|task| tags.iter().any(|tag| task.tags.contains(tag)))
            .cloned()
            .collect())
    }

    async fn get_by_status(&self, status: TaskStatus) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().unwrap();
        Ok(tasks
            .values()
            .filter(|task| task.status == status)
            .cloned()
            .collect())
    }

    async fn exists(&self, id: TaskId) -> Result<bool> {
        let tasks = self.tasks.read().unwrap();
        Ok(tasks.contains_key(&id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_create_and_retrieve_task() {
        let repo = InMemoryTaskRepository::new();
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        let task_id = task.id.clone();

        repo.create(task.clone()).await.unwrap();
        let retrieved = repo.get_by_id(task_id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Task");
    }

    #[tokio::test]
    async fn should_update_existing_task() {
        let repo = InMemoryTaskRepository::new();
        let mut task = Task::new("Original".to_string(), 4).unwrap();
        let task_id = task.id.clone();

        repo.create(task.clone()).await.unwrap();
        
        task.name = "Updated".to_string();
        repo.update(task).await.unwrap();

        let retrieved = repo.get_by_id(task_id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated");
    }

    #[tokio::test]
    async fn should_filter_tasks_by_status() {
        let active_task = Task::new("Active".to_string(), 4).unwrap();
        let mut completed_task = Task::new("Completed".to_string(), 1).unwrap();
        completed_task.increment_session().unwrap(); // Makes it completed

        let repo = InMemoryTaskRepository::with_tasks(vec![
            active_task.clone(),
            completed_task,
        ]);

        let active_tasks = repo.get_active_tasks().await.unwrap();
        assert_eq!(active_tasks.len(), 1);
        assert_eq!(active_tasks[0].name, "Active");
    }

    #[tokio::test]
    async fn should_filter_tasks_by_tags() {
        let work_task = Task::new("Work Task".to_string(), 4)
            .unwrap()
            .with_tags(vec!["work".to_string(), "urgent".to_string()]);
        
        let personal_task = Task::new("Personal Task".to_string(), 2)
            .unwrap()
            .with_tags(vec!["personal".to_string()]);

        let repo = InMemoryTaskRepository::with_tasks(vec![
            work_task,
            personal_task,
        ]);

        let work_tasks = repo.get_by_tags(&["work".to_string()]).await.unwrap();
        assert_eq!(work_tasks.len(), 1);
        assert_eq!(work_tasks[0].name, "Work Task");
    }
}