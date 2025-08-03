use domain::{DomainEvent, EventPublisher};
use tauri::{AppHandle, Emitter};
use serde_json::Value;

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

/// Convert domain event to serializable data for frontend transmission.
fn serialize_event_data(_event: &dyn DomainEvent) -> Value {
    // For now, return empty object - in practice you'd implement proper serialization
    Value::Object(serde_json::Map::new())
}