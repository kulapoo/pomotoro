use super::asset::AudioAsset;
use super::library::{AudioLibrary, PlaybackHandle, PlaybackRequest};
use crate::shared_kernel::Result;

/// Domain service for audio playback operations
///
/// This trait defines the contract for audio operations that the application layer
/// can depend on. Concrete implementations reside in the infrastructure layer.
pub trait AudioService: Send + Sync {
    /// Start playing an audio asset with the specified configuration
    fn play_audio(
        &mut self,
        request: PlaybackRequest,
    ) -> Result<PlaybackHandle>;

    /// Stop a specific audio playback by its ID
    fn stop_audio(&mut self, playback_id: &str) -> Result<()>;

    /// Stop all currently playing audio
    fn stop_all_audio(&mut self) -> Result<()>;

    /// Pause a specific audio playback
    fn pause_audio(&mut self, playback_id: &str) -> Result<()>;

    /// Resume a paused audio playback
    fn resume_audio(&mut self, playback_id: &str) -> Result<()>;

    /// Set the volume for a specific playback (0.0 to 1.0)
    fn set_volume(&mut self, playback_id: &str, volume: f32) -> Result<()>;

    /// Get all currently active playback handles
    fn get_active_playbacks(&self) -> Result<Vec<PlaybackHandle>>;

    /// Clean up finished playbacks from internal state
    fn cleanup_finished(&mut self) -> Result<()>;

    /// Get the audio library with available assets
    fn get_library(&self) -> &AudioLibrary;

    /// Play a notification sound
    fn play_notification(&mut self, asset_id: &str, volume: f32) -> Result<PlaybackHandle>;

    /// Play background audio (looped with fade-in)
    fn play_background_audio(&mut self, asset_id: &str, volume: f32) -> Result<PlaybackHandle>;

    /// Stop all background audio (looped audio)
    fn stop_background_audio(&mut self) -> Result<()>;

    /// Add a new audio asset to the library
    fn add_asset(&mut self, asset: AudioAsset);

    /// Remove an audio asset from the library
    fn remove_asset(&mut self, asset_id: &str) -> Option<AudioAsset>;
}
