#[tauri::command(rename_all = "snake_case")]
pub async fn request_notification_permission() -> Result<bool, String> {
    Ok(true)
}
