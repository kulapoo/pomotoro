use super::*;
use crate::commands::task_cmd::create_task::{create_task, CreateTaskRequest};

#[tauri::command(rename_all = "snake_case")]
pub async fn debug_create_test_task(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<TaskDto, String> {
    println!("=== DEBUG TEST TASK CREATION ===");

    let request = CreateTaskRequest {
        name: "Debug Test Task".to_string(),
        description: Some("A test task for debugging".to_string()),
        max_sessions: 3,
        tags: vec!["debug".to_string(), "test".to_string()],
        work_duration: None,
        short_break_duration: None,
        long_break_duration: None,
        sessions_until_long_break: None,
        enable_screen_blocking: None,
        audio_config: None,
    };

    create_task(request, task_repo, config_repo, event_publisher).await
}