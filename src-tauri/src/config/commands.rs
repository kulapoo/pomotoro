use super::{GlobalConfig, ConfigRepository};
use tauri::State;

#[tauri::command]
pub async fn get_global_config(
    config_repo: State<'_, ConfigRepository>,
) -> Result<GlobalConfig, String> {
    config_repo
        .get_config()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_global_config(
    config: GlobalConfig,
    config_repo: State<'_, ConfigRepository>,
) -> Result<(), String> {
    config_repo
        .save_config(&config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_global_config_to_defaults(
    config_repo: State<'_, ConfigRepository>,
) -> Result<GlobalConfig, String> {
    config_repo
        .reset_to_defaults()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_default_timings(
    work_minutes: u32,
    short_break_minutes: u32,
    long_break_minutes: u32,
    config_repo: State<'_, ConfigRepository>,
) -> Result<GlobalConfig, String> {
    let mut config = config_repo
        .get_config()
        .map_err(|e| e.to_string())?;
    
    config.update_default_timings(work_minutes, short_break_minutes, long_break_minutes);
    
    config_repo
        .save_config(&config)
        .map_err(|e| e.to_string())?;
    
    Ok(config)
}

#[tauri::command]
pub async fn update_default_cycle_length(
    sessions_until_long_break: u8,
    config_repo: State<'_, ConfigRepository>,
) -> Result<GlobalConfig, String> {
    let mut config = config_repo
        .get_config()
        .map_err(|e| e.to_string())?;
    
    config.update_default_cycle_length(sessions_until_long_break);
    
    config_repo
        .save_config(&config)
        .map_err(|e| e.to_string())?;
    
    Ok(config)
}

#[tauri::command]
pub async fn update_app_preferences(
    preferences: super::models::AppPreferences,
    config_repo: State<'_, ConfigRepository>,
) -> Result<GlobalConfig, String> {
    let mut config = config_repo
        .get_config()
        .map_err(|e| e.to_string())?;
    
    config.app_preferences = preferences;
    
    config_repo
        .save_config(&config)
        .map_err(|e| e.to_string())?;
    
    Ok(config)
}

#[tauri::command]
pub async fn update_notification_preferences(
    preferences: super::models::NotificationPreferences,
    config_repo: State<'_, ConfigRepository>,
) -> Result<GlobalConfig, String> {
    let mut config = config_repo
        .get_config()
        .map_err(|e| e.to_string())?;
    
    config.notification_preferences = preferences;
    
    config_repo
        .save_config(&config)
        .map_err(|e| e.to_string())?;
    
    Ok(config)
}

#[tauri::command]
pub async fn update_ui_preferences(
    preferences: super::models::UiPreferences,
    config_repo: State<'_, ConfigRepository>,
) -> Result<GlobalConfig, String> {
    let mut config = config_repo
        .get_config()
        .map_err(|e| e.to_string())?;
    
    config.ui_preferences = preferences;
    
    config_repo
        .save_config(&config)
        .map_err(|e| e.to_string())?;
    
    Ok(config)
}

#[tauri::command]
pub async fn update_default_audio_config(
    audio_config: crate::task::models::AudioConfig,
    config_repo: State<'_, ConfigRepository>,
) -> Result<GlobalConfig, String> {
    let mut config = config_repo
        .get_config()
        .map_err(|e| e.to_string())?;
    
    config.default_audio_config = audio_config;
    
    config_repo
        .save_config(&config)
        .map_err(|e| e.to_string())?;
    
    Ok(config)
}

#[tauri::command]
pub async fn get_effective_task_config(
    task_id: Option<uuid::Uuid>,
    task_repo: State<'_, crate::task::TaskRepository>,
    config_repo: State<'_, ConfigRepository>,
) -> Result<crate::task::models::TaskConfig, String> {
    let global_config = config_repo
        .get_config()
        .map_err(|e| e.to_string())?;
    
    if let Some(task_id) = task_id {
        if let Ok(Some(task)) = task_repo.get_by_id(task_id).await {
            return Ok(task.config);
        }
    }
    
    Ok(global_config.default_task_config)
}

#[tauri::command]
pub async fn get_effective_audio_config(
    task_id: Option<uuid::Uuid>,
    task_repo: State<'_, crate::task::TaskRepository>,
    config_repo: State<'_, ConfigRepository>,
) -> Result<crate::task::models::AudioConfig, String> {
    let global_config = config_repo
        .get_config()
        .map_err(|e| e.to_string())?;
    
    if let Some(task_id) = task_id {
        if let Ok(Some(task)) = task_repo.get_by_id(task_id).await {
            return Ok(task.audio_config);
        }
    }
    
    Ok(global_config.default_audio_config)
}