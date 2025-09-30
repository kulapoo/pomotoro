use crate::adapters::audio::AudioServiceWrapper;
use anyhow::Context;
use domain::{AudioAsset, AudioLibrary, PlaybackHandle, PlaybackRequest};
use tauri::State;

type AudioServiceState<'a> = State<'a, AudioServiceWrapper>;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_audio_library(
    audio_service: AudioServiceState<'_>,
) -> Result<AudioLibrary, String> {
    Ok(audio_service.get_library())
}

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

#[tauri::command(rename_all = "snake_case")]
pub async fn pause_audio(
    handle_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service
        .pause_audio(&handle_id)
        .with_context(|| {
            format!("Failed to pause audio playback: {}", handle_id)
        })
        .map_err(|e| e.to_string())
}

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

#[tauri::command(rename_all = "snake_case")]
pub async fn get_active_playbacks(
    audio_service: AudioServiceState<'_>,
) -> Result<Vec<PlaybackHandle>, String> {
    audio_service
        .get_active_playbacks()
        .context("Failed to get active playbacks")
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn stop_all_audio(
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service
        .stop_all_audio()
        .context("Failed to stop all audio")
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn play_notification_sound(
    asset_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    audio_service
        .play_notification(&asset_id, volume)
        .with_context(|| {
            format!("Failed to play notification sound: {}", asset_id)
        })
        .map_err(|e| e.to_string())
}

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

#[tauri::command(rename_all = "snake_case")]
pub async fn stop_background_audio(
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service
        .stop_background_audio()
        .context("Failed to stop background audio")
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn add_custom_audio_asset(
    asset: AudioAsset,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service.add_asset(asset);
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn remove_audio_asset(
    asset_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<Option<AudioAsset>, String> {
    Ok(audio_service.remove_asset(&asset_id))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn cleanup_finished_audio(
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service
        .cleanup_finished()
        .context("Failed to cleanup finished audio")
        .map_err(|e| e.to_string())
}

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

    audio_service
        .play_audio(request)
        .context("Failed to play audio preview")
        .map_err(|e| e.to_string())
}
