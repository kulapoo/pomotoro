/// Example wrapper module showing how to handle initialization in commands
/// This module provides examples of wrapping commands to ensure initialization
use crate::AppState;
use tauri::{AppHandle, State};

/// Example wrapper that ensures initialization before executing
/// This can be used as a template for wrapping existing commands
pub async fn with_initialization<F, T>(
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
    func: F,
) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    // Ensure app is initialized
    app_state.ensure_initialized(app_handle).await?;

    // Execute the actual command logic
    func()
}

/// Alternative approach: wrapper for async functions
pub async fn with_initialization_async<F, Fut, T>(
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
    func: F,
) -> Result<T, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    // Ensure app is initialized
    app_state.ensure_initialized(app_handle).await?;

    // Execute the actual command logic
    func().await
}