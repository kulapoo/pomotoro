use std::sync::{Arc, Mutex};
use std::any::TypeId;
use std::collections::HashMap;
use domain::shared_kernel::events::{Event, EventPublisher};
use infra::adapters::events::{EventHandler, EventSubscriber};
use domain::Result;

/// Mock event bus for testing
pub struct MockEventBus {
    events: Arc<Mutex<Vec<Box<dyn Event>>>>,
    publish_count: Arc<Mutex<usize>>,
    handlers: Arc<Mutex<HashMap<TypeId, Vec<Box<dyn EventHandler>>>>>,
}

impl MockEventBus {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            publish_count: Arc::new(Mutex::new(0)),
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn published_events(&self) -> Vec<String> {
        self.events.lock().unwrap()
            .iter()
            .map(|e| e.event_type().to_string())
            .collect()
    }

    pub fn published_count(&self) -> usize {
        *self.publish_count.lock().unwrap()
    }

    pub fn last_event(&self) -> Option<String> {
        self.events.lock().unwrap()
            .last()
            .map(|e| e.event_type().to_string())
    }

    pub fn events_of_type(&self, event_type: &str) -> Vec<String> {
        self.events.lock().unwrap()
            .iter()
            .filter(|e| e.event_type() == event_type)
            .map(|e| e.event_type().to_string())
            .collect()
    }

    pub fn has_event_type(&self, event_type: &str) -> bool {
        self.events.lock().unwrap()
            .iter()
            .any(|e| e.event_type() == event_type)
    }

    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
        *self.publish_count.lock().unwrap() = 0;
        self.handlers.lock().unwrap().clear();
    }

    pub fn assert_event_published(&self, event_type: &str) {
        assert!(
            self.has_event_type(event_type),
            "Expected event '{}' to be published, but it wasn't. Published events: {:?}",
            event_type,
            self.events.lock().unwrap()
                .iter()
                .map(|e| e.event_type().to_string())
                .collect::<Vec<_>>()
        );
    }

    pub fn assert_no_events(&self) {
        let count = self.published_count();
        assert_eq!(
            count, 0,
            "Expected no events to be published, but {} were published",
            count
        );
    }

    pub fn assert_event_count(&self, expected: usize) {
        let actual = self.published_count();
        assert_eq!(
            actual, expected,
            "Expected {} events to be published, but {} were published",
            expected, actual
        );
    }

    pub fn handler_count(&self) -> usize {
        self.handlers.lock().unwrap()
            .values()
            .map(|v| v.len())
            .sum()
    }

    pub fn handler_count_for_type(&self, event_type: TypeId) -> usize {
        self.handlers.lock().unwrap()
            .get(&event_type)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn has_handler_for_type(&self, event_type: TypeId) -> bool {
        self.handlers.lock().unwrap()
            .contains_key(&event_type)
    }
}

impl EventPublisher for MockEventBus {
    fn publish(&self, event: Box<dyn Event>) {
        self.events.lock().unwrap().push(event);
        *self.publish_count.lock().unwrap() += 1;
    }

    fn publish_batch(&self, events: Vec<Box<dyn Event>>) {
        let mut stored_events = self.events.lock().unwrap();
        let count = events.len();
        stored_events.extend(events);
        *self.publish_count.lock().unwrap() += count;
    }
}

impl EventSubscriber for MockEventBus {
    fn subscribe(&self, handler: Box<dyn EventHandler>) -> Result<()> {
        let event_type = handler.subscribes_to();
        let mut handlers = self.handlers.lock().unwrap();
        handlers.entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
        Ok(())
    }

    fn clear_handlers_for_type(&self, event_type: TypeId) -> Result<()> {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.remove(&event_type);
        Ok(())
    }

    fn unsubscribe_by_name(
        &self,
        event_type: TypeId,
        handler_name: &str,
    ) -> Result<bool> {
        let mut handlers = self.handlers.lock().unwrap();
        if let Some(type_handlers) = handlers.get_mut(&event_type) {
            let initial_len = type_handlers.len();
            type_handlers.retain(|h| h.name() != handler_name);
            Ok(initial_len != type_handlers.len())
        } else {
            Ok(false)
        }
    }
}

impl Default for MockEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MockEventBus {
    fn clone(&self) -> Self {
        Self {
            events: Arc::clone(&self.events),
            publish_count: Arc::clone(&self.publish_count),
            handlers: Arc::clone(&self.handlers),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::any::Any;

    #[derive(Debug, Clone)]
    struct TestEvent {
        event_type: String,
        aggregate_id: String,
        version: u64,
        occurred_at: chrono::DateTime<chrono::Utc>,
    }

    impl TestEvent {
        fn new(event_type: &str) -> Self {
            Self {
                event_type: event_type.to_string(),
                aggregate_id: "test_aggregate".to_string(),
                version: 1,
                occurred_at: Utc::now(),
            }
        }
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            Box::leak(self.event_type.clone().into_boxed_str())
        }

        fn aggregate_id(&self) -> String {
            self.aggregate_id.clone()
        }

        fn version(&self) -> u64 {
            self.version
        }

        fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
            self.occurred_at
        }

        fn clone_box(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn create_test_event(event_type: &str) -> Box<dyn Event> {
        Box::new(TestEvent::new(event_type))
    }

    #[test]
    fn tracks_published_events() {
        let bus = MockEventBus::new();
        
        bus.publish(create_test_event("task.created"));
        bus.publish(create_test_event("timer.started"));
        
        assert_eq!(bus.published_count(), 2);
        assert!(bus.has_event_type("task.created"));
        assert!(bus.has_event_type("timer.started"));
    }

    #[test]
    fn filters_events_by_type() {
        let bus = MockEventBus::new();
        
        bus.publish(create_test_event("task.created"));
        bus.publish(create_test_event("task.updated"));
        bus.publish(create_test_event("timer.started"));
        bus.publish(create_test_event("task.created"));
        
        let task_created = bus.events_of_type("task.created");
        assert_eq!(task_created.len(), 2);
        
        let timer_events = bus.events_of_type("timer.started");
        assert_eq!(timer_events.len(), 1);
    }

    #[test]
    fn clears_events() {
        let bus = MockEventBus::new();
        
        bus.publish(create_test_event("test"));
        assert_eq!(bus.published_count(), 1);
        
        bus.clear();
        assert_eq!(bus.published_count(), 0);
        assert!(bus.published_events().is_empty());
    }

    #[test]
    fn assertions_work() {
        let bus = MockEventBus::new();
        
        bus.assert_no_events();
        
        bus.publish(create_test_event("task.created"));
        bus.assert_event_published("task.created");
        bus.assert_event_count(1);
    }

    #[test]
    fn clone_shares_state() {
        let bus1 = MockEventBus::new();
        let bus2 = bus1.clone();
        
        bus1.publish(create_test_event("test"));
        assert_eq!(bus2.published_count(), 1);
        
        bus2.publish(create_test_event("test2"));
        assert_eq!(bus1.published_count(), 2);
    }
}