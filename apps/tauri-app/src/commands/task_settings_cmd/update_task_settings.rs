use super::*;

#[command]
pub async fn update_task_settings(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
    cmd: UpdateTaskSettingsCmd,
) -> Result<TaskSettingsResponse, String> {
    let task_id = TaskId::from_string(&cmd.task_id)
        .map_err(|e| format!("Invalid task ID: {}", e))?;

    let updated_task = usecases::task::update_task_settings(
        &task_repo,
        &event_publisher,
        task_id,
        cmd.settings,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(TaskSettingsResponse {
        task_id: updated_task.id().to_string(),
        settings: Some(updated_task.config().clone()),
    })
}
