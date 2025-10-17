use super::*;

#[command]
pub async fn reset_task_settings_to_defaults(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
    task_id: String,
) -> Result<TaskSettingsResponse, String> {
    let task_id = TaskId::from_string(&task_id).map_err(|e| format!("Invalid task ID: {}", e))?;

    let updated_task = usecases::task::reset_task_settings_to_defaults(
        &task_repo,
        &event_publisher,
        task_id,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(TaskSettingsResponse {
        task_id: updated_task.id().to_string(),
        settings: Some(updated_task.config.clone()),
    })
}