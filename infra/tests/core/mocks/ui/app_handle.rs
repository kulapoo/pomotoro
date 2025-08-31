use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;

/// Mock implementation of Tauri's AppHandle for testing
#[derive(Clone)]
pub struct MockAppHandle {
    /// Events emitted through the handle
    emitted_events: Arc<Mutex<Vec<EmittedEvent>>>,
    /// Command handlers registered
    command_handlers: Arc<Mutex<HashMap<String, Box<dyn Fn(Value) -> Value + Send + Sync>>>>,
    /// Event listeners
    event_listeners: Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(Value) + Send + Sync>>>>>,
}

#[derive(Debug, Clone)]
pub struct EmittedEvent {
    pub event_name: String,
    pub payload: Value,
    pub timestamp: std::time::Instant,
}

impl MockAppHandle {
    pub fn new() -> Self {
        Self {
            emitted_events: Arc::new(Mutex::new(Vec::new())),
            command_handlers: Arc::new(Mutex::new(HashMap::new())),
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Emit an event (mimics Tauri's emit functionality)
    pub fn emit<S: Serialize>(&self, event: &str, payload: S) -> Result<(), String> {
        let json_payload = serde_json::to_value(payload)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?;
        
        // Store the emitted event
        self.emitted_events.lock().unwrap().push(EmittedEvent {
            event_name: event.to_string(),
            payload: json_payload.clone(),
            timestamp: std::time::Instant::now(),
        });

        // Trigger any listeners
        if let Some(listeners) = self.event_listeners.lock().unwrap().get(event) {
            for listener in listeners {
                listener(json_payload.clone());
            }
        }

        Ok(())
    }

    /// Listen to an event
    pub fn listen<F>(&self, event: &str, handler: F) 
    where
        F: Fn(Value) + Send + Sync + 'static
    {
        self.event_listeners
            .lock()
            .unwrap()
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }

    /// Get all emitted events
    pub fn emitted_events(&self) -> Vec<EmittedEvent> {
        self.emitted_events.lock().unwrap().clone()
    }

    /// Get events of a specific type
    pub fn events_of_type(&self, event_type: &str) -> Vec<EmittedEvent> {
        self.emitted_events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.event_name == event_type)
            .cloned()
            .collect()
    }

    /// Clear all emitted events
    pub fn clear_events(&self) {
        self.emitted_events.lock().unwrap().clear();
    }

    /// Check if an event was emitted (convenience method)
    pub fn was_event_emitted(&self, event_type: &str) -> bool {
        !self.events_of_type(event_type).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_mock_app_handle() {
        let app_handle = MockAppHandle::new();
        
        // Test emitting events
        app_handle.emit("test_event", json!({"data": "test"})).unwrap();
        assert_eq!(app_handle.emitted_events().len(), 1);
        
        // Test event listener
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();
        
        app_handle.listen("test_event", move |payload| {
            received_clone.lock().unwrap().push(payload);
        });
        
        app_handle.emit("test_event", json!({"data": "test2"})).unwrap();
        
        // Check both emission and listener
        assert_eq!(app_handle.emitted_events().len(), 2);
        assert_eq!(received.lock().unwrap().len(), 1);
        
        // Test filtering by event type
        app_handle.emit("other_event", json!({"data": "other"})).unwrap();
        let test_events = app_handle.events_of_type("test_event");
        assert_eq!(test_events.len(), 2);
    }
}