use domain::{Event, EventPublisher};
use super::EventHandler;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::adapters::events::EventSubscriber;

/// Metadata for a registered event handler
struct HandlerMetadata {
    /// Unique ID for this handler instance
    id: u64,
    /// Name of the handler (from EventHandler::name())
    name: String,
    /// The actual closure that executes the handler
    handler_fn: Arc<dyn Fn(&dyn Event) + Send + Sync>,
}

type HandlersMap = HashMap<TypeId, Vec<HandlerMetadata>>;

pub trait EventBus: EventPublisher + EventSubscriber + Send + Sync {}

/// # InMemoryEventBus - In-Memory Event Bus Implementation
///
/// A simplified async event bus that bridges sync publish with async handlers.
///
/// ## Why Async is Needed
///
/// Event handlers often need to perform I/O operations:
/// - Database writes (sqlx, diesel)
/// - HTTP calls (reqwest, hyper)
/// - Cache updates (redis)
/// - File operations (tokio::fs)
///
/// Since modern Rust libraries are async-first, handlers must be async.
/// But EventPublisher::publish() is sync (domain layer stays simple).
/// So we spawn async tasks to bridge this gap.
///
/// ## Clean Architecture Placement
///
/// - **Location**: Infrastructure Layer
/// - **Purpose**: Concrete implementation of EventPublisher abstraction
/// - **Dependencies**: Depends on domain abstractions, not vice versa
///
/// ## Implementation Details
///
/// - Handlers are stored with metadata (id, name) to support unsubscribe operations
/// - Each handler gets a unique ID for precise removal
/// - Handler names enable searching and debugging
/// - Handlers with duplicate names are allowed but have unique IDs
///
/// ## Thread Safety
///
/// - All operations use Mutex for thread-safe access
/// - Lock poisoning will cause panics (fail-fast approach)
/// - Consider using parking_lot::Mutex for production to avoid poisoning
///
/// ## Future Enhancements (TODOs)
///
/// ```rust,ignore
/// // TODO: Add bounded concurrency control with semaphore
/// // TODO: Add handler execution metrics and monitoring
/// // TODO: Add dead letter queue for failed events
/// // TODO: Add timeout protection for slow handlers
/// // TODO: Add retry logic for transient failures
/// // TODO: Add event replay capability
/// // TODO: Add handler priority/ordering support
/// // TODO: Add distributed event bus support (Redis, RabbitMQ)
/// // TODO: Replace std::sync::Mutex with parking_lot::Mutex
/// // TODO: Add proper async testing utilities instead of sleep()
/// ```
#[derive(Clone)]
pub struct InMemoryEventBus {
    handlers: Arc<Mutex<HandlersMap>>,
    next_handler_id: Arc<AtomicU64>,
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryEventBus {
    /// Creates a new empty event bus
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            next_handler_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Returns the total number of registered handlers across all event types
    pub fn handler_count(&self) -> usize {
        let handlers = self.handlers.lock().unwrap_or_else(|e| e.into_inner());
        handlers.values().map(|v| v.len()).sum()
    }

    /// Returns the number of handlers for a specific event type
    pub fn handler_count_for_type(&self, event_type: TypeId) -> usize {
        let handlers = self.handlers.lock().unwrap_or_else(|e| e.into_inner());
        handlers.get(&event_type).map(|v| v.len()).unwrap_or(0)
    }

    /// Lists all handler names for a specific event type
    pub fn list_handlers_for_type(&self, event_type: TypeId) -> Vec<String> {
        let handlers = self.handlers.lock().unwrap_or_else(|e| e.into_inner());
        handlers
            .get(&event_type)
            .map(|handlers| handlers.iter().map(|h| h.name.clone()).collect())
            .unwrap_or_default()
    }

    /// Returns handler IDs for a specific event type (useful for debugging)
    pub fn list_handler_ids_for_type(&self, event_type: TypeId) -> Vec<u64> {
        let handlers = self.handlers.lock().unwrap_or_else(|e| e.into_inner());
        handlers
            .get(&event_type)
            .map(|handlers| handlers.iter().map(|h| h.id).collect())
            .unwrap_or_default()
    }
}

impl EventPublisher for InMemoryEventBus {
    fn publish(&self, event: Box<dyn Event>) {
        // Check for runtime availability early
        if tokio::runtime::Handle::try_current().is_err() {
            // TODO: Use proper logging
            eprintln!("Warning: No tokio runtime available for publishing events");
            return;
        }

        let handlers = self.handlers.lock().unwrap_or_else(|e| e.into_inner());
        let type_id = event.as_any().type_id();

        if let Some(event_handlers) = handlers.get(&type_id) {
            // Execute each handler's closure
            for handler_meta in event_handlers {
                (handler_meta.handler_fn)(event.as_ref());
            }
        }
    }

    fn publish_batch(&self, events: Vec<Box<dyn Event>>) {
        for event in events {
            self.publish(event);
        }
    }
}

impl EventSubscriber for InMemoryEventBus {
    fn subscribe(&mut self, handler: Box<dyn EventHandler>) -> domain::Result<()> {
        let event_type = handler.subscribes_to();
        let handler_name = handler.name().to_string();
        let handler_id = self.next_handler_id.fetch_add(1, Ordering::SeqCst);
        let handler_arc = Arc::new(handler);

        // Log subscription
        eprintln!("Subscribing handler '{}' with ID {} for event type", handler_name, handler_id);

        // Create a closure that spawns an async task for the handler
        let handler_fn = Arc::new(move |event: &dyn Event| {
            let event_box = event.clone_box();
            let handler_clone = Arc::clone(&handler_arc);

            // Get current tokio runtime or return if none available
            let handle = match tokio::runtime::Handle::try_current() {
                Ok(handle) => handle,
                Err(_) => {
                    // TODO: Use proper logging framework (tracing/log)
                    eprintln!("Warning: No tokio runtime available for event handler");
                    return;
                }
            };

            // Spawn async task to handle the event
            handle.spawn(async move {
                // Execute the async handler
                if let Err(e) = handler_clone.handle(event_box).await {
                    // TODO: Add proper error handling/logging
                    // TODO: Add to dead letter queue
                    eprintln!("Event handler error: {}", e);
                }
            });
        }) as Arc<dyn Fn(&dyn Event) + Send + Sync>;

        // Store handler with metadata
        let handler_metadata = HandlerMetadata {
            id: handler_id,
            name: handler_name,
            handler_fn,
        };

        self.handlers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entry(event_type)
            .or_default()
            .push(handler_metadata);

        Ok(())
    }

    fn clear_handlers_for_type(&mut self, event_type: TypeId) -> domain::Result<()> {
        let mut handlers = self.handlers.lock().unwrap_or_else(|e| e.into_inner());
        let removed_count = handlers.remove(&event_type).map(|v| v.len()).unwrap_or(0);

        if removed_count > 0 {
            // TODO: Use proper logging
            eprintln!("Cleared {} handlers for event type", removed_count);
        }

        Ok(())
    }

    fn unsubscribe(&mut self, handler: Box<dyn EventHandler>) -> domain::Result<()> {
        // Use the handler's type and name to find and remove it
        let event_type = handler.subscribes_to();
        let handler_name = handler.name();

        let mut handlers = self.handlers.lock().unwrap_or_else(|e| e.into_inner());

        if let Some(event_handlers) = handlers.get_mut(&event_type) {
            // Remove all handlers with matching name
            let initial_len = event_handlers.len();
            event_handlers.retain(|h| h.name != handler_name);
            let removed_count = initial_len - event_handlers.len();

            if removed_count > 0 {
                // TODO: Use proper logging
                eprintln!(
                    "Unsubscribed {} handler(s) named '{}'",
                    removed_count, handler_name
                );

                // If no handlers left for this type, remove the entry
                if event_handlers.is_empty() {
                    handlers.remove(&event_type);
                }
            }
        }

        Ok(())
    }

    fn unsubscribe_by_name(
        &mut self,
        event_type: TypeId,
        handler_name: &str,
    ) -> domain::Result<bool> {
        let mut handlers = self.handlers.lock().unwrap_or_else(|e| e.into_inner());

        if let Some(event_handlers) = handlers.get_mut(&event_type) {
            let initial_len = event_handlers.len();
            event_handlers.retain(|h| h.name != handler_name);
            let removed_count = initial_len - event_handlers.len();

            if removed_count > 0 {
                // TODO: Use proper logging
                eprintln!(
                    "Unsubscribed {} handler(s) named '{}' from event type",
                    removed_count, handler_name
                );

                // If no handlers left for this type, remove the entry
                if event_handlers.is_empty() {
                    handlers.remove(&event_type);
                }

                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl EventBus for InMemoryEventBus {}

pub type EventPublisherArc = Arc<dyn EventPublisher + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::{AudioConfig, Result, TaskConfig, TaskCreated, TaskId};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::sync::Notify;
    use std::sync::Arc;

    // Improved test handler that counts invocations and signals completion
    struct TestEventHandler {
        name: String,
        counter: Arc<AtomicUsize>,
        event_type: TypeId,
        // Signal when handler has been called
        notify: Arc<Notify>,
    }

    impl TestEventHandler {
        fn new(name: &str, counter: Arc<AtomicUsize>) -> Self {
            Self {
                name: name.to_string(),
                counter,
                event_type: TypeId::of::<TaskCreated>(),
                notify: Arc::new(Notify::new()),
            }
        }

        fn new_with_notify(name: &str, counter: Arc<AtomicUsize>, notify: Arc<Notify>) -> Self {
            Self {
                name: name.to_string(),
                counter,
                event_type: TypeId::of::<TaskCreated>(),
                notify,
            }
        }

        async fn wait_for_call(&self, timeout_ms: u64) -> bool {
            tokio::time::timeout(
                Duration::from_millis(timeout_ms),
                self.notify.notified()
            ).await.is_ok()
        }
    }

    #[async_trait]
    impl EventHandler for TestEventHandler {
        fn subscribes_to(&self) -> TypeId {
            self.event_type
        }

        async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
            if event.as_any().is::<TaskCreated>() {
                self.counter.fetch_add(1, Ordering::SeqCst);
                self.notify.notify_one();
            }
            Ok(())
        }

        fn name(&self) -> &'static str {
            // FIXED: No memory leak - store name as static in test
            match self.name.as_str() {
                "Handler1" => "Handler1",
                "Handler2" => "Handler2",
                "TestHandler" => "TestHandler",
                "RemovableHandler" => "RemovableHandler",
                "BatchHandler" => "BatchHandler",
                _ => "UnknownHandler",
            }
        }
    }

    #[test]
    fn should_create_empty_event_bus() {
        let bus = InMemoryEventBus::new();
        assert_eq!(bus.handler_count(), 0);
    }

    #[tokio::test]
    async fn should_subscribe_and_publish_events() {
        let mut bus = InMemoryEventBus::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let notify = Arc::new(Notify::new());

        let handler = TestEventHandler::new_with_notify("Handler1", Arc::clone(&call_count), Arc::clone(&notify));
        bus.subscribe(Box::new(handler)).unwrap();

        assert_eq!(bus.handler_count(), 1);
        assert_eq!(bus.handler_count_for_type(TypeId::of::<TaskCreated>()), 1);

        let task_created = TaskCreated::new(
            TaskId::new(),
            "Test Task".to_string(),
            None,
            4,
            vec![],
            TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            },
            AudioConfig::default(),
            1,
        );

        bus.publish(Box::new(task_created));

        // FIXED: Wait for handler completion instead of arbitrary sleep
        tokio::time::timeout(Duration::from_millis(100), notify.notified())
            .await
            .expect("Handler should complete within 100ms");

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn should_handle_multiple_handlers_for_same_event() {
        let mut bus = InMemoryEventBus::new();
        let call_count1 = Arc::new(AtomicUsize::new(0));
        let call_count2 = Arc::new(AtomicUsize::new(0));
        let notify1 = Arc::new(Notify::new());
        let notify2 = Arc::new(Notify::new());

        let handler1 = TestEventHandler::new_with_notify("Handler1", Arc::clone(&call_count1), Arc::clone(&notify1));
        let handler2 = TestEventHandler::new_with_notify("Handler2", Arc::clone(&call_count2), Arc::clone(&notify2));

        bus.subscribe(Box::new(handler1)).unwrap();
        bus.subscribe(Box::new(handler2)).unwrap();

        assert_eq!(bus.handler_count(), 2);

        let task_created = TaskCreated::new(
            TaskId::new(),
            "Test Task".to_string(),
            None,
            4,
            vec![],
            TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            },
            AudioConfig::default(),
            1,
        );

        bus.publish(Box::new(task_created));

        // Wait for both handlers
        let (r1, r2) = tokio::join!(
            tokio::time::timeout(Duration::from_millis(100), notify1.notified()),
            tokio::time::timeout(Duration::from_millis(100), notify2.notified())
        );

        assert!(r1.is_ok(), "Handler1 should complete");
        assert!(r2.is_ok(), "Handler2 should complete");

        assert_eq!(call_count1.load(Ordering::SeqCst), 1);
        assert_eq!(call_count2.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn should_unsubscribe_by_name() {
        let mut bus = InMemoryEventBus::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let notify = Arc::new(Notify::new());

        let handler = TestEventHandler::new_with_notify("RemovableHandler", Arc::clone(&call_count), Arc::clone(&notify));
        bus.subscribe(Box::new(handler)).unwrap();

        assert_eq!(bus.handler_count(), 1);

        // Publish first event - handler should be called
        let task1 = TaskCreated::new(
            TaskId::new(),
            "Task 1".to_string(),
            None,
            4,
            vec![],
            TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            },
            AudioConfig::default(),
            1,
        );

        bus.publish(Box::new(task1));

        tokio::time::timeout(Duration::from_millis(100), notify.notified())
            .await
            .expect("Handler should complete");

        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Unsubscribe the handler
        let removed = bus
            .unsubscribe_by_name(TypeId::of::<TaskCreated>(), "RemovableHandler")
            .unwrap();
        assert!(removed);
        assert_eq!(bus.handler_count(), 0);

        // Publish second event - handler should NOT be called
        let task2 = TaskCreated::new(
            TaskId::new(),
            "Task 2".to_string(),
            None,
            4,
            vec![],
            TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            },
            AudioConfig::default(),
            2,
        );

        bus.publish(Box::new(task2));

        // Small delay to ensure no handler runs
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Count should still be 1
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn should_unsubscribe_handler() {
        let mut bus = InMemoryEventBus::new();
        let call_count = Arc::new(AtomicUsize::new(0));

        let handler = TestEventHandler::new("TestHandler", Arc::clone(&call_count));
        bus.subscribe(Box::new(handler)).unwrap();

        assert_eq!(bus.handler_count(), 1);

        // Create another handler with same name to test unsubscribe
        let handler_to_remove = TestEventHandler::new("TestHandler", Arc::new(AtomicUsize::new(0)));
        bus.unsubscribe(Box::new(handler_to_remove)).unwrap();

        assert_eq!(bus.handler_count(), 0);
    }

    #[tokio::test]
    async fn should_clear_handlers_for_type() {
        let mut bus = InMemoryEventBus::new();
        let call_count = Arc::new(AtomicUsize::new(0));

        let handler = TestEventHandler::new("Handler1", Arc::clone(&call_count));
        bus.subscribe(Box::new(handler)).unwrap();

        let handler2 = TestEventHandler::new("Handler2", Arc::new(AtomicUsize::new(0)));
        bus.subscribe(Box::new(handler2)).unwrap();

        assert_eq!(bus.handler_count(), 2);

        // Clear all handlers for TaskCreated
        bus.clear_handlers_for_type(TypeId::of::<TaskCreated>())
            .unwrap();

        assert_eq!(bus.handler_count(), 0);

        let task_created = TaskCreated::new(
            TaskId::new(),
            "Test Task".to_string(),
            None,
            4,
            vec![],
            TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            },
            AudioConfig::default(),
            1,
        );

        bus.publish(Box::new(task_created));

        // Small delay to ensure no handler runs
        tokio::time::sleep(Duration::from_millis(50)).await;

        // No handlers should have been called
        assert_eq!(call_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn should_list_handlers_for_type() {
        let mut bus = InMemoryEventBus::new();

        let handler1 = TestEventHandler::new("Handler1", Arc::new(AtomicUsize::new(0)));
        let handler2 = TestEventHandler::new("Handler2", Arc::new(AtomicUsize::new(0)));

        bus.subscribe(Box::new(handler1)).unwrap();
        bus.subscribe(Box::new(handler2)).unwrap();

        let handlers = bus.list_handlers_for_type(TypeId::of::<TaskCreated>());
        assert_eq!(handlers.len(), 2);
        assert!(handlers.contains(&"Handler1".to_string()));
        assert!(handlers.contains(&"Handler2".to_string()));

        // Also test listing handler IDs
        let ids = bus.list_handler_ids_for_type(TypeId::of::<TaskCreated>());
        assert_eq!(ids.len(), 2);
    }

    #[tokio::test]
    async fn should_handle_batch_publish() {
        let mut bus = InMemoryEventBus::new();
        let call_count = Arc::new(AtomicUsize::new(0));

        // Use a simple handler without notify for batch test
        let handler = TestEventHandler::new("BatchHandler", Arc::clone(&call_count));
        bus.subscribe(Box::new(handler)).unwrap();

        let events: Vec<Box<dyn Event>> = (0..3)
            .map(|i| {
                Box::new(TaskCreated::new(
                    TaskId::new(),
                    format!("Task {}", i),
                    None,
                    4,
                    vec![],
                    TaskConfig {
                        work_duration: Duration::from_secs(1500),
                        short_break_duration: Duration::from_secs(300),
                        long_break_duration: Duration::from_secs(900),
                        sessions_until_long_break: 4,
                        enable_screen_blocking: false,
                    },
                    AudioConfig::default(),
                    1,
                )) as Box<dyn Event>
            })
            .collect();

        bus.publish_batch(events);

        // Give handlers time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn should_handle_duplicate_handler_names() {
        let mut bus = InMemoryEventBus::new();

        // Register two handlers with the same name
        let handler1 = TestEventHandler::new("Handler1", Arc::new(AtomicUsize::new(0)));
        let handler2 = TestEventHandler::new("Handler1", Arc::new(AtomicUsize::new(0)));

        bus.subscribe(Box::new(handler1)).unwrap();
        bus.subscribe(Box::new(handler2)).unwrap();

        assert_eq!(bus.handler_count(), 2);

        // Both should have the same name but different IDs
        let names = bus.list_handlers_for_type(TypeId::of::<TaskCreated>());
        assert_eq!(names.len(), 2);
        assert!(names.iter().all(|n| n == "Handler1"));

        let ids = bus.list_handler_ids_for_type(TypeId::of::<TaskCreated>());
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1], "Handlers should have different IDs");
    }

    #[test]
    fn should_handle_no_runtime_gracefully() {
        // This test runs without tokio runtime
        let bus = InMemoryEventBus::new();

        let task_created = TaskCreated::new(
            TaskId::new(),
            "Test Task".to_string(),
            None,
            4,
            vec![],
            TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            },
            AudioConfig::default(),
            1,
        );

        // Should not panic, just log warning
        bus.publish(Box::new(task_created));
    }
}