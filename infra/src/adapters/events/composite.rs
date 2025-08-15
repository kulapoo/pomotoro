use domain::{DomainEvent, EventPublisher};
use std::sync::Arc;

/// # CompositeEventPublisher - Combines Multiple Publishers
///
/// This publisher allows combining multiple event publishers,
/// useful for publishing to both internal handlers and the frontend.
pub struct CompositeEventPublisher {
    publishers: Vec<Arc<dyn EventPublisher + Send + Sync>>,
}

impl Default for CompositeEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeEventPublisher {
    pub fn new() -> Self {
        Self {
            publishers: Vec::new(),
        }
    }

    pub fn add_publisher(&mut self, publisher: Arc<dyn EventPublisher + Send + Sync>) {
        self.publishers.push(publisher);
    }
}

impl EventPublisher for CompositeEventPublisher {
    fn publish(&self, event: Box<dyn DomainEvent>) {
        let publisher_count = self.publishers.len();

        if publisher_count == 0 {
            return;
        }

        for publisher in self.publishers.iter().take(publisher_count - 1) {
            let cloned_event = event.clone_box();
            publisher.publish(cloned_event);
        }

        if let Some(last_publisher) = self.publishers.last() {
            last_publisher.publish(event);
        }
    }

    fn publish_batch(&self, events: Vec<Box<dyn DomainEvent>>) {
        let publisher_count = self.publishers.len();

        if publisher_count == 0 {
            return;
        }

        for publisher in self.publishers.iter().take(publisher_count - 1) {
            let cloned_events: Vec<Box<dyn DomainEvent>> = events
                .iter()
                .map(|event| event.clone_box())
                .collect();
            publisher.publish_batch(cloned_events);
        }

        if let Some(last_publisher) = self.publishers.last() {
            last_publisher.publish_batch(events);
        }
    }
}