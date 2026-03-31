use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn activate_screen_block(app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_fullscreen(false);
        let _ = window.set_always_on_top(false);
        let _ = window.set_always_on_top(true);
        let _ = window.set_fullscreen(true);
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub fn deactivate_screen_block(app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_fullscreen(false);
        let _ = window.set_always_on_top(false);
    }
}
