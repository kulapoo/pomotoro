use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use serde_json::{json, Value};
use domain::shared_kernel::events::EventPublisher;
use domain::event_names::ui_listeners::{timer as timer_events, task as task_events, config as config_events, app as app_events};

use super::{
    app_handle::MockAppHandle,
    timer_actions::TimerUiActions,
    task_actions::TaskUiActions,
    config_actions::ConfigUiActions,
    audio_actions::AudioUiActions,
    response::UiResponse,
};

/// Simple UI Simulator for integration testing
///
/// This simulator provides:
/// - UI action modules (timer, task, config, audio) to simulate user interactions
/// - Event tracking via MockAppHandle
/// - Simple response handling for testing event flow
#[derive(Clone)]
pub struct UiSimulator {
    app_handle: MockAppHandle,
    #[allow(unused)]
    event_bus: Arc<dyn EventPublisher>,
    response_tx: mpsc::UnboundedSender<UiResponse>,
    response_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<UiResponse>>>>,

    // UI action modules
    pub timer: TimerUiActions,
    pub task: TaskUiActions,
    pub config: ConfigUiActions,
    pub audio: AudioUiActions,
}

impl UiSimulator {
    pub fn new(event_bus: Arc<dyn EventPublisher>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let app_handle = MockAppHandle::new();

        // Setup basic event listener for timer ticks
        let tx_clone = tx.clone();
        app_handle.listen(timer_events::TICK, move |payload| {
            let _ = tx_clone.send(UiResponse::EventReceived {
                event_type: timer_events::TICK.to_string(),
                payload,
            });
        });

        Self {
            timer: TimerUiActions::new(app_handle.clone()),
            task: TaskUiActions::new(app_handle.clone()),
            config: ConfigUiActions::new(app_handle.clone()),
            audio: AudioUiActions::new(app_handle.clone()),
            app_handle,
            event_bus,
            response_tx: tx,
            response_rx: Arc::new(Mutex::new(Some(rx))),
        }
    }

    /// Get the mock app handle for checking emitted events
    pub fn app_handle(&self) -> &MockAppHandle {
        &self.app_handle
    }

    /// Start auto-responder that logs UI events
    /// Returns self for continued use
    pub fn start_listen_to_events(self) -> Self {
        // Take the receiver and start processing in background
        if let Some(mut rx) = self.response_rx.lock().unwrap().take() {
            tokio::spawn(async move {
                println!("UI auto-responder started");
                while let Some(response) = rx.recv().await {
                    // Simple logging of responses
                    match response {
                        UiResponse::TimerTick { remaining_seconds } => {
                            tracing::trace!("UI: Timer tick - {} seconds", remaining_seconds);
                        }
                        _ => {
                            tracing::trace!("UI: Event received");
                        }
                    }
                }
            });
        }

        self
    }

    /// Trigger a UI response for testing
    pub fn trigger_response(&self, response: UiResponse) {
        let _ = self.response_tx.send(response);
    }

    /// Simulate acknowledging timer ticks
    pub async fn acknowledge_ticks(&self, count: usize) {
        for i in 0..count {
            self.trigger_response(UiResponse::TimerTick {
                remaining_seconds: 1500 - (i as u32 * 30),
            });
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Respond to a domain event
    pub fn respond_to_event(&self, event_type: &str, payload: Value) {
        let response = match event_type {
            timer_events::TICK => UiResponse::TimerTick {
                remaining_seconds: payload["remaining_seconds"].as_u64().unwrap_or(0) as u32,
            },
            timer_events::STATUS_CHANGED => UiResponse::TimerStatusChanged {
                status: payload["status"].as_str().unwrap_or("unknown").to_string(),
            },
            timer_events::PHASE_EVENT => UiResponse::TimerPhaseEvent {
                phase: payload["phase"].as_str().unwrap_or("Work").to_string(),
            },
            task_events::LIST_UPDATED => UiResponse::TaskListUpdated {
                tasks: payload["tasks"].as_array().cloned().unwrap_or_default(),
            },
            task_events::ACTIVE_CHANGED => UiResponse::TaskActiveChanged {
                task_id: payload["task_id"].as_str().unwrap_or("").to_string(),
            },
            config_events::SETTINGS_UPDATED => UiResponse::ConfigSettingsUpdated {
                settings: payload,
            },
            _ => UiResponse::EventReceived {
                event_type: event_type.to_string(),
                payload,
            },
        };

        self.trigger_response(response);
    }

    /// Simulate UI disconnect (stop responding)
    pub fn simulate_disconnect(&self) {
        // Simply stop sending responses
        // The channel remains open but we stop using it
        tracing::trace!("UI: Simulating disconnect");
    }
}

/// Builder for UiSimulator with custom configuration
pub struct UiSimulatorBuilder {
    auto_acknowledge_ticks: bool,
    auto_acknowledge_state_updates: bool,
    response_delay_ms: u64,
    initial_config: Option<Value>,
}

impl UiSimulatorBuilder {
    pub fn new() -> Self {
        Self {
            auto_acknowledge_ticks: true,
            auto_acknowledge_state_updates: true,
            response_delay_ms: 10,
            initial_config: None,
        }
    }

    pub fn with_auto_acknowledge_ticks(mut self, enabled: bool) -> Self {
        self.auto_acknowledge_ticks = enabled;
        self
    }

    pub fn with_auto_acknowledge_state_updates(mut self, enabled: bool) -> Self {
        self.auto_acknowledge_state_updates = enabled;
        self
    }

    pub fn with_response_delay(mut self, delay_ms: u64) -> Self {
        self.response_delay_ms = delay_ms;
        self
    }

    pub fn with_initial_config(mut self, config: Value) -> Self {
        self.initial_config = Some(config);
        self
    }

    pub fn build(self, event_bus: Arc<dyn EventPublisher>) -> UiSimulator {
        let mut simulator = UiSimulator::new(event_bus);

        // Apply initial config if provided
        if let Some(config) = self.initial_config {
            if let Some(config_obj) = config.as_object() {
                for (key, value) in config_obj {
                    let _ = simulator.config.set_value(key, value.clone());
                }
            }
        }

        simulator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::mocks::MockEventBus;

    #[tokio::test]
    async fn test_basic_simulator() {
        let event_bus = Arc::new(MockEventBus::new());
        let simulator = UiSimulator::new(event_bus.clone());

        // Test timer action
        let start_result = simulator.timer.click_start().await;
        assert!(start_result.is_object());

        // Test task creation
        let task = simulator.task.create_task("Test", None).await;
        assert!(task["id"].is_string());

        // Check events were emitted
        assert!(simulator.app_handle().was_event_emitted("start_timer"));
        assert!(simulator.app_handle().was_event_emitted("create_task"));
    }

    #[tokio::test]
    async fn test_auto_responder() {
        let event_bus = Arc::new(MockEventBus::new());
        let simulator = UiSimulator::new(event_bus.clone());
        let simulator = simulator.start_listen_to_events();

        // Trigger some responses
        simulator.trigger_response(UiResponse::TimerTick { remaining_seconds: 1450 });
        simulator.trigger_response(UiResponse::TaskActiveChanged {
            task_id: "test_task".to_string()
        });

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Test acknowledge ticks
        simulator.acknowledge_ticks(3).await;
    }

    #[tokio::test]
    async fn test_event_responses() {
        let event_bus = Arc::new(MockEventBus::new());
        let simulator = UiSimulator::new(event_bus.clone());
        let simulator = simulator.start_listen_to_events();

        // Test various event responses
        simulator.respond_to_event(timer_events::TICK, json!({
            "remaining_seconds": 1200
        }));

        simulator.respond_to_event(task_events::LIST_UPDATED, json!({
            "tasks": [
                {"id": "1", "title": "Task 1"},
                {"id": "2", "title": "Task 2"}
            ]
        }));

        simulator.respond_to_event(config_events::SETTINGS_UPDATED, json!({
            "work_duration": 30,
            "short_break_duration": 10
        }));

        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}