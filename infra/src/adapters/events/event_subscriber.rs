use std::any::TypeId;

use super::EventHandler;
use domain::Result;

pub trait EventSubscriber: Send + Sync {
    fn subscribe(&self, handler: Box<dyn EventHandler>) -> Result<()>;

    fn clear_handlers_for_type(&self, event_type: TypeId) -> Result<()>;

    fn unsubscribe(&self, handler: Box<dyn EventHandler>) -> Result<()> {
        let event_type = handler.subscribes_to();
        let handler_name = handler.name();

        self.unsubscribe_by_name(event_type, handler_name)?;
        Ok(())
    }

    fn unsubscribe_by_name(&self, event_type: TypeId, handler_name: &str) -> Result<bool>;
}
