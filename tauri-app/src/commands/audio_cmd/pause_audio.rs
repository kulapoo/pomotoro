use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn pause_audio(
    handle_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service
        .pause_audio(&handle_id)
        .with_context(|| format!("Failed to pause audio playback: {}", handle_id))
        .map_err(|e| e.to_string())
}