use std::any::TypeId;

use super::EventHandler;
use domain::Result;

pub trait EventSubscriber: Send + Sync {
    fn subscribe(&mut self, handler: Box<dyn EventHandler>) -> Result<()>;

    fn clear_handlers_for_type(&mut self, event_type: TypeId) -> Result<()>;

    fn unsubscribe(&mut self, handler: Box<dyn EventHandler>) -> Result<()>;

    fn unsubscribe_by_name(&mut self, event_type: TypeId, handler_name: &str) -> Result<bool>;
}