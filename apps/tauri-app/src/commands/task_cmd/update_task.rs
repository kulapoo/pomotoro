use super::*;
use usecases::task::{UpdateTaskCmd, update_task as update_task_usecase};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_sessions: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub work_duration: Option<u64>,
    pub short_break_duration: Option<u64>,
    pub long_break_duration: Option<u64>,
    pub sessions_until_long_break: Option<u8>,
    pub audio_config: Option<AudioConfig>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_task(
    request: UpdateTaskRequest,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Task, String> {
    let task_id = TaskId::from_string(&request.id)
        .map_err(|_| format!("Invalid task ID: {}", request.id))?;

    let cmd = UpdateTaskCmd {
        id: task_id,
        name: request.name,
        description: request.description,
        max_sessions: request.max_sessions,
        tags: request.tags,
        work_duration: request.work_duration.map(Duration::from_secs),
        short_break_duration: request
            .short_break_duration
            .map(Duration::from_secs),
        long_break_duration: request
            .long_break_duration
            .map(Duration::from_secs),
        sessions_until_long_break: request.sessions_until_long_break,
        audio_config: request.audio_config,
    };

    let task = update_task_usecase(
        task_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .map_err(|e| {
        let error_msg =
            format!("Failed to update task with id {}: {:?}", request.id, e);
        log::error!("{}", error_msg);
        error_msg
    })?;
    Ok(task)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reproduces the exact JSON payload sent by the React UI's TaskFormModal
    // when "Custom timer settings" is enabled: durations as plain integer
    // seconds (not the serde-default {secs, nanos} object).
    #[test]
    fn update_task_request_deserializes_plain_second_durations() {
        let json = r#"{
            "id": "00000000-0000-0000-0000-000000000001",
            "work_duration": 1500,
            "short_break_duration": 300,
            "long_break_duration": 900,
            "sessions_until_long_break": 4
        }"#;

        let req: UpdateTaskRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.id, "00000000-0000-0000-0000-000000000001");
        assert_eq!(req.work_duration, Some(1500));
        assert_eq!(req.short_break_duration, Some(300));
        assert_eq!(req.long_break_duration, Some(900));
        assert_eq!(req.sessions_until_long_break, Some(4));

        // The command body converts these to Duration via map(Duration::from_secs).
        let work = req.work_duration.map(Duration::from_secs).unwrap();
        assert_eq!(work, Duration::from_secs(25 * 60));
    }

    #[test]
    fn update_task_request_deserializes_without_optional_durations() {
        // When "Custom timer settings" is off, the React UI omits these fields.
        let json = r#"{ "id": "00000000-0000-0000-0000-000000000002" }"#;
        let req: UpdateTaskRequest = serde_json::from_str(json).unwrap();
        assert!(req.work_duration.is_none());
        assert!(req.short_break_duration.is_none());
        assert!(req.long_break_duration.is_none());
        assert!(req.sessions_until_long_break.is_none());
    }
}
