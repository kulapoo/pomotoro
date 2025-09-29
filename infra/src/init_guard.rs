use crate::AppState;
use std::sync::Arc;
use tauri::AppHandle;

/// Macro to ensure app is initialized before executing command logic
#[macro_export]
macro_rules! ensure_initialized {
    ($app_state:expr, $app_handle:expr) => {
        match $app_state.ensure_initialized($app_handle).await {
            Ok(registry) => registry,
            Err(err) => return Err(err),
        }
    };
}

/// Helper function to get the app registry or initialize it if needed
pub async fn get_or_init_registry(
    app_state: &AppState,
    app_handle: AppHandle,
) -> Result<Arc<crate::bootstrap::AppRegistry>, String> {
    app_state.ensure_initialized(app_handle).await
}

/// Wrapper type for commands that need initialization
pub struct InitializedCommand;

impl InitializedCommand {
    /// Ensures the app is initialized and returns the registry
    pub async fn ensure_ready(
        app_state: &AppState,
        app_handle: AppHandle,
    ) -> Result<Arc<crate::bootstrap::AppRegistry>, String> {
        app_state.ensure_initialized(app_handle).await
    }
}