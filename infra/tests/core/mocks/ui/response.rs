use serde_json::Value;

#[derive(Debug, Clone)]
pub enum UiResponse {
    // Timer-related responses
    TimerTick { remaining_seconds: u32 },
    TimerStatusChanged { status: String },
    TimerPhaseEvent { phase: String },
    TimerStateUpdated { state: Value },
    
    // Task-related responses
    TaskListUpdated { tasks: Vec<Value> },
    TaskActiveChanged { task_id: String },
    TaskProgressUpdated { task_id: String, progress: f32 },
    
    // Config-related responses
    ConfigSettingsUpdated { settings: Value },
    ConfigThemeChanged { theme: String },
    
    // App-related responses
    AppStarted,
    AppExited,
    
    // Command acknowledgments with JSON payloads
    CommandAcknowledged { command: String, result: Value },
    
    // Event received from backend
    EventReceived { event_type: String, payload: Value },
}