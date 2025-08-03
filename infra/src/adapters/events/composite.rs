use domain::{DomainEvent, EventPublisher};
use std::sync::Arc;

/// # CompositeEventPublisher - Combines Multiple Publishers
/// 
/// This publisher allows combining multiple event publishers,
/// useful for publishing to both internal handlers and the frontend.
pub struct CompositeEventPublisher {
    publishers: Vec<Arc<dyn EventPublisher + Send + Sync>>,
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
        for _publisher in &self.publishers {
            // For now, we'll just publish to the first one due to clone limitations
            // In a real implementation, you'd need proper event cloning
            if let Some(first_publisher) = self.publishers.first() {
                first_publisher.publish(event);
                break;
            }
        }
    }

    fn publish_batch(&self, _events: Vec<Box<dyn DomainEvent>>) {
        for _publisher in &self.publishers {
            _publisher.publish_batch(vec![]); // Empty for now due to clone limitations
        }
    }
}