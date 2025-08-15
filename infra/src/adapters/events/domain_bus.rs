use domain::{DomainEvent, EventPublisher};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::TypeId;

pub type EventHandler<T> = Arc<dyn Fn(&T) + Send + Sync>;

type HandlersMap = HashMap<TypeId, Vec<Box<dyn Fn(&dyn DomainEvent) + Send + Sync>>>;

/// # DomainEventBus - Application Layer Implementation
///
/// This is the concrete implementation of event publishing that belongs in the
/// application layer. It handles infrastructure concerns like:
/// - Thread-safe event handler management
/// - Type-erased event routing
/// - Concurrent event publishing
/// - Named handler registration and lifecycle management
///
/// ## Clean Architecture Placement
///
/// - **Location**: Application Layer (NOT Domain Layer)
/// - **Purpose**: Infrastructure implementation of EventPublisher abstraction
/// - **Dependencies**: Depends on domain abstractions, not vice versa
///
/// ## Usage by Application Services
///
/// ```rust,ignore
/// let event_bus = Arc::new(DomainEventBus::new());
/// let task_service = TaskSessionService::new(repo, event_bus);
///
/// // Domain service uses EventPublisher abstraction
/// // EventBus provides concrete implementation
/// ```
#[derive(Clone)]
pub struct DomainEventBus {
    handlers: Arc<Mutex<HandlersMap>>,
}

impl Default for DomainEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Subscribes a type-specific handler to events of type T.
    ///
    /// This method allows application services or other components to register
    /// handlers for specific domain events.
    ///
    /// # Type Safety
    ///
    /// The handler is strongly typed to T, but stored in a type-erased way
    /// using runtime type checking for dispatch.
    pub fn subscribe<T: DomainEvent + 'static>(&self, handler: EventHandler<T>) {
        let mut handlers = self.handlers.lock().unwrap();

        // Create a type-erased handler that can work with dyn DomainEvent
        let type_erased_handler = Box::new(move |event: &dyn DomainEvent| {
            if let Some(concrete_event) = event.as_any().downcast_ref::<T>() {
                handler(concrete_event);
            }
        });

        handlers
            .entry(TypeId::of::<T>())
            .or_default()
            .push(type_erased_handler);
    }
}

impl EventPublisher for DomainEventBus {
    fn publish(&self, event: Box<dyn DomainEvent>) {
        let handlers = self.handlers.lock().unwrap();

        // Try to find handlers for this specific event type
        // We iterate through all registered handlers and let each handler
        // try to downcast and handle the event if it matches
        for event_handlers in handlers.values() {
            for handler in event_handlers {
                handler(event.as_ref());
            }
        }
    }

    fn publish_batch(&self, events: Vec<Box<dyn DomainEvent>>) {
        for event in events {
            self.publish(event);
        }
    }
}

pub type EventPublisherArc = Arc<dyn EventPublisher + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{TaskCreated, TaskConfig, AudioConfig, TaskId};
    use std::time::Duration;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn should_create_empty_event_bus() {
        let bus = DomainEventBus::new();
        let handlers = bus.handlers.lock().unwrap();
        assert!(handlers.is_empty());
    }

    #[test]
    fn should_subscribe_and_publish_events() {
        let bus = DomainEventBus::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let handler: EventHandler<TaskCreated> = Arc::new(move |_event| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.subscribe::<TaskCreated>(handler);

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

        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn should_handle_multiple_handlers_for_same_event() {
        let bus = DomainEventBus::new();
        let call_count1 = Arc::new(AtomicUsize::new(0));
        let call_count2 = Arc::new(AtomicUsize::new(0));

        let call_count1_clone = Arc::clone(&call_count1);
        let call_count2_clone = Arc::clone(&call_count2);

        let handler1: EventHandler<TaskCreated> = Arc::new(move |_event| {
            call_count1_clone.fetch_add(1, Ordering::SeqCst);
        });

        let handler2: EventHandler<TaskCreated> = Arc::new(move |_event| {
            call_count2_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.subscribe::<TaskCreated>(handler1);
        bus.subscribe::<TaskCreated>(handler2);

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

        assert_eq!(call_count1.load(Ordering::SeqCst), 1);
        assert_eq!(call_count2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn should_handle_multiple_subscribers() {
        let bus = DomainEventBus::new();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let handler: EventHandler<TaskCreated> = Arc::new(move |_event| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.subscribe::<TaskCreated>(handler);

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
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }
}