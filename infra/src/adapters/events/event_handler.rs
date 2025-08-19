use async_trait::async_trait;
use std::any::TypeId;

use domain::{Event, Result};

/// Handles domain events asynchronously in the infrastructure layer.
///
/// Event handlers are responsible for processing domain events that represent
/// state changes or significant occurrences within the system. This trait provides
/// a type-safe mechanism for subscribing to specific event types and processing
/// them asynchronously.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow concurrent event processing
/// across multiple async tasks and threads.
///
/// # Examples
///
/// ```rust
/// use async_trait::async_trait;
/// use std::any::TypeId;
/// use domain::{Event, Result, TaskCompleted, Error};
/// use infra::adapters::events::EventHandler;
///
/// struct TaskCompletedHandler;
///
/// #[async_trait]
/// impl EventHandler for TaskCompletedHandler {
///     fn subscribes_to(&self) -> TypeId {
///         TypeId::of::<TaskCompleted>()
///     }
///
///     async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
///         let task_completed = event.as_any().downcast_ref::<TaskCompleted>().ok_or_else(|| {
///             return Error::InvalidLifecycle { message: "Failed to downcast event to TaskCompleted".into()};
///         })?;
///         // Process the task completion event
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Returns the TypeId of the event type this handler subscribes to.
    ///
    /// This method is used by the event dispatcher to route events to the
    /// appropriate handlers. Each handler should subscribe to exactly one
    /// event type.
    ///
    /// # Returns
    ///
    /// The `TypeId` of the specific event type this handler processes.
    fn subscribes_to(&self) -> TypeId;

    /// Processes a domain event asynchronously.
    ///
    /// This method receives a boxed event and should downcast it to the
    /// specific event type the handler subscribes to. The handler should
    /// perform any necessary side effects, such as updating read models,
    /// sending notifications, or triggering external integrations.
    ///
    /// # Parameters
    ///
    /// * `event` - A boxed trait object containing the event to process
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure of event processing.
    /// Errors should be handled gracefully by the event dispatcher.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Event downcasting fails (wrong event type)
    /// - Infrastructure operations fail (database, network, etc.)
    /// - Business logic validation fails
    async fn handle(&self, event: Box<dyn Event>) -> Result<()>;

    /// Returns a human-readable name for this event handler.
    ///
    /// By default, this returns the full type name of the implementing struct.
    /// Override this method to provide a more user-friendly name for logging
    /// and debugging purposes.
    ///
    /// # Returns
    ///
    /// A static string slice containing the handler's name.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
