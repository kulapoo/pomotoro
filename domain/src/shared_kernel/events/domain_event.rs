use chrono::{DateTime, Utc};


/// # Implementation Notes
///
/// This trait belongs in the shared kernel as it's a cross-cutting concern used
/// by all bounded contexts. All domain events in the system should implement this trait.
pub trait DomainEvent: Send + Sync + std::fmt::Debug + std::any::Any {
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
}
