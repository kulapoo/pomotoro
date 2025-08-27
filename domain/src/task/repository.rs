use super::{Task, id::Id, status::Status};
use crate::{Result, shared_kernel::traits::searchable::SearchCriteria};
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub enum SortBy {
    Name,
    CreatedAt,
    SessionsCompleted,
    Status,
}

#[derive(Clone, Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Clone, Debug)]
pub struct SearchOptions {
    pub criteria: SearchCriteria,
    pub sort_by: Option<SortBy>,
    pub sort_order: Option<SortOrder>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            criteria: SearchCriteria::default(),
            sort_by: None,
            sort_order: None,
        }
    }
}

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
    async fn search(&self, options: SearchOptions) -> Result<Vec<Task>>;
    async fn search_fuzzy(&self, query: &str) -> Result<Vec<Task>>;
    async fn get_incomplete_tasks(&self) -> Result<Vec<Task>>;
    async fn get_completed_tasks(&self) -> Result<Vec<Task>>;
}
