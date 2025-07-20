use super::AudioService;
use pomotoro_domain::{AudioAsset, AudioLibrary, PlaybackRequest, PlaybackHandle};
use tauri::State;
use std::sync::Mutex;

type AudioServiceState<'a> = State<'a, Mutex<AudioService>>;

#[tauri::command]
pub async fn get_audio_library(
    audio_service: AudioServiceState<'_>,
) -> Result<AudioLibrary, String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    Ok(service.get_library().clone())
}

#[tauri::command]
pub async fn play_audio(
    request: PlaybackRequest,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    service.play(request).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_audio(
    handle_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    service.stop_playback(&handle_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_audio(
    handle_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    service.pause_playback(&handle_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_audio(
    handle_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    service.resume_playback(&handle_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_audio_volume(
    handle_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    service.set_volume(&handle_id, volume).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_playbacks(
    audio_service: AudioServiceState<'_>,
) -> Result<Vec<PlaybackHandle>, String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    Ok(service.get_active_playbacks())
}

#[tauri::command]
pub async fn stop_all_audio(
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    service.stop_all_playbacks();
    Ok(())
}

#[tauri::command]
pub async fn play_notification_sound(
    asset_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    service.play_notification(&asset_id, volume).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn play_background_audio(
    asset_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    service.play_background_audio(&asset_id, volume).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_background_audio(
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    service.stop_background_audio();
    Ok(())
}

#[tauri::command]
pub async fn add_custom_audio_asset(
    asset: AudioAsset,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let mut service = audio_service.lock().map_err(|e| e.to_string())?;
    service.add_asset(asset);
    Ok(())
}

#[tauri::command]
pub async fn remove_audio_asset(
    asset_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<Option<AudioAsset>, String> {
    let mut service = audio_service.lock().map_err(|e| e.to_string())?;
    Ok(service.remove_asset(&asset_id))
}

#[tauri::command]
pub async fn cleanup_finished_audio(
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;
    service.cleanup_finished_playbacks();
    Ok(())
}

#[tauri::command]
pub async fn test_audio_preview(
    asset_id: String,
    volume: f32,
    audio_service: AudioServiceState<'_>,
) -> Result<PlaybackHandle, String> {
    let service = audio_service.lock().map_err(|e| e.to_string())?;

    let request = PlaybackRequest::new(asset_id, volume)
        .map_err(|e| e.to_string())?
        .with_fade_in(500);

    service.play(request).map_err(|e| e.to_string())
}