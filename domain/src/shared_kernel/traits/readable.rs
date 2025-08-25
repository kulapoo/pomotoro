use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Readable<T, ID> {
    async fn find_by_id(&self, id: &ID) -> Result<Option<T>>;
    async fn find_all(&self) -> Result<Vec<T>>;
    async fn exists(&self, id: &ID) -> Result<bool>;
    async fn count(&self) -> Result<usize>;
}

pub trait ReadableSync<T, ID> {
    fn find_by_id(&self, id: &ID) -> Result<Option<T>>;
    fn find_all(&self) -> Result<Vec<T>>;
    fn exists(&self, id: &ID) -> Result<bool>;
    fn count(&self) -> Result<usize>;
}
