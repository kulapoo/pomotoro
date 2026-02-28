use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn set_audio_volume(
    handle_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service
        .set_volume(&handle_id, volume)
        .with_context(|| {
            format!(
                "Failed to set volume to {} for playback: {}",
                volume, handle_id
            )
        })
        .map_err(|e| e.to_string())
}
