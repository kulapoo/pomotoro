//! Config command handlers module
//!
//! This module contains all configuration-related command handlers that serve as entry points
//! for external configuration management requests.

// Common imports and types used across multiple config commands
pub use crate::adapters::events::mem_event_bus::EventPublisherArc;
pub use anyhow::Context;
pub use domain::{
    AppearanceConfig, AudioConfig, Config, ConfigRepository, ConfigReset, ConfigUpdated,
    GeneralConfig, NotificationConfig, TaskId, TimerConfiguration,
};
pub use std::sync::Arc;
pub use tauri::State;

// Declare submodules
mod get_effective_audio_config;
mod get_global_config;
mod reset_config_to_defaults;
mod save_global_config;
mod update_appearance_config;
mod update_audio_config;
mod update_general_config;
mod update_notification_config;
mod update_timing_config;

// Re-export all command functions
pub use get_effective_audio_config::get_effective_audio_config;
pub use get_global_config::get_global_config;
pub use reset_config_to_defaults::reset_config_to_defaults;
pub use save_global_config::save_global_config;
pub use update_appearance_config::update_appearance_config;
pub use update_audio_config::update_audio_config;
pub use update_general_config::update_general_config;
pub use update_notification_config::update_notification_config;
pub use update_timing_config::update_timing_config;