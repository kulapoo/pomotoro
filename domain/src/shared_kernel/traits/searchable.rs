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

#[async_trait]
pub trait Searchable<T> {
    async fn search(&self, criteria: &SearchCriteria) -> Result<Vec<T>>;
    async fn search_by_tags(&self, tags: &[String]) -> Result<Vec<T>>;
    async fn search_by_query(&self, query: &str) -> Result<Vec<T>>;
}

pub trait SearchableSync<T> {
    fn search(&self, criteria: &SearchCriteria) -> Result<Vec<T>>;
    fn search_by_tags(&self, tags: &[String]) -> Result<Vec<T>>;
    fn search_by_query(&self, query: &str) -> Result<Vec<T>>;
}
