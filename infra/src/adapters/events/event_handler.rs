use async_trait::async_trait;
use std::any::TypeId;

use domain::{Event, Result};

#[async_trait]
pub trait EventHandler: Send + Sync {
    fn subscribes_to(&self) -> TypeId;
    async fn handle(&self, event: Box<dyn Event>) -> Result<()>;
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}