use domain::events::DomainEventHandler;
use std::any::TypeId;
use std::collections::HashMap;

pub struct HandlerRegistry {
    handlers: HashMap<TypeId, Vec<(String, Box<dyn DomainEventHandler>)>>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        HandlerRegistry {
            handlers: HashMap::new(),
        }
    }
    pub fn register(&mut self, handler: Box<dyn DomainEventHandler>) {
        let event_type = handler.subscribes_to();
        let handler_name = handler.name().to_string();

        self.handlers
            .entry(event_type)
            .or_default()
            .push((handler_name, handler));
    }

    pub fn unregister_by_name(&mut self, event_type: TypeId, handler_name: &str) -> bool {
        if let Some(handlers) = self.handlers.get_mut(&event_type) {
            if let Some(pos) = handlers.iter().position(|(name, _)| name == handler_name) {
                handlers.remove(pos);
                return true;
            }
        }
        false
    }

    pub fn clear_handlers_for_type(&mut self, event_type: TypeId) {
        self.handlers.remove(&event_type);
    }
 }


impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}