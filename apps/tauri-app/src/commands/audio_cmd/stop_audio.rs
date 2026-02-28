use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn stop_audio(
    handle_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service
        .stop_audio(&handle_id)
        .with_context(|| {
            format!("Failed to stop audio playback: {}", handle_id)
        })
        .map_err(|e| e.to_string())
}
