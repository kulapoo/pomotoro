use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn resume_audio(
    handle_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service
        .resume_audio(&handle_id)
        .with_context(|| {
            format!("Failed to resume audio playback: {}", handle_id)
        })
        .map_err(|e| e.to_string())
}
