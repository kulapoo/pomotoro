use domain::{EventPublisher, PhaseCompleted, TaskCompleted, Phase};
use domain::timer::events::{Started as TimerStarted, Paused as TimerPaused};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn test_notification(
    event_publisher: State<'_, Arc<dyn EventPublisher + Send + Sync>>,
    notification_type: String,
) -> Result<(), String> {
    let event: Box<dyn domain::Event> = match notification_type.as_str() {
        "phase_completed" => {
            Box::new(PhaseCompleted::new(
                None,
                Phase::Work,
                Phase::ShortBreak,
                1,
                1,
                1,
            ))
        },
        "task_completed" => {
            Box::new(TaskCompleted::new(
                domain::TaskId::new(),
                10,
                1,
            ))
        },
        "timer_started" => {
            Box::new(TimerStarted::new(
                Some("test-task-id".to_string()),
                Phase::Work,
                1500,
                1,
            ))
        },
        "timer_paused" => {
            Box::new(TimerPaused::new(
                Some("test-task-id".to_string()),
                Phase::Work,
                900,
                1,
            ))
        },
        _ => {
            return Err(format!("Unknown notification type: {}", notification_type));
        }
    };
    
    event_publisher.publish(event);
    Ok(())
}

#[tauri::command]
pub async fn request_notification_permission() -> Result<bool, String> {
    Ok(true)
}