use domain::timer::events::{Paused as TimerPaused, Started as TimerStarted};
use domain::{
    EventPublisher, Phase, PhaseCompleted, TaskCompleted, TimerConfiguration,
    TimerId,
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn test_notification(
    event_publisher: State<'_, Arc<dyn EventPublisher + Send + Sync>>,
    notification_type: String,
) -> Result<(), String> {
    let event: Box<dyn domain::Event> = match notification_type.as_str() {
        "phase_completed" => Box::new(PhaseCompleted::new(
            TimerId::new(),
            Phase::Work,
            Phase::ShortBreak,
            1,
            1,
        )),
        "task_completed" => {
            Box::new(TaskCompleted::new(domain::TaskId::new(), 10, 1))
        }
        "timer_started" => {
            // Use default configuration values
            let default_config = TimerConfiguration::default();
            let work_duration_seconds =
                default_config.get_phase_duration_seconds(Phase::Work);
            Box::new(TimerStarted::new(
                TimerId::new(),
                Phase::Work,
                work_duration_seconds,
                1,
            ))
        }
        "timer_paused" => {
            // Use half of default work duration for pause test
            let default_config = TimerConfiguration::default();
            let work_duration_seconds =
                default_config.get_phase_duration_seconds(Phase::Work);
            Box::new(TimerPaused::new(
                TimerId::new(),
                Phase::Work,
                work_duration_seconds / 2,
                1,
                TimerConfiguration::default(),
            ))
        }
        _ => {
            return Err(format!(
                "Unknown notification type: {}",
                notification_type
            ));
        }
    };

    event_publisher.publish(event);
    Ok(())
}

#[tauri::command]
pub async fn request_notification_permission() -> Result<bool, String> {
    Ok(true)
}
