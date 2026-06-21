use super::*;
use log::{debug, info};
use usecases::task::{CreateTaskCmd, create_task as create_task_usecase};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub tags: Vec<String>,
    pub work_duration: Option<u64>,
    pub short_break_duration: Option<u64>,
    pub long_break_duration: Option<u64>,
    pub sessions_until_long_break: Option<u8>,
    pub audio_config: Option<AudioConfig>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_task(
    request: CreateTaskRequest,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Task, String> {
    debug!(
        "Creating task: name='{}', sessions={}, tags={:?}",
        request.name, request.max_sessions, request.tags
    );

    // Build the full Config if any timer configuration fields are provided
    let config = if request.work_duration.is_some()
        || request.short_break_duration.is_some()
        || request.long_break_duration.is_some()
        || request.sessions_until_long_break.is_some()
        || request.audio_config.is_some()
    {
        // Get the default config from the repository
        let mut default_config = config_repo
            .get_config()
            .await
            .map_err(|e| format!("Failed to get default config: {}", e))?;

        // Override individual timer fields if provided
        if let Some(work_duration) = request.work_duration {
            default_config.timer = default_config
                .timer
                .with_work_duration(Duration::from_secs(work_duration))
                .map_err(|e| format!("Invalid work duration: {}", e))?;
        }
        if let Some(short_break_duration) = request.short_break_duration {
            default_config.timer = default_config
                .timer
                .with_short_break_duration(Duration::from_secs(
                    short_break_duration,
                ))
                .map_err(|e| format!("Invalid short break duration: {}", e))?;
        }
        if let Some(long_break_duration) = request.long_break_duration {
            default_config.timer = default_config
                .timer
                .with_long_break_duration(Duration::from_secs(
                    long_break_duration,
                ))
                .map_err(|e| format!("Invalid long break duration: {}", e))?;
        }
        if let Some(sessions_until_long_break) =
            request.sessions_until_long_break
        {
            default_config.timer = default_config
                .timer
                .with_sessions_until_long_break(sessions_until_long_break)
                .map_err(|e| {
                    format!("Invalid sessions until long break: {}", e)
                })?;
        }
        if let Some(audio_config) = request.audio_config {
            default_config.audio = audio_config;
        }

        Some(default_config)
    } else {
        None
    };

    let cmd = CreateTaskCmd {
        name: request.name.clone(),
        description: request.description,
        max_sessions: request.max_sessions,
        tags: request.tags,
        config,
    };

    match create_task_usecase(
        task_repo.inner().clone(),
        config_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    {
        Ok(task) => {
            info!("Created task: id={}, name='{}'", task.id(), task.name());
            Ok(task)
        }
        Err(e) => {
            log::error!("Failed to create task '{}': {}", request.name, e);
            Err(format!("Failed to create task '{}': {}", request.name, e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reproduces the exact JSON payload sent by the React UI's TaskFormModal
    // when "Custom timer settings" is enabled: durations as plain integer
    // seconds (not the serde-default {secs, nanos} object).
    #[test]
    fn create_task_request_deserializes_plain_second_durations() {
        let json = r#"{
            "name": "Custom pomodoro",
            "max_sessions": 4,
            "tags": ["work"],
            "work_duration": 1500,
            "short_break_duration": 300,
            "long_break_duration": 900,
            "sessions_until_long_break": 4
        }"#;

        let req: CreateTaskRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Custom pomodoro");
        assert_eq!(req.work_duration, Some(1500));
        assert_eq!(req.short_break_duration, Some(300));
        assert_eq!(req.long_break_duration, Some(900));
        assert_eq!(req.sessions_until_long_break, Some(4));

        // The command body converts these via Duration::from_secs before
        // passing to the domain builder methods.
        let work = req.work_duration.map(Duration::from_secs).unwrap();
        assert_eq!(work, Duration::from_secs(25 * 60));
    }
}
