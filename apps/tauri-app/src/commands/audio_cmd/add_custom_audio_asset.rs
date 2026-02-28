use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn add_custom_audio_asset(
    asset: AudioAsset,
    audio_service: AudioServiceState<'_>,
) -> Result<(), String> {
    audio_service.add_asset(asset);
    Ok(())
}
