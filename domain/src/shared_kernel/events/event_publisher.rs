use crate::Event;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// # EventPublisher Trait
/// 
/// This trait belongs in the shared kernel as it defines how domain objects
/// communicate with the outside world without depending on infrastructure.
/// 
/// ## Clean Architecture Principle
/// 
/// - **Domain Layer**: Defines WHAT events occur (business semantics)
/// - **Application Layer**: Orchestrates WHEN events are published
/// - **Infrastructure Layer**: Implements WHERE events go (concrete mechanisms)
/// 
/// By depending only on this abstraction, domain services remain pure and testable.
pub trait EventPublisher: Send + Sync {
    /// Publishes a domain event to the event handling system.
    /// 
    /// This method should be called by application services after successful
    /// domain operations to notify other bounded contexts of important business events.
    /// 
    /// # Arguments
    /// 
    /// * `event` - The boxed domain event to publish
    /// 
    /// # Implementation Notes
    /// 
    /// Implementations should:
    /// - Handle event routing to appropriate handlers
    /// - Ensure reliable delivery (at least once)
    /// - Log events for debugging/audit purposes
    /// - Handle failures gracefully (dead letter queue, retry logic)
    fn publish(&self, event: Box<dyn Event>);
    
    /// Publishes multiple events as a batch operation.
    /// 
    /// This is useful for publishing all uncommitted events from an aggregate
    /// after a successful domain operation.
    /// 
    /// # Arguments
    /// 
    /// * `events` - Vector of boxed domain events to publish
    fn publish_batch(&self, events: Vec<Box<dyn Event>>);
}

/// # NoOpEventPublisher
/// 
/// A no-operation implementation for testing domain logic without event infrastructure.
/// This allows pure domain testing by eliminating external dependencies.
/// 
/// ## Usage in Tests
/// 
/// ```rust,ignore
/// let event_publisher = Arc::new(NoOpEventPublisher);
/// let service = TaskSessionService::new(repo, event_publisher);
/// // Test domain logic without event infrastructure
/// ```
#[derive(Debug, Default)]
pub struct NoOpEventPublisher;

impl EventPublisher for NoOpEventPublisher {
    fn publish(&self, _event: Box<dyn Event>) {
        // No-op: Events are ignored for testing domain logic
    }
    
    fn publish_batch(&self, _events: Vec<Box<dyn Event>>) {
        // No-op: Events are ignored for testing domain logic
    }
}

/// # MockEventPublisher
/// 
/// A mock implementation that captures published events for testing verification.
/// This allows tests to verify that the correct events were published with proper payloads.
/// 
/// ## Usage in Tests
/// 
/// ```rust,ignore
/// let mock_publisher = Arc::new(MockEventPublisher::new());
/// let service = TaskService::new(repo, mock_publisher.clone());
/// 
/// // Execute use case
/// service.complete_task(task_id).await;
/// 
/// // Verify events were published
/// let events = mock_publisher.published_events();
/// assert_eq!(events.len(), 1);
/// assert!(events[0].event_type() == "TaskCompleted");
/// ```
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct MockEventPublisher {
    events: Arc<Mutex<VecDeque<Box<dyn Event>>>>,
}

#[allow(dead_code)]
impl MockEventPublisher {
    /// Create a new MockEventPublisher
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    /// Get the count of published events
    pub fn event_count(&self) -> usize {
        let events = self.events.lock().unwrap();
        events.len()
    }
    
    /// Clear all published events (useful for test setup)
    pub fn clear_events(&self) {
        let mut events = self.events.lock().unwrap();
        events.clear();
    }
    
    /// Check if an event with the given type was published
    pub fn has_event_type(&self, event_type: &str) -> bool {
        let events = self.events.lock().unwrap();
        events.iter().any(|e| e.event_type() == event_type)
    }
    
    /// Get the event type of the most recently published event
    pub fn last_event_type(&self) -> Option<String> {
        let events = self.events.lock().unwrap();
        events.back().map(|e| e.event_type().to_string())
    }
    
    /// Get the event type of the first published event
    pub fn first_event_type(&self) -> Option<String> {
        let events = self.events.lock().unwrap();
        events.front().map(|e| e.event_type().to_string())
    }
    
    /// Get all event types that were published, in order
    pub fn event_types(&self) -> Vec<String> {
        let events = self.events.lock().unwrap();
        events.iter().map(|e| e.event_type().to_string()).collect()
    }
    
    /// Get count of events of a specific type
    pub fn count_events_of_type(&self, event_type: &str) -> usize {
        let events = self.events.lock().unwrap();
        events.iter().filter(|e| e.event_type() == event_type).count()
    }
    
    /// Verify events were published in expected order
    pub fn verify_event_sequence(&self, expected_types: &[&str]) -> bool {
        let actual_types = self.event_types();
        let expected_types: Vec<String> = expected_types.iter().map(|s| s.to_string()).collect();
        actual_types == expected_types
    }
}

impl EventPublisher for MockEventPublisher {
    fn publish(&self, event: Box<dyn Event>) {
        let mut events = self.events.lock().unwrap();
        events.push_back(event);
    }
    
    fn publish_batch(&self, events: Vec<Box<dyn Event>>) {
        let mut event_queue = self.events.lock().unwrap();
        for event in events {
            event_queue.push_back(event);
        }
    }
}

/// Implementation of EventPublisher for Arc<T> where T: EventPublisher
/// This allows Arc-wrapped publishers to be used directly as event publishers
impl<T: EventPublisher + ?Sized> EventPublisher for std::sync::Arc<T> {
    fn publish(&self, event: Box<dyn Event>) {
        (**self).publish(event)
    }
    
    fn publish_batch(&self, events: Vec<Box<dyn Event>>) {
        (**self).publish_batch(events)
    }
}