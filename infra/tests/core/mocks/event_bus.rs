use std::sync::{Arc, Mutex};
use domain::shared_kernel::events::{Event, EventPublisher};

/// Mock event bus for testing
pub struct MockEventBus {
    events: Arc<Mutex<Vec<Box<dyn Event>>>>,
    publish_count: Arc<Mutex<usize>>,
}

impl MockEventBus {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            publish_count: Arc::new(Mutex::new(0)),
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