mod domain_event;
mod task_events;
mod timer_events;

pub use domain_event::*;
pub use task_events::*;
pub use timer_events::*;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

pub type EventHandler = Arc<dyn Fn(&DomainEvent) + Send + Sync>;

#[derive(Clone)]
pub struct DomainEventBus {
    handlers: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
}

impl Default for DomainEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn subscribe<T: DomainEventData>(&self, handler: EventHandler) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers
            .entry(T::event_type().to_string())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub fn publish(&self, event: &DomainEvent) {
        let handlers = self.handlers.lock().unwrap();
        if let Some(event_handlers) = handlers.get(&event.event_type) {
            for handler in event_handlers {
                handler(event);
            }
        }
    }

    pub fn publish_typed<T: DomainEventData>(&self, aggregate_id: String, event_data: T, version: u64) {
        let event_json = serde_json::to_value(&event_data).unwrap();
        let event = DomainEvent::new(
            aggregate_id,
            T::event_type().to_string(),
            event_json,
            version,
        );
        self.publish(&event);
    }
}

pub trait EventSourced {
    fn apply_event(&mut self, event: &DomainEvent) -> crate::Result<()>;
    fn get_uncommitted_events(&self) -> Vec<DomainEvent>;
    fn mark_events_as_committed(&mut self);
    fn get_version(&self) -> u64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use crate::events::task_events::TaskCreated;
    use std::time::Duration;

    #[test]
    fn should_create_empty_event_bus() {
        let bus = DomainEventBus::new();
        let handlers = bus.handlers.lock().unwrap();
        assert!(handlers.is_empty());
    }

    #[test]
    fn should_subscribe_and_publish_events() {
        let bus = DomainEventBus::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let handler: EventHandler = Arc::new(move |_event| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.subscribe::<TaskCreated>(handler);

        let task_created = TaskCreated {
            task_id: Uuid::new_v4(),
            name: "Test Task".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
            config: crate::TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            },
            audio_config: crate::AudioConfig::default(),
        };

        bus.publish_typed("task-123".to_string(), task_created, 1);

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn should_handle_multiple_handlers_for_same_event() {
        let bus = DomainEventBus::new();
        let call_count1 = Arc::new(AtomicUsize::new(0));
        let call_count2 = Arc::new(AtomicUsize::new(0));
        
        let call_count1_clone = Arc::clone(&call_count1);
        let call_count2_clone = Arc::clone(&call_count2);

        let handler1: EventHandler = Arc::new(move |_event| {
            call_count1_clone.fetch_add(1, Ordering::SeqCst);
        });

        let handler2: EventHandler = Arc::new(move |_event| {
            call_count2_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.subscribe::<TaskCreated>(handler1);
        bus.subscribe::<TaskCreated>(handler2);

        let task_created = TaskCreated {
            task_id: Uuid::new_v4(),
            name: "Test Task".to_string(),
            description: None,
            max_sessions: 4,
            tags: vec![],
            config: crate::TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            },
            audio_config: crate::AudioConfig::default(),
        };

        bus.publish_typed("task-123".to_string(), task_created, 1);

        assert_eq!(call_count1.load(Ordering::SeqCst), 1);
        assert_eq!(call_count2.load(Ordering::SeqCst), 1);
    }
}