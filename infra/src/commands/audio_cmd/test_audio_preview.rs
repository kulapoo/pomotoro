use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn test_audio_preview(
    asset_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    let request = PlaybackRequest::new(asset_id.clone(), volume)
        .with_context(|| format!("Failed to create playback request for asset: {}", asset_id))
        .map_err(|e| e.to_string())?
        .with_fade_in(500);

    audio_service
        .play_audio(request)
        .context("Failed to play audio preview")
        .map_err(|e| e.to_string())
}