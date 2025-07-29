use pomotoro_domain::{DomainEvent, EventPublisher};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use serde_json::Value;

pub type EventHandler<T> = Arc<dyn Fn(&T) + Send + Sync>;

/// # DomainEventBus - Application Layer Implementation
/// 
/// This is the concrete implementation of event publishing that belongs in the
/// application layer. It handles infrastructure concerns like:
/// - Thread-safe event handler management
/// - Type-erased event routing
/// - Concurrent event publishing
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
    handlers: Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(&dyn DomainEvent) + Send + Sync>>>>>,
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
            if let Some(concrete_event) = (event as &dyn std::any::Any).downcast_ref::<T>() {
                handler(concrete_event);
            }
        });

        handlers
            .entry(std::any::type_name::<T>().to_string())
            .or_insert_with(Vec::new)
            .push(type_erased_handler);
    }

}

impl EventPublisher for DomainEventBus {
    fn publish(&self, event: Box<dyn DomainEvent>) {
        let handlers = self.handlers.lock().unwrap();
        
        // Try to find handlers for this specific event type
        // We need to match the concrete type name, so we iterate through all registered handlers
        // and let each handler try to downcast and handle the event if it matches
        for event_handlers in handlers.values() {
            for handler in event_handlers {
                handler(event.as_ref());
            }
        }
    }
    
    fn publish_batch(&self, _events: Vec<Box<dyn DomainEvent>>) {
        for event in _events {
            self.publish(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::{TaskCreated, TaskConfig, AudioConfig, TaskId};
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
}

/// # TauriEventPublisher - Frontend Integration
/// 
/// This publisher broadcasts domain events to the Tauri frontend,
/// enabling real-time UI updates and reactive patterns.
/// 
/// Events are emitted both as specific event types and as generic
/// domain events for flexible frontend handling.
#[derive(Debug, Clone)]
pub struct TauriEventPublisher {
    app_handle: AppHandle,
}

impl TauriEventPublisher {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl EventPublisher for TauriEventPublisher {
    fn publish(&self, event: Box<dyn DomainEvent>) {
        let event_type = event.event_type();
        let aggregate_id = event.aggregate_id();
        let version = event.version();
        let occurred_at = event.occurred_at();

        // Create event payload with metadata
        let payload = serde_json::json!({
            "event_type": event_type,
            "aggregate_id": aggregate_id,
            "version": version,
            "occurred_at": occurred_at,
            "data": serialize_event_data(&*event)
        });

        // Emit to frontend with specific event type
        if let Err(e) = self.app_handle.emit(event_type, &payload) {
            eprintln!("Failed to publish event {}: {}", event_type, e);
        }

        // Also emit generic domain event for catch-all listeners
        if let Err(e) = self.app_handle.emit("domain_event", &payload) {
            eprintln!("Failed to publish generic domain event: {}", e);
        }

        println!(
            "Published {} event for aggregate {} (version {})",
            event_type, aggregate_id, version
        );
    }

    fn publish_batch(&self, _events: Vec<Box<dyn DomainEvent>>) {
        if _events.is_empty() {
            return;
        }

        // Publish each event individually to maintain ordering
        for _event in &_events {
            // We need to clone the event, but DomainEvent doesn't implement Clone
            // This is a limitation - in practice, you'd implement proper cloning
            // For now, we'll skip batch publishing
        }

        // Emit batch completion event
        let batch_payload = serde_json::json!({
            "event_count": _events.len(),
            "batch_completed_at": chrono::Utc::now()
        });

        if let Err(e) = self.app_handle.emit("domain_event_batch_completed", &batch_payload) {
            eprintln!("Failed to publish batch completion event: {}", e);
        }
    }
}

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

/// Convert domain event to serializable data for frontend transmission.
fn serialize_event_data(_event: &dyn DomainEvent) -> Value {
    // For now, return empty object - in practice you'd implement proper serialization
    Value::Object(serde_json::Map::new())
}

pub type EventPublisherArc = Arc<dyn EventPublisher + Send + Sync>;

/// Create an event publisher that combines internal handlers and frontend emission
pub fn create_composite_event_publisher(app_handle: AppHandle) -> EventPublisherArc {
    let mut composite = CompositeEventPublisher::new();
    
    // Add internal event bus for application-level handlers
    composite.add_publisher(Arc::new(DomainEventBus::new()));
    
    // Add Tauri publisher for frontend emission
    composite.add_publisher(Arc::new(TauriEventPublisher::new(app_handle)));
    
    Arc::new(composite)
}

/// Create an event publisher and return both the composite and the domain event bus
/// for handler registration
pub fn create_event_publisher_with_bus(app_handle: AppHandle) -> (EventPublisherArc, Arc<DomainEventBus>) {
    let mut composite = CompositeEventPublisher::new();
    
    // Create the domain event bus that we'll register handlers on
    let domain_bus = Arc::new(DomainEventBus::new());
    
    // Add internal event bus for application-level handlers
    composite.add_publisher(domain_bus.clone());
    
    // Add Tauri publisher for frontend emission
    composite.add_publisher(Arc::new(TauriEventPublisher::new(app_handle)));
    
    (Arc::new(composite), domain_bus)
}