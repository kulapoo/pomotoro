use domain::{TaskId, TaskSettings};
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use std::sync::Arc;

use crate::adapters::{
    events::mem_event_bus::EventPublisherArc,
    TaskRepositoryArc,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskSettingsCmd {
    pub task_id: String,
    pub settings: TaskSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSettingsResponse {
    pub task_id: String,
    pub settings: Option<TaskSettings>,
    pub has_custom_settings: bool,
}

#[command]
pub async fn update_task_settings(
    task_repo: State<'_, TaskRepositoryArc>,
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
        settings: Some(updated_task.settings.clone()),
        has_custom_settings: updated_task.has_custom_settings(),
    })
}

#[command]
pub async fn reset_task_settings_to_defaults(
    task_repo: State<'_, TaskRepositoryArc>,
    event_publisher: State<'_, EventPublisherArc>,
    task_id: String,
) -> Result<TaskSettingsResponse, String> {
    let task_id = TaskId::from_string(&task_id)
        .map_err(|e| format!("Invalid task ID: {}", e))?;
    
    let updated_task = usecases::task::reset_task_settings_to_defaults(
        &task_repo,
        &event_publisher,
        task_id,
    )
    .await
    .map_err(|e| e.to_string())?;
    
    Ok(TaskSettingsResponse {
        task_id: updated_task.id().to_string(),
        settings: Some(updated_task.settings.clone()),
        has_custom_settings: false,
    })
}

#[command]
pub async fn get_task_effective_settings(
    task_repo: State<'_, TaskRepositoryArc>,
    _config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    task_id: String,
) -> Result<domain::EffectiveSettings, String> {
    let task_id = TaskId::from_string(&task_id)
        .map_err(|e| format!("Invalid task ID: {}", e))?;
    
    usecases::task::get_effective_task_settings(
        &*task_repo,
        task_id,
    )
    .await
    .map_err(|e| e.to_string())
}