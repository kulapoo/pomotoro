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

pub mod manage_library;
pub mod notification_audio;
pub mod play_audio;

pub use play_audio::{
    PlayAudioCmd, StopAudioCmd, pause_audio, play_audio, resume_audio,
    set_audio_volume, stop_all_audio, stop_audio,
};

pub use manage_library::{
    AddAudioAssetCmd, AudioLibraryService, GetAudioLibraryQuery,
    RemoveAudioAssetCmd, add_audio_asset, get_assets_by_category,
    get_audio_asset, get_audio_library, remove_audio_asset,
};

pub use notification_audio::{
    NotificationType, PlayBackgroundAudioCmd, PlayNotificationSoundCmd,
    play_background_audio, play_notification_sound, stop_background_audio,
};
