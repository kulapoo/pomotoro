use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn stop_background_audio(audio_service: AudioServiceState<'_>) -> Result<(), String> {
    audio_service
        .stop_background_audio()
        .context("Failed to stop background audio")
        .map_err(|e| e.to_string())
}