use super::{AppHandle, Manager};

/// Restore the main window: exit fullscreen and clear always-on-top.
///
/// Invoked by the frontend when the user dismisses the screen blocker
/// overlay (ESC or the dismiss button).
#[tauri::command(rename_all = "snake_case")]
pub async fn deactivate_screen_block(
    app_handle: AppHandle,
) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    window
        .set_always_on_top(false)
        .map_err(|e| format!("Failed to clear always-on-top: {e}"))?;
    window
        .set_fullscreen(false)
        .map_err(|e| format!("Failed to exit fullscreen: {e}"))?;

    Ok(())
}
