use super::{Task, id::Id, status::Status};
use crate::Result;
use async_trait::async_trait;

#[derive(Default, Clone, Debug)]
pub struct SearchCriteria {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl SearchCriteria {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn with_status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

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

#[derive(Clone, Debug, Default)]
pub struct SearchOptions {
    pub criteria: SearchCriteria,
    pub sort_by: Option<SortBy>,
    pub sort_order: Option<SortOrder>,
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
    async fn search(&self, options: SearchOptions) -> Result<Vec<Task>>;
    async fn search_fuzzy(&self, query: &str) -> Result<Vec<Task>>;
    async fn get_incomplete_tasks(&self) -> Result<Vec<Task>>;
}
