use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn play_background_audio(
    asset_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    audio_service
        .play_background_audio(&asset_id, volume)
        .with_context(|| {
            format!("Failed to play background audio: {}", asset_id)
        })
        .map_err(|e| e.to_string())
}
