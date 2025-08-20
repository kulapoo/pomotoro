use async_trait::async_trait;
use crate::{Task, TaskId, TaskStatus, Result};

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
    async fn get_default_task(&self) -> Result<Option<Task>>;
}