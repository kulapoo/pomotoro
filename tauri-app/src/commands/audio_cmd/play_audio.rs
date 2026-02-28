use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn play_audio(
    request: PlaybackRequest,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    audio_service
        .play_audio(request)
        .context("Failed to play audio")
        .map_err(|e| e.to_string())
}