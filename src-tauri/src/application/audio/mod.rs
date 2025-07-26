//! Audio Application Layer
//! 
//! This module contains the use cases for audio management, including:
//! - Audio playback control (play, pause, stop, volume)
//! - Audio library management (add, remove, organize assets)
//! - Notification sound handling
//! - Background audio for focus sessions
//! 
//! ## Design Principles
//! 
//! - All audio operations are async and return Results
//! - Dependencies are injected as trait objects for testability
//! - Volume validation is consistent across all use cases
//! - Clear separation between notification and background audio
//! - Audio services handle infrastructure concerns (file I/O, playback)
//! 
//! ## Use Cases
//! 
//! - `play_audio` - Core audio playback operations
//! - `manage_library` - Audio asset and library management
//! - `notification_audio` - Specialized notification and background audio

pub mod play_audio;
pub mod manage_library;
pub mod notification_audio;

// Re-export main types and functions for easier imports
pub use play_audio::{
    AudioService, PlayAudioCmd, StopAudioCmd, 
    play_audio, stop_audio, stop_all_audio, 
    pause_audio, resume_audio, set_audio_volume
};

pub use manage_library::{
    AudioLibraryService, AddAudioAssetCmd, RemoveAudioAssetCmd, GetAudioLibraryQuery,
    get_audio_library, add_audio_asset, remove_audio_asset, 
    get_audio_asset, get_assets_by_category
};

pub use notification_audio::{
    NotificationType, PlayNotificationSoundCmd, PlayBackgroundAudioCmd,
    play_notification_sound, play_background_audio, stop_background_audio
};