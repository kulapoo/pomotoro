use async_trait::async_trait;
use crate::Result;
use super::{Task, id::Id, status::Status};

#[async_trait]
pub trait Repository: Send + Sync {
    async fn create(&self, task: Task) -> Result<()>;
    async fn get_by_id(&self, id: Id) -> Result<Option<Task>>;
    async fn get_all(&self) -> Result<Vec<Task>>;
    async fn get_active_tasks(&self) -> Result<Vec<Task>>;
    async fn update(&self, task: Task) -> Result<()>;
    async fn delete(&self, id: Id) -> Result<bool>;
    async fn get_by_tags(&self, tags: &[String]) -> Result<Vec<Task>>;
    async fn get_by_status(&self, status: Status) -> Result<Vec<Task>>;
    async fn exists(&self, id: Id) -> Result<bool>;
    async fn get_default_task(&self) -> Result<Option<Task>>;
}