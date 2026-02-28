use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn test_notification(
    event_publisher: State<'_, Arc<dyn EventPublisher + Send + Sync>>,
    notification_type: String,
) -> Result<(), String> {
    let event: Box<dyn Event> = match notification_type.as_str() {
        "work_phase_completed" => Box::new(WorkPhaseCompleted::new(
            TaskId::new(),
            1500,
            1,
        )),
        "task_completed" => Box::new(TaskCompleted::new(TaskId::new(), 10, 1)),
        "timer_started" => {
            // Use default configuration values
            let default_config = TimerConfiguration::default();
            let work_duration_seconds = default_config.get_phase_duration_seconds(Phase::Work);
            Box::new(TimerStarted::new(
                TaskId::new(),
                Phase::Work,
                work_duration_seconds,
                1,
            ))
        }
        "timer_paused" => {
            // Use half of default work duration for pause test
            let default_config = TimerConfiguration::default();
            let work_duration_seconds = default_config.get_phase_duration_seconds(Phase::Work);
            Box::new(TimerPaused::new(
                TaskId::new(),
                Phase::Work,
                work_duration_seconds / 2,
                1,
                TimerConfiguration::default(),
            ))
        }
        _ => {
            return Err(format!("Unknown notification type: {}", notification_type));
        }
    };

    event_publisher.publish(event);
    Ok(())
}