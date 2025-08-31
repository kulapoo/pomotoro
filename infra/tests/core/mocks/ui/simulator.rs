use std::{sync::{Arc, Mutex}};
use tokio::sync::mpsc;
use serde_json::{Value};
use domain::event_names::ui_listeners::{timer as timer_events, task as task_events, config as config_events, app as app_events};
use domain::event_names::commands::{
    timer as timer_commands,
    task as task_commands,
    config as config_commands,
};

use super::{
    app_handle::MockAppHandle,
};

fn register_events(app_handle: MockAppHandle, tx: mpsc::UnboundedSender<Value>) {
    let ui_events = vec![
        timer_events::TICK,
        timer_events::STATUS_CHANGED,
        timer_events::PHASE_EVENT,
        timer_events::PHASE_COMPLETED,
        timer_events::PHASE_SKIPPED,
        timer_events::STATE_UPDATED,
        task_events::LIST_UPDATED,
        task_events::ACTIVE_CHANGED,
        task_events::PROGRESS_UPDATED,
        config_events::SETTINGS_UPDATED,
        config_events::THEME_CHANGED,
        app_events::APP_STARTED,
        app_events::APP_EXITED,
    ];

    let command_events = vec![
        timer_commands::START,
        timer_commands::PAUSE,
        timer_commands::RESET,
        timer_commands::SKIP_PHASE,
        timer_commands::GET_STATE,
        task_commands::CREATE,
        task_commands::UPDATE,
        task_commands::DELETE,
        task_commands::GET,
        task_commands::GET_ALL,
        task_commands::GET_ACTIVE,
        task_commands::GET_BY_TAGS,
        task_commands::COMPLETE_SESSION,
        task_commands::RESET_SESSIONS,
        task_commands::SEARCH,
        task_commands::SEARCH_FUZZY,
        task_commands::FILTER_BY_STATUS,
        task_commands::CYCLE_INCOMPLETE_TASK,
        task_commands::GET_TASK_CYCLE_POSITION,
        task_commands::GET_INCOMPLETE_TASKS,
        config_commands::GET_GLOBAL,
        config_commands::SAVE_GLOBAL,
        config_commands::UPDATE_GENERAL,
        config_commands::UPDATE_NOTIFICATIONS,
        config_commands::UPDATE_APPEARANCE,
        config_commands::UPDATE_AUDIO,
        config_commands::UPDATE_TIMINGS,
        config_commands::RESET_TO_DEFAULTS,
        config_commands::GET_EFFECTIVE_AUDIO,
        config_commands::CONFIG_UPDATED,
        config_commands::CONFIG_RESET
    ];

    let events = [ui_events, command_events].concat();

    for event in events {
        let tx_clone = tx.clone();
        let event_name = event.to_string();
        app_handle.listen(event, move |mut payload| {
            if let Value::Object(ref mut map) = payload {
                map.insert("event_name".to_string(), Value::String(event_name.clone()));
            }
            let _ = tx_clone.send(payload);
        });
    }

}

/// Simple UI Simulator for integration testing
///
/// This simulator provides:
/// - UI action modules (timer, task, config, audio) to simulate user interactions
/// - Event tracking via MockAppHandle
/// - Simple response handling for testing event flow
#[derive(Clone)]
pub struct UiSimulator {
    app_handle: MockAppHandle,
    response_tx: mpsc::UnboundedSender<Value>,
    response_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<Value>>>>,
}

impl UiSimulator {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let app_handle = MockAppHandle::new();

        register_events(app_handle.clone(), tx.clone());

        Self {
            app_handle,
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
                println!("Event listener started");
                while let Some(response) = rx.recv().await {
                    println!("Event: {:#?}", response);
                }
            });
        }

        self
    }

    /// Trigger a UI response for testing
    pub fn trigger_response(&self, response: Value) {
        let _ = self.response_tx.send(response);
    }

    /// Respond to a domain event
    pub fn respond_to_event(&self, event_type: &str, payload: Value) {

        // self.trigger_response(response);
    }
}