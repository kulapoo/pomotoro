use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Writable<T, ID> {
    async fn save(&mut self, entity: &T) -> Result<()>;
    async fn update(&mut self, id: &ID, entity: &T) -> Result<()>;
    async fn delete(&mut self, id: &ID) -> Result<bool>;
    async fn delete_all(&mut self) -> Result<usize>;
}

pub trait WritableSync<T, ID> {
    fn save(&mut self, entity: &T) -> Result<()>;
    fn update(&mut self, id: &ID, entity: &T) -> Result<()>;
    fn delete(&mut self, id: &ID) -> Result<bool>;
    fn delete_all(&mut self) -> Result<usize>;
}

pub trait Persistable {
    fn is_dirty(&self) -> bool;
    fn mark_clean(&mut self);
    fn mark_dirty(&mut self);
}

pub trait Versionable {
    fn get_version(&self) -> u64;
    fn increment_version(&mut self);
    fn set_version(&mut self, version: u64);
}
