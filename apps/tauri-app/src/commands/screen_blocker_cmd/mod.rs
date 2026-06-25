//! Screen blocker command handlers module
//!
//! Commands that drive the native-window side of the focus-enforcement
//! overlay: forcing the main window into fullscreen + always-on-top while
//! the in-DOM blocker is shown, and restoring it on dismiss.

pub use tauri::{AppHandle, Manager};

// Declare submodules
mod activate_screen_block;
mod deactivate_screen_block;

// Re-export all command functions
pub use activate_screen_block::activate_screen_block;
pub use deactivate_screen_block::deactivate_screen_block;
