use domain::{DomainEvent, Result};
use domain::events::DomainEventHandler;
use std::any::TypeId;
use std::collections::HashMap;

pub struct HandlerRegistry {
    handlers: HashMap<TypeId, Vec<Box<dyn DomainEventHandler>>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, handler: Box<dyn DomainEventHandler>) {
        let event_type = handler.subscribes_to();
        self.handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub async fn handle_event(&self, event: Box<dyn DomainEvent>) -> Result<()> {
        let event_type_id = (*event).type_id();

        if let Some(handlers) = self.handlers.get(&event_type_id) {
            for handler in handlers {
                handler.handle(event.clone_box()).await?;
            }
        }

        Ok(())
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}