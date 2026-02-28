use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn cleanup_finished_audio(
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service
        .cleanup_finished()
        .context("Failed to cleanup finished audio")
        .map_err(|e| e.to_string())
}
