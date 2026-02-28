use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn stop_all_audio(audio_service: AudioServiceState<'_>) -> Result<(), String> {
    audio_service
        .stop_all_audio()
        .context("Failed to stop all audio")
        .map_err(|e| e.to_string())
}