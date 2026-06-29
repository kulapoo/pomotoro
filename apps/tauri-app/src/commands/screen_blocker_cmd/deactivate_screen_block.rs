use super::{AppHandle, Manager};

/// Restore the main window: exit fullscreen and clear always-on-top.
///
/// Invoked by the frontend when the user dismisses the screen blocker
/// overlay (ESC or the dismiss button).
///
/// After restoring window geometry, also restores the prior window
/// visibility: if [`activate_screen_block`](super::activate_screen_block) had
/// to surface a hidden window, this hides it again (and re-syncs the tray
/// Show/Hide label). The hide runs *after* exiting fullscreen so we never
/// try to hide a window that is still in fullscreen, which can misbehave on
/// some window managers.
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

    crate::tray::restore_visibility_after_screen_block(&app_handle);

    Ok(())
}
