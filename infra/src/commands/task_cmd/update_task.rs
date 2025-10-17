use super::*;
use usecases::task::{update_task as update_task_usecase, UpdateTaskCmd};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_sessions: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub work_duration: Option<Duration>,
    pub short_break_duration: Option<Duration>,
    pub long_break_duration: Option<Duration>,
    pub sessions_until_long_break: Option<u8>,
    pub enable_screen_blocking: Option<bool>,
    pub audio_config: Option<AudioConfig>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_task(
    request: UpdateTaskRequest,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<TaskDto, String> {
    let task_id = TaskId::from_string(&request.id)
        .map_err(|_| format!("Invalid task ID: {}", request.id))?;

    let cmd = UpdateTaskCmd {
        id: task_id,
        name: request.name,
        description: request.description,
        max_sessions: request.max_sessions,
        tags: request.tags,
        work_duration: request.work_duration,
        short_break_duration: request.short_break_duration,
        long_break_duration: request.long_break_duration,
        sessions_until_long_break: request.sessions_until_long_break,
        enable_screen_blocking: request.enable_screen_blocking,
        audio_config: request.audio_config,
    };

    let task = update_task_usecase(
        task_repo.inner().clone(),
        event_publisher.inner().clone(),
        cmd,
    )
    .await
    .map_err(|e| {
        let error_msg = format!("Failed to update task with id {}: {:?}", request.id, e);
        log::error!("{}", error_msg);
        error_msg
    })?;
    Ok(TaskDto::from(task))
}