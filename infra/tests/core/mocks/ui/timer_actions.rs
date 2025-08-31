use serde_json::{json, Value};
use domain::event_names::commands::timer as timer_commands;
use super::app_handle::MockAppHandle;

#[derive(Clone)]
/// Timer-specific UI actions
pub struct TimerUiActions {
    app_handle: MockAppHandle,
}

impl TimerUiActions {
    pub fn new(app_handle: MockAppHandle) -> Self {
        Self { app_handle }
    }

    /// Simulate clicking the Start button
    pub async fn click_start(&self) -> Value {
        self.app_handle.emit(timer_commands::START, json!({
            "timestamp": chrono::Utc::now().to_rfc3339()
        })).unwrap();

        json!({
            "status": "started",
            "phase": "Work",
            "remaining_seconds": 1500
        })
    }

    /// Simulate clicking the Pause button
    pub async fn click_pause(&self) -> Value {
        self.app_handle.emit(timer_commands::PAUSE, json!({})).unwrap();

        json!({
            "status": "paused",
            "paused_at": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Simulate clicking the Reset button
    pub async fn click_reset(&self) -> Value {
        self.app_handle.emit(timer_commands::RESET, json!({})).unwrap();

        json!({
            "status": "idle",
            "phase": "Work",
            "remaining_seconds": 1500
        })
    }

    /// Simulate clicking Skip Phase button
    pub async fn click_skip_phase(&self) -> Value {
        self.app_handle.emit(timer_commands::SKIP_PHASE, json!({})).unwrap();

        json!({
            "phase_skipped": true,
            "new_phase": "ShortBreak"
        })
    }

    /// Simulate switching active task
    pub async fn switch_task(&self, task_id: &str) -> Value {
        self.app_handle.emit(timer_commands::SWITCH_ACTIVE_TASK, json!({
            "task_id": task_id
        })).unwrap();

        json!({
            "active_task_id": task_id,
            "switched_at": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Get current timer state
    pub async fn get_state(&self) -> Value {
        self.app_handle.emit(timer_commands::GET_STATE, json!({})).unwrap();

        json!({
            "status": "idle",
            "phase": "Work",
            "remaining_seconds": 1500,
            "active_task_id": null,
            "session_count": 0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::core::mocks::MockEventBus;

    #[tokio::test]
    async fn test_timer_ui_actions() {
        let app_handle = MockAppHandle::new();
        let timer_actions = TimerUiActions::new(app_handle.clone());

        // Test timer start
        let result = timer_actions.click_start().await;
        assert_eq!(result["status"], "started");
        assert!(app_handle.was_event_emitted(timer_commands::START));

        // Test timer pause
        let result = timer_actions.click_pause().await;
        assert_eq!(result["status"], "paused");
        assert!(app_handle.was_event_emitted(timer_commands::PAUSE));

        // Test timer reset
        let result = timer_actions.click_reset().await;
        assert_eq!(result["status"], "idle");
        assert!(app_handle.was_event_emitted(timer_commands::RESET));
    }
}