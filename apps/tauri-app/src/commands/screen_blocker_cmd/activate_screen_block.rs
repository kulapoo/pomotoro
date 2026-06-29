use super::{AppHandle, Manager};

/// Force the main window into fullscreen + always-on-top.
///
/// Invoked by the frontend when the screen blocker overlay is shown, so the
/// user cannot simply alt-tab away from the focus-enforcement overlay.
///
/// If the window is currently hidden (tray "Show/Hide", close-to-tray, or
/// start-minimized), it is shown + focused first — otherwise the fullscreen
/// overlay would render inside an invisible webview and the user would see
/// nothing. The prior visibility is remembered so
/// [`deactivate_screen_block`](super::deactivate_screen_block) can restore it
/// on dismiss. Showing before `set_fullscreen` also matters for
/// cross-platform correctness: Linux/GTK and macOS can silently drop a
/// `set_fullscreen` call on an unrealized (hidden) window.
#[tauri::command(rename_all = "snake_case")]
pub async fn activate_screen_block(
    app_handle: AppHandle,
) -> Result<(), String> {
    crate::tray::ensure_visible_for_screen_block(&app_handle);

    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    window
        .set_fullscreen(true)
        .map_err(|e| format!("Failed to set fullscreen: {e}"))?;
    window
        .set_always_on_top(true)
        .map_err(|e| format!("Failed to set always-on-top: {e}"))?;

    Ok(())
}
