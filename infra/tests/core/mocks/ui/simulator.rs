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

#[derive(Clone)]
/// Comprehensive UI Simulator for integration testing
pub struct UiSimulator {
    app_handle: MockAppHandle,
    event_bus: Arc<dyn EventPublisher>,
    response_channel: mpsc::UnboundedSender<UiResponse>,
    response_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<UiResponse>>>>,

    // Configuration
    auto_acknowledge_ticks: bool,
    auto_acknowledge_state_updates: bool,
    response_delay_ms: u64,
    initial_config: Option<Value>,

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

        // Setup event listeners for UI events
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
            response_channel: tx,
            response_receiver: Arc::new(Mutex::new(Some(rx))),
            auto_acknowledge_ticks: true,
            auto_acknowledge_state_updates: true,
            response_delay_ms: 10,
            initial_config: None,
        }
    }

    /// Set auto acknowledge ticks configuration
    pub fn set_auto_acknowledge_ticks(&mut self, enabled: bool) {
        self.auto_acknowledge_ticks = enabled;
    }

    /// Set auto acknowledge state updates configuration
    pub fn set_auto_acknowledge_state_updates(&mut self, enabled: bool) {
        self.auto_acknowledge_state_updates = enabled;
    }

    /// Set response delay in milliseconds
    pub fn set_response_delay_ms(&mut self, delay_ms: u64) {
        self.response_delay_ms = delay_ms;
    }

    /// Apply initial configuration
    pub fn apply_initial_config(&mut self, config: Value) {
        self.initial_config = Some(config.clone());
        // Apply configuration to the config actions
        if let Some(config_obj) = config.as_object() {
            for (key, value) in config_obj {
                let _ = self.config.set_value(key, value.clone());
            }
        }
    }

    /// Get the mock app handle for direct access
    pub fn app_handle(&self) -> &MockAppHandle {
        &self.app_handle
    }

    /// Start the UI simulator as an auto-responder
    pub fn start_auto_responder(mut self) -> UiSimulatorHandle {
        // Take the receiver from the Arc<Mutex<Option>>
        let mut receiver = self.response_receiver.lock().unwrap().take()
            .expect("start_auto_responder can only be called once");

        let handle = UiSimulatorHandle::new(
            self.response_channel.clone(),
            self.app_handle.clone(),
        );
        let response_delay_ms = self.response_delay_ms;
        let auto_acknowledge_ticks = self.auto_acknowledge_ticks;
        let auto_acknowledge_state_updates = self.auto_acknowledge_state_updates;
        println!("Starting auto responder");
        tokio::spawn(async move {
            while let Some(response) = receiver.recv().await {
                // Simulate UI processing delay using configured value
                tokio::time::sleep(Duration::from_millis(response_delay_ms)).await;
                println!("Received response: {:?}", response);
                match response {
                    UiResponse::TimerTick { remaining_seconds } => {
                        // UI updates timer display
                        tracing::trace!("UI: Timer tick - {} seconds remaining", remaining_seconds);
                        if auto_acknowledge_ticks {
                            // Auto-acknowledge the tick was received
                            tracing::trace!("UI: Auto-acknowledging timer tick");
                        }
                    }
                    UiResponse::TimerStatusChanged { status } => {
                        // UI updates status indicator
                        tracing::trace!("UI: Timer status changed to {}", status);
                    }
                    UiResponse::TimerPhaseEvent { phase } => {
                        // UI handles phase transition
                        tracing::trace!("UI: Phase changed to {}", phase);
                    }
                    UiResponse::TimerStateUpdated { state } => {
                        // UI refreshes entire timer state
                        tracing::trace!("UI: Timer state updated: {:?}", state);
                        if auto_acknowledge_state_updates {
                            // Auto-acknowledge the state update was received
                            tracing::trace!("UI: Auto-acknowledging state update");
                        }
                    }
                    UiResponse::TaskListUpdated { tasks } => {
                        // UI refreshes task list
                        tracing::trace!("UI: Task list updated with {} tasks", tasks.len());
                    }
                    UiResponse::TaskActiveChanged { task_id } => {
                        // UI highlights active task
                        tracing::trace!("UI: Active task changed to {}", task_id);
                    }
                    UiResponse::TaskProgressUpdated { task_id, progress } => {
                        // UI updates progress bar
                        tracing::trace!("UI: Task {} progress: {}%", task_id, progress * 100.0);
                    }
                    UiResponse::ConfigSettingsUpdated { settings } => {
                        // UI applies new settings
                        tracing::trace!("UI: Settings updated: {:?}", settings);
                    }
                    UiResponse::ConfigThemeChanged { theme } => {
                        // UI applies theme
                        tracing::trace!("UI: Theme changed to {}", theme);
                    }
                    UiResponse::AppStarted => {
                        // UI initialization complete
                        println!("UI: App started");
                        tracing::trace!("UI: App started");
                    }
                    UiResponse::AppExited => {
                        // UI cleanup
                        tracing::trace!("UI: App exited");
                    }
                    UiResponse::CommandAcknowledged { command, result } => {
                        // UI acknowledges command execution
                        tracing::trace!("UI: Command '{}' acknowledged: {:?}", command, result);
                    }
                    UiResponse::EventReceived { event_type, payload } => {
                        // UI received event from backend
                        tracing::trace!("UI: Received event '{}': {:?}", event_type, payload);
                    }
                }
            }
        });

        handle
    }

    /// Simulate a complete timer workflow
    pub async fn simulate_pomodoro_session(&self) -> Vec<Value> {
        let mut responses = Vec::new();

        // Start timer
        responses.push(self.timer.click_start().await);
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Simulate some ticks
        for i in 0..5 {
            self.response_channel.send(UiResponse::TimerTick {
                remaining_seconds: 1500 - i * 30,
            }).unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Complete the phase
        responses.push(json!({
            "phase_completed": true,
            "next_phase": "ShortBreak"
        }));

        responses
    }

    /// Simulate task management workflow
    pub async fn simulate_task_workflow(&self) -> Vec<Value> {
        let mut responses = Vec::new();

        // Create a task
        let task = self.task.create_task("Test Task", Some("Description")).await;
        let task_id = task["id"].as_str().unwrap().to_string();
        responses.push(task);

        // Switch to the task
        responses.push(self.timer.switch_task(&task_id).await);

        // Start timer with task
        responses.push(self.timer.click_start().await);

        // Complete a session
        responses.push(self.task.complete_session(&task_id).await);

        responses
    }
}

/// Handle for controlling the UI simulator after it's started
pub struct UiSimulatorHandle {
    response_channel: mpsc::UnboundedSender<UiResponse>,
    app_handle: MockAppHandle,
}

impl UiSimulatorHandle {
    pub fn new(response_channel: mpsc::UnboundedSender<UiResponse>, app_handle: MockAppHandle) -> Self {
        Self {
            response_channel,
            app_handle,
        }
    }

    /// Trigger a UI response manually
    pub fn trigger_response(&self, response: UiResponse) {
        let _ = self.response_channel.send(response);
    }

    /// Get the mock app handle
    pub fn app_handle(&self) -> &MockAppHandle {
        &self.app_handle
    }

    /// Simulate the UI acknowledging multiple timer ticks
    pub async fn acknowledge_ticks(&self, count: usize) {
        for i in 0..count {
            self.trigger_response(UiResponse::TimerTick {
                remaining_seconds: 1500 - (i as u32 * 30),
            });
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Simulate UI responding to a specific domain event
    pub fn respond_to_event(&self, event_type: &str, payload: Value) {
        let response = match event_type {
            // Timer events
            timer_events::TICK => UiResponse::TimerTick {
                remaining_seconds: payload["remaining_seconds"].as_u64().unwrap_or(0) as u32,
            },
            timer_events::STATUS_CHANGED => UiResponse::TimerStatusChanged {
                status: payload["status"].as_str().unwrap_or("unknown").to_string(),
            },
            timer_events::PHASE_EVENT => UiResponse::TimerPhaseEvent {
                phase: payload["phase"].as_str().unwrap_or("Work").to_string(),
            },
            timer_events::STATE_UPDATED => UiResponse::TimerStateUpdated {
                state: payload,
            },

            // Task events
            task_events::LIST_UPDATED => UiResponse::TaskListUpdated {
                tasks: payload["tasks"].as_array()
                    .map(|v| v.clone())
                    .unwrap_or_default(),
            },
            task_events::ACTIVE_CHANGED => UiResponse::TaskActiveChanged {
                task_id: payload["task_id"].as_str().unwrap_or("").to_string(),
            },
            task_events::PROGRESS_UPDATED => UiResponse::TaskProgressUpdated {
                task_id: payload["task_id"].as_str().unwrap_or("").to_string(),
                progress: payload["progress"].as_f64().unwrap_or(0.0) as f32,
            },

            // Config events
            config_events::SETTINGS_UPDATED => UiResponse::ConfigSettingsUpdated {
                settings: payload,
            },
            config_events::THEME_CHANGED => UiResponse::ConfigThemeChanged {
                theme: payload["theme"].as_str().unwrap_or("dark").to_string(),
            },

            // App events
            app_events::APP_STARTED => UiResponse::AppStarted,
            app_events::APP_EXITED => UiResponse::AppExited,

            // Commands
            _ => UiResponse::CommandAcknowledged {
                command: event_type.to_string(),
                result: payload,
            },
        };

        self.trigger_response(response);
    }

    /// Simulate a UI disconnect (stop responding)
    pub fn simulate_disconnect(&self) {
        // Drop the channel to simulate disconnect
        // In a real scenario, we might want to keep the channel but stop processing
    }

    /// Check if an event was emitted
    pub fn was_event_emitted(&self, event_type: &str) -> bool {
        self.app_handle.was_event_emitted(event_type)
    }

    /// Get the last emitted event of a type
    pub fn last_event_of_type(&self, event_type: &str) -> Option<Value> {
        self.app_handle.events_of_type(event_type)
            .last()
            .map(|e| e.payload.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::mocks::MockEventBus;

    #[tokio::test]
    async fn test_complete_workflow() {
        let event_bus = Arc::new(MockEventBus::new());
        let simulator = UiSimulator::new(event_bus.clone());

        // Run a complete pomodoro session simulation
        let responses = simulator.simulate_pomodoro_session().await;
        assert!(!responses.is_empty());

        // Run a task workflow
        let task_responses = simulator.simulate_task_workflow().await;
        assert!(!task_responses.is_empty());
    }

    #[tokio::test]
    async fn test_auto_responder() {
        let event_bus = Arc::new(MockEventBus::new());
        let simulator = UiSimulator::new(event_bus.clone());
        let handle = simulator.start_auto_responder();

        // Trigger some responses
        handle.trigger_response(UiResponse::TimerTick { remaining_seconds: 1450 });
        handle.trigger_response(UiResponse::TaskActiveChanged {
            task_id: "test_task".to_string()
        });

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Verify responses were processed (check logs in real scenario)
        handle.acknowledge_ticks(3).await;
    }

    #[tokio::test]
    async fn test_event_responses() {
        let event_bus = Arc::new(MockEventBus::new());
        let simulator = UiSimulator::new(event_bus.clone());
        let handle = simulator.start_auto_responder();

        // Test timer tick response
        handle.respond_to_event(timer_events::TICK, json!({
            "remaining_seconds": 1200
        }));

        // Test task list update
        handle.respond_to_event(task_events::LIST_UPDATED, json!({
            "tasks": [
                {"id": "1", "title": "Task 1"},
                {"id": "2", "title": "Task 2"}
            ]
        }));

        // Test config update
        handle.respond_to_event(config_events::SETTINGS_UPDATED, json!({
            "work_duration": 30,
            "short_break_duration": 10
        }));

        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}