use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn remove_audio_asset(
    asset_id: String,
    audio_service: AudioServiceState<'_>,
) -> Result<Option<AudioAsset>, String> {
    Ok(audio_service.remove_asset(&asset_id))
}
