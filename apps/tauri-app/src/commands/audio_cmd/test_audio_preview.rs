use super::*;
use std::time::Duration;

/// Maximum duration a preview playback is allowed to run before being
/// automatically stopped. Keeps the Test button from leaving a sound
/// (especially long ambient files) playing indefinitely.
const PREVIEW_MAX_DURATION_MS: u64 = 3000;

#[tauri::command(rename_all = "snake_case")]
pub async fn test_audio_preview(
    asset_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    let request = PlaybackRequest::new(asset_id.clone(), volume)
        .with_context(|| {
            format!("Failed to create playback request for asset: {}", asset_id)
        })
        .map_err(|e| e.to_string())?
        .with_fade_in(500);

    let handle = audio_service
        .play_audio(request)
        .context("Failed to play audio preview")
        .map_err(|e| e.to_string())?;

    // Auto-stop after the preview time limit so playback can't run forever.
    let playback_id = handle.id.clone();
    let service = audio_service.inner().clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(PREVIEW_MAX_DURATION_MS))
            .await;
        if let Err(e) = service.stop_audio(&playback_id) {
            eprintln!("Failed to stop audio preview: {:?}", e);
        }
    });

    Ok(handle)
}
