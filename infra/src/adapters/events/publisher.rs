use domain::{
    DomainEvent, EventPublisher,
    // Timer Events
    TimerStarted, TimerPaused, TimerReset, PhaseCompleted, PhaseSkipped,
    TimerStatusChanged, ActiveTaskSwitched, BreakSessionStarted, BreakSessionCompleted,
    WorkSessionStarted, WorkSessionCompleted, SessionStarted, SessionFlowReset,
    // Task Events
    TaskCreated, TaskCompleted, TaskSessionCompleted, TaskStatusChanged, TaskUpdated,
    SessionTransitionCompleted, TaskSwitchWorkflowCompleted, AutomaticTaskCyclingCompleted,
    TaskCyclingExhausted,
    // UI Events
    events::ui,
};
use serde_json::Value;
use tauri::{AppHandle, Emitter};

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

    fn publish_batch(&self, events: Vec<Box<dyn DomainEvent>>) {
        if events.is_empty() {
            return;
        }

        let event_count = events.len();

        // Publish each event individually to maintain ordering
        for event in events {
            self.publish(event);
        }

        // Emit batch completion event
        let batch_payload = serde_json::json!({
            "event_count": event_count,
            "batch_completed_at": chrono::Utc::now()
        });

        if let Err(e) = self
            .app_handle
            .emit("domain_event_batch_completed", &batch_payload)
        {
            eprintln!("Failed to publish batch completion event: {}", e);
        }
    }
}

impl TauriEventPublisher {
    /// Publish a UI event to the frontend.
    ///
    /// UI events are frontend-specific events that don't correspond to domain events
    /// but are used for UI state synchronization and updates.
    pub fn publish_ui_event(&self, event_type: &str, data: Value) {
        let payload = serde_json::json!({
            "type": event_type,
            "data": data,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        if let Err(e) = self.app_handle.emit("ui_event", &payload) {
            eprintln!("Failed to publish UI event '{}': {}", event_type, e);
        }
    }

    /// Publish timer UI events
    pub fn publish_timer_tick(&self, remaining_seconds: u64) {
        self.publish_ui_event(ui::timer::TICK, serde_json::json!({
            "remaining_seconds": remaining_seconds
        }));
    }

    pub fn publish_timer_status_changed(&self, status: &str) {
        self.publish_ui_event(ui::timer::STATUS_CHANGED, serde_json::json!({
            "status": status
        }));
    }

    pub fn publish_timer_phase_event(&self, phase: &str) {
        self.publish_ui_event(ui::timer::PHASE_EVENT, serde_json::json!({
            "phase": phase
        }));
    }

    pub fn publish_timer_state_updated(&self, state: Value) {
        self.publish_ui_event(ui::timer::STATE_UPDATED, state);
    }

    /// Publish task UI events
    pub fn publish_task_list_updated(&self, tasks: Value) {
        self.publish_ui_event(ui::task::LIST_UPDATED, tasks);
    }

    pub fn publish_task_active_changed(&self, task_id: &str) {
        self.publish_ui_event(ui::task::ACTIVE_CHANGED, serde_json::json!({
            "task_id": task_id
        }));
    }

    pub fn publish_task_progress_updated(&self, task_id: &str, progress: f64) {
        self.publish_ui_event(ui::task::PROGRESS_UPDATED, serde_json::json!({
            "task_id": task_id,
            "progress": progress
        }));
    }

    /// Publish config UI events
    pub fn publish_config_settings_updated(&self, settings: Value) {
        self.publish_ui_event(ui::config::SETTINGS_UPDATED, settings);
    }

    pub fn publish_config_theme_changed(&self, theme: &str) {
        self.publish_ui_event(ui::config::THEME_CHANGED, serde_json::json!({
            "theme": theme
        }));
    }
}

/// Convert domain event to serializable data for frontend transmission.
///
/// This function uses type downcasting to convert domain events to JSON values
/// that can be sent to the Tauri frontend. Each event type is handled individually
/// to ensure proper serialization of all event-specific data.
///
/// Uses `.unwrap_or_default()` for graceful error handling - returns `Value::Null`
/// instead of panicking if serialization fails.
fn serialize_event_data(event: &dyn DomainEvent) -> Value {
    use std::any::Any;

    let any_event = event as &dyn Any;


    // Timer Events
    if let Some(e) = any_event.downcast_ref::<TimerStarted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<TimerPaused>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<TimerReset>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<PhaseCompleted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<PhaseSkipped>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<TimerStatusChanged>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<ActiveTaskSwitched>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<BreakSessionStarted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<BreakSessionCompleted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<WorkSessionStarted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<WorkSessionCompleted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<SessionStarted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<SessionFlowReset>() {
        serde_json::to_value(e).unwrap_or_default()

    // Task Events
    } else if let Some(e) = any_event.downcast_ref::<TaskCreated>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<TaskCompleted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<TaskSessionCompleted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<TaskStatusChanged>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<TaskUpdated>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<SessionTransitionCompleted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<TaskSwitchWorkflowCompleted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<AutomaticTaskCyclingCompleted>() {
        serde_json::to_value(e).unwrap_or_default()
    } else if let Some(e) = any_event.downcast_ref::<TaskCyclingExhausted>() {
        serde_json::to_value(e).unwrap_or_default()

    } else {
        Value::Object(serde_json::Map::new())
    }
}
