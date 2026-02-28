//! Audio command handlers module
//!
//! This module contains all audio-related command handlers that serve as entry points
//! for external audio control requests.

// Common imports and types used across multiple audio commands
pub use anyhow::Context;
pub use domain::{AudioAsset, AudioLibrary, PlaybackHandle, PlaybackRequest};
pub use infra::adapters::audio::AudioServiceWrapper;
pub use tauri::State;

pub type AudioServiceState<'a> = State<'a, AudioServiceWrapper>;

// Declare submodules
mod add_custom_audio_asset;
mod cleanup_finished_audio;
mod get_active_playbacks;
mod get_audio_library;
mod pause_audio;
mod play_audio;
mod play_background_audio;
mod play_notification_sound;
mod remove_audio_asset;
mod resume_audio;
mod set_audio_volume;
mod stop_all_audio;
mod stop_audio;
mod stop_background_audio;
mod test_audio_preview;

// Re-export all command functions
pub use add_custom_audio_asset::add_custom_audio_asset;
pub use cleanup_finished_audio::cleanup_finished_audio;
pub use get_active_playbacks::get_active_playbacks;
pub use get_audio_library::get_audio_library;
pub use pause_audio::pause_audio;
pub use play_audio::play_audio;
pub use play_background_audio::play_background_audio;
pub use play_notification_sound::play_notification_sound;
pub use remove_audio_asset::remove_audio_asset;
pub use resume_audio::resume_audio;
pub use set_audio_volume::set_audio_volume;
pub use stop_all_audio::stop_all_audio;
pub use stop_audio::stop_audio;
pub use stop_background_audio::stop_background_audio;
pub use test_audio_preview::test_audio_preview;
