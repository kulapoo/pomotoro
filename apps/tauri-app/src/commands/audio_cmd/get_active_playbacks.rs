use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_active_playbacks(
    audio_service: AudioServiceState<'_>,
) -> Result<Vec<PlaybackHandle>, String> {
    audio_service
        .get_active_playbacks()
        .context("Failed to get active playbacks")
        .map_err(|e| e.to_string())
}
