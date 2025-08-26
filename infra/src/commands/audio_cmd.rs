use crate::adapters::RodioAudioService;
use domain::{
    AudioAsset, AudioLibrary, AudioService, PlaybackHandle, PlaybackRequest,
};
use std::sync::Mutex;
use tauri::State;
use anyhow::Context;

type AudioServiceState<'a> = State<'a, Mutex<RodioAudioService>>;

#[tauri::command]
pub async fn get_audio_library(
    audio_service: AudioServiceState<'_>,
) -> Result<AudioLibrary, String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    Ok(service.get_library().clone())
}

#[tauri::command]
pub async fn play_audio(
    request: PlaybackRequest,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service.play(request)
        .context("Failed to play audio")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_audio(
    handle_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service.stop_playback(&handle_id)
        .with_context(|| format!("Failed to stop audio playback: {}", handle_id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_audio(
    handle_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service
        .pause_playback(&handle_id)
        .with_context(|| format!("Failed to pause audio playback: {}", handle_id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_audio(
    handle_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service
        .resume_playback(&handle_id)
        .with_context(|| format!("Failed to resume audio playback: {}", handle_id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_audio_volume(
    handle_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let mut service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service
        .set_volume(&handle_id, volume)
        .with_context(|| format!("Failed to set volume to {} for playback: {}", volume, handle_id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_playbacks(
    audio_service: AudioServiceState<'_>,
) -> Result<Vec<PlaybackHandle>, String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    AudioService::get_active_playbacks(&*service)
        .context("Failed to get active playbacks")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_all_audio(
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service.stop_all_playbacks();
    Ok(())
}

#[tauri::command]
pub async fn play_notification_sound(
    asset_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service
        .play_notification(&asset_id, volume)
        .with_context(|| format!("Failed to play notification sound: {}", asset_id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn play_background_audio(
    asset_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service
        .play_background_audio(&asset_id, volume)
        .with_context(|| format!("Failed to play background audio: {}", asset_id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_background_audio(
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service.stop_background_audio();
    Ok(())
}

#[tauri::command]
pub async fn add_custom_audio_asset(
    asset: AudioAsset,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let mut service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service.add_asset(asset);
    Ok(())
}

#[tauri::command]
pub async fn remove_audio_asset(
    asset_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<Option<AudioAsset>, String> {
    let mut service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    Ok(service.remove_asset(&asset_id))
}

#[tauri::command]
pub async fn cleanup_finished_audio(
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;
    service.cleanup_finished_playbacks();
    Ok(())
}

#[tauri::command]
pub async fn test_audio_preview(
    asset_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    let service = audio_service.lock()
        .map_err(|e| format!("Failed to acquire audio service lock: {}", e))?;

    let request = PlaybackRequest::new(asset_id.clone(), volume)
        .with_context(|| format!("Failed to create playback request for asset: {}", asset_id))
        .map_err(|e| e.to_string())?
        .with_fade_in(500);

    service.play(request)
        .context("Failed to play audio preview")
        .map_err(|e| e.to_string())
}
