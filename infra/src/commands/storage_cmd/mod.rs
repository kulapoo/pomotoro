//! Storage command handlers module
//!
//! This module contains all storage-related command handlers that serve as entry points
//! for external storage management requests.

// Common imports and types used across storage commands
pub use anyhow::{Context, Result};
pub use domain::{Config, ConfigRepository};
pub use std::path::PathBuf;
pub use std::sync::Arc;
pub use tauri::{command, State};

// Declare submodules
mod clear_all_data;
mod export_settings;
mod import_settings;
mod open_data_directory;
mod update_storage_path;
mod validate_storage_path;

// Re-export all command functions
pub use clear_all_data::clear_all_data;
pub use export_settings::export_settings;
pub use import_settings::import_settings;
pub use open_data_directory::open_data_directory;
pub use update_storage_path::update_storage_path;
pub use validate_storage_path::validate_storage_path;