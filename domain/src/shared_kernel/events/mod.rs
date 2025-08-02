mod domain_event;
mod event_publisher;

pub use event_publisher::*;

pub use domain_event::*;
/// # EventSourced Trait
///
/// This trait defines the contract for domain aggregates that generate and track
/// domain events. It follows the Event Sourcing pattern where domain objects
/// collect events internally rather than publishing them directly.
///
/// ## Key Principles
///
/// 1. **Event Collection**: Domain objects collect events without publishing them
/// 2. **Dependency Inversion**: No dependency on infrastructure concerns
/// 3. **Application Orchestration**: Application services handle event publishing
///
/// ## Usage Pattern
///
/// ```rust,ignore
/// // 1. Domain operation generates events
/// let mut task = repository.get(id)?;
/// task.complete_session()?; // Collects events internally
///
/// // 2. Application service publishes events
/// let events = task.get_uncommitted_events();
/// for event in events {
///     event_publisher.publish(event);
/// }
///
/// // 3. Mark events as published
/// task.mark_events_as_committed();
/// repository.save(task)?;
/// ```
pub trait EventSourced {
    /// Applies an event to the aggregate's state.
    ///
    /// This is used for event replay and state reconstruction.
    /// The event should update the aggregate's internal state accordingly.
    fn apply_event(&mut self, event: &dyn DomainEvent) -> crate::Result<()>;

    /// Returns all events that have been generated but not yet published.
    ///
    /// Application services use this to retrieve events for publishing
    /// after successful domain operations.
    fn get_uncommitted_events(&self) -> Vec<Box<dyn DomainEvent>>;

    /// Marks all uncommitted events as committed (published).
    ///
    /// This should be called after successful event publishing to prevent
    /// duplicate event publication.
    fn mark_events_as_committed(&mut self);

    /// Returns the current version of the aggregate.
    ///
    /// Used for optimistic concurrency control and event ordering.
    fn get_version(&self) -> u64;

    /// Increments the aggregate version.
    ///
    /// Should be called when applying new events to maintain version consistency.
    fn increment_version(&mut self);

    /// Adds an event to the uncommitted events collection.
    ///
    /// Domain operations should use this to record events that occurred
    /// during business logic execution.
    fn add_event(&mut self, event: Box<dyn DomainEvent>);
}

