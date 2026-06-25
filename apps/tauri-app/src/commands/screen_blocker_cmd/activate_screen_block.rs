use super::{AppHandle, Manager};

/// Force the main window into fullscreen + always-on-top.
///
/// Invoked by the frontend when the screen blocker overlay is shown, so the
/// user cannot simply alt-tab away from the focus-enforcement overlay.
#[tauri::command(rename_all = "snake_case")]
pub async fn activate_screen_block(
    app_handle: AppHandle,
) -> Result<(), String> {
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
