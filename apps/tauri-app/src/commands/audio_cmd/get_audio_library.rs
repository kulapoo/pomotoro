use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_audio_library(
    audio_service: AudioServiceState<'_>,
) -> Result<AudioLibrary, String> {
    Ok(audio_service.get_library())
}
