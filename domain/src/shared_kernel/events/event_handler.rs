use async_trait::async_trait;
use std::any::TypeId;

use crate::{DomainEvent, Result};

#[async_trait]
pub trait EventHandler: Send + Sync {
    fn subscribes_to(&self) -> TypeId;
    async fn handle(&self, event: Box<dyn DomainEvent>) -> Result<()>;
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
