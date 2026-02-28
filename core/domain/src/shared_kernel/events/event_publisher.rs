use crate::Event;

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
