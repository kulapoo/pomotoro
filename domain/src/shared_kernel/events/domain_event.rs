use chrono::{DateTime, Utc};
use std::any::Any;

/// # Implementation Notes
///
/// This trait belongs in the shared kernel as it's a cross-cutting concern used
/// by all bounded contexts. All domain events in the system should implement this trait.
pub trait DomainEvent: Send + Sync + std::fmt::Debug + Any {
    /// Unique identifier for the type of event.
    ///
    /// This should be a stable string that uniquely identifies the event type.
    /// It's used for event routing, serialization, and versioning.
    ///
    /// # Returns
    ///
    /// A static string representing the event type (e.g., "NoteCreated", "UserRegistered")
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn event_type(&self) -> &'static str {
    ///     "NoteCreated"  // Stable identifier for this event type
    /// }
    /// ```
    fn event_type(&self) -> &'static str;

    /// ID of the aggregate that generated this event.
    ///
    /// This identifies which specific aggregate instance raised the event.
    /// Used for event sourcing, debugging, and routing events back to aggregates.
    ///
    /// # Returns
    ///
    /// String representation of the aggregate's unique identifier
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn aggregate_id(&self) -> String {
    ///     self.task.to_string()  // ID of the task that was created
    /// }
    /// ```
    fn aggregate_id(&self) -> String;

    /// Version of the aggregate when this event occurred.
    ///
    /// Used for optimistic concurrency control and ensuring events are processed
    /// in the correct order. Each event increments the aggregate version.
    ///
    /// # Returns
    ///
    /// The aggregate version number at the time the event occurred
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn version(&self) -> u64 {
    ///     self.version  // Version 1 for creation, 2 for first update, etc.
    /// }
    /// ```
    fn version(&self) -> u64;

    /// When this event occurred in the domain.
    ///
    /// This represents the business time when the event happened, not necessarily
    /// when it was persisted or processed. Always in UTC for consistency.
    ///
    /// # Returns
    ///
    /// UTC timestamp of when the business event occurred
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn occurred_at(&self) -> DateTime<Utc> {
    ///     self.created_at  // When the note was actually created
    /// }
    /// ```
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Clone this domain event into a boxed trait object.
    ///
    /// This method enables cloning of domain events when they are in trait object form.
    /// It's required for scenarios where the same event needs to be published to multiple
    /// publishers or handlers.
    ///
    /// # Returns
    ///
    /// A new boxed instance of this event with identical data
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn clone_box(&self) -> Box<dyn DomainEvent> {
    ///     Box::new(self.clone())
    /// }
    /// ```
    fn clone_box(&self) -> Box<dyn DomainEvent>;

    /// Returns a reference to the underlying Any type for downcasting.
    ///
    /// This enables type-safe downcasting of domain events from trait objects
    /// back to their concrete types when needed by handlers.
    ///
    /// # Returns
    ///
    /// A reference to self as Any trait object
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if let Some(task_created) = event.as_any().downcast_ref::<TaskCreated>() {
    ///     // Handle specific event type
    /// }
    /// ```
    fn as_any(&self) -> &dyn Any;
}
