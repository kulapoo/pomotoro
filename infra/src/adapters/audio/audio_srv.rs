use super::asset_provider::DefaultAudioAssetProvider;
use domain::{
    AudioAsset, AudioError, AudioLibrary, AudioService, PlaybackHandle,
    PlaybackRequest, Result,
};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Concrete implementation of AudioService using the Rodio audio library
///
/// This infrastructure implementation provides audio playback capabilities
/// while implementing the domain AudioService interface.
pub struct RodioAudioService {
    stream_handle: OutputStream,
    library: AudioLibrary,
    active_playbacks: Arc<Mutex<HashMap<String, AudioPlayback>>>,
}

unsafe impl Send for RodioAudioService {}
unsafe impl Sync for RodioAudioService {}

struct AudioPlayback {
    sink: Sink,
    handle: PlaybackHandle,
}

impl RodioAudioService {
    pub fn new() -> std::result::Result<Self, AudioError> {
        let stream_handle = OutputStreamBuilder::open_default_stream()
            .map_err(|e| {
                AudioError::PlaybackFailed(format!(
                    "Failed to create audio stream: {e}"
                ))
            })?;

        Ok(Self {
            stream_handle,
            library:
                DefaultAudioAssetProvider::create_library_with_default_assets(),
            active_playbacks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Resolve relative paths to absolute paths based on the current directory
    /// or the executable's directory
    fn resolve_audio_path(path: &Path) -> PathBuf {
        // If the path is already absolute, return it as-is
        if path.is_absolute() {
            return path.to_path_buf();
        }

        // Try to resolve relative to current directory first
        let from_cwd = std::env::current_dir().ok().map(|cwd| cwd.join(path));

        if let Some(ref p) = from_cwd {
            if p.exists() {
                return p.clone();
            }
        }

        // If not found, try relative to executable location
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // In development, the executable might be in target/debug or target/release
                // We need to go up to the project root
                let mut current = exe_dir;

                // Look for the assets folder by traversing up
                for _ in 0..5 {
                    // Limit traversal depth
                    let candidate = current.join(path);
                    if candidate.exists() {
                        return candidate;
                    }

                    // Also check if we need to go to parent
                    if let Some(parent) = current.parent() {
                        let parent_candidate = parent.join(path);
                        if parent_candidate.exists() {
                            return parent_candidate;
                        }
                        current = parent;
                    } else {
                        break;
                    }
                }
            }
        }

        // If all else fails, return the original path
        // (will likely fail when trying to open, but preserves original error message)
        path.to_path_buf()
    }

    pub fn get_library(&self) -> &AudioLibrary {
        &self.library
    }

    pub fn add_asset(&mut self, asset: AudioAsset) {
        self.library.add_asset(asset);
    }

    pub fn remove_asset(&mut self, asset_id: &str) -> Option<AudioAsset> {
        self.stop_playback(asset_id).ok();
        self.library.remove_asset(asset_id)
    }

    pub fn play(
        &self,
        request: PlaybackRequest,
    ) -> std::result::Result<PlaybackHandle, AudioError> {
        let asset =
            self.library.get_asset(&request.asset_id).ok_or_else(|| {
                AudioError::AssetNotFound(request.asset_id.clone())
            })?;

        if asset.file_path.to_string_lossy().starts_with("embedded:") {
            return self.play_embedded_sound(request, asset);
        }

        // Resolve the audio file path
        let resolved_path = Self::resolve_audio_path(&asset.file_path);

        let file = File::open(&resolved_path).map_err(|e| {
            eprintln!("Warning: Audio file not found: {} (resolved to: {}). Error: {}", 
                asset.file_path.to_string_lossy(),
                resolved_path.to_string_lossy(),
                e);
            AudioError::InvalidFile(
                asset.file_path.to_string_lossy().to_string(),
            )
        })?;

        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader).map_err(|e| {
            AudioError::PlaybackFailed(format!("Failed to decode audio: {e}"))
        })?;

        let sink = Sink::connect_new(self.stream_handle.mixer());

        if let Some(fade_in_ms) = request.fade_in_ms {
            let source: Box<dyn Source<Item = f32> + Send> = if request.looped {
                Box::new(
                    decoder
                        .repeat_infinite()
                        .amplify(request.volume)
                        .fade_in(Duration::from_millis(fade_in_ms as u64)),
                )
            } else {
                Box::new(
                    decoder
                        .amplify(request.volume)
                        .fade_in(Duration::from_millis(fade_in_ms as u64)),
                )
            };
            sink.append(source);
        } else {
            let source: Box<dyn Source<Item = f32> + Send> = if request.looped {
                Box::new(decoder.repeat_infinite().amplify(request.volume))
            } else {
                Box::new(decoder.amplify(request.volume))
            };
            sink.append(source);
        }

        let handle_id = Uuid::new_v4().to_string();
        let handle = PlaybackHandle {
            id: handle_id.clone(),
            asset_id: request.asset_id.clone(),
            is_playing: true,
            is_looped: request.looped,
            volume: request.volume,
        };

        let playback = AudioPlayback {
            sink,
            handle: handle.clone(),
        };

        if let Ok(mut active_playbacks) = self.active_playbacks.lock() {
            active_playbacks.insert(handle_id, playback);
        }

        Ok(handle)
    }

    pub fn stop_playback(
        &self,
        handle_id: &str,
    ) -> std::result::Result<(), AudioError> {
        if let Ok(mut active_playbacks) = self.active_playbacks.lock() {
            if let Some(playback) = active_playbacks.remove(handle_id) {
                playback.sink.stop();
                return Ok(());
            }
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    pub fn pause_playback(
        &self,
        handle_id: &str,
    ) -> std::result::Result<(), AudioError> {
        if let Ok(active_playbacks) = self.active_playbacks.lock() {
            if let Some(playback) = active_playbacks.get(handle_id) {
                playback.sink.pause();
                return Ok(());
            }
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    pub fn resume_playback(
        &self,
        handle_id: &str,
    ) -> std::result::Result<(), AudioError> {
        if let Ok(active_playbacks) = self.active_playbacks.lock() {
            if let Some(playback) = active_playbacks.get(handle_id) {
                playback.sink.play();
                return Ok(());
            }
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    pub fn set_audio_volume(
        &self,
        handle_id: &str,
        volume: f32,
    ) -> std::result::Result<(), AudioError> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(AudioError::VolumeOutOfRange(volume));
        }

        if let Ok(active_playbacks) = self.active_playbacks.lock() {
            if let Some(playback) = active_playbacks.get(handle_id) {
                playback.sink.set_volume(volume);
                return Ok(());
            }
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    pub fn get_active_playbacks(&self) -> Vec<PlaybackHandle> {
        if let Ok(active_playbacks) = self.active_playbacks.lock() {
            return active_playbacks
                .values()
                .map(|playback| playback.handle.clone())
                .collect();
        }
        Vec::new()
    }

    pub fn stop_all_playbacks(&self) {
        if let Ok(mut active_playbacks) = self.active_playbacks.lock() {
            for (_, playback) in active_playbacks.drain() {
                playback.sink.stop();
            }
        }
    }

    pub fn play_notification(
        &self,
        asset_id: &str,
        volume: f32,
    ) -> std::result::Result<PlaybackHandle, AudioError> {
        let request = PlaybackRequest::new(asset_id.to_string(), volume)?;
        self.play(request)
    }

    pub fn play_background_audio(
        &self,
        asset_id: &str,
        volume: f32,
    ) -> std::result::Result<PlaybackHandle, AudioError> {
        let request = PlaybackRequest::new(asset_id.to_string(), volume)?
            .with_loop()
            .with_fade_in(2000);
        self.play(request)
    }

    pub fn stop_background_audio(&self) {
        let handles: Vec<String> =
            if let Ok(active_playbacks) = self.active_playbacks.lock() {
                active_playbacks
                    .values()
                    .filter(|playback| playback.handle.is_looped)
                    .map(|playback| playback.handle.id.clone())
                    .collect()
            } else {
                return;
            };

        for handle_id in handles {
            self.stop_playback(&handle_id).ok();
        }
    }

    pub fn cleanup_finished_playbacks(&self) {
        if let Ok(mut active_playbacks) = self.active_playbacks.lock() {
            let finished_handles: Vec<String> = active_playbacks
                .iter()
                .filter(|(_, playback)| playback.sink.empty())
                .map(|(handle_id, _)| handle_id.clone())
                .collect();

            for handle_id in finished_handles {
                active_playbacks.remove(&handle_id);
            }
        }
    }

    fn play_embedded_sound(
        &self,
        request: PlaybackRequest,
        _asset: &AudioAsset,
    ) -> std::result::Result<PlaybackHandle, AudioError> {
        let sink = Sink::connect_new(self.stream_handle.mixer());

        let silent_samples: Vec<f32> = vec![0.0; 44100 / 10];
        let source =
            rodio::buffer::SamplesBuffer::new(1, 44100, silent_samples);

        sink.append(source);
        sink.set_volume(0.0);

        let handle_id = Uuid::new_v4().to_string();
        let handle = PlaybackHandle {
            id: handle_id.clone(),
            asset_id: request.asset_id.clone(),
            is_playing: true,
            is_looped: false,
            volume: 0.0,
        };

        let playback = AudioPlayback {
            sink,
            handle: handle.clone(),
        };

        if let Ok(mut active_playbacks) = self.active_playbacks.lock() {
            active_playbacks.insert(handle_id, playback);
        }

        Ok(handle)
    }
}

/// Implementation of the domain AudioService trait
///
/// This allows the infrastructure RodioAudioService to be used as a dependency
/// in the application layer through the domain interface.
impl AudioService for RodioAudioService {
    fn play_audio(
        &mut self,
        request: PlaybackRequest,
    ) -> Result<PlaybackHandle> {
        self.play(request).map_err(|e| e.into())
    }

    fn stop_audio(&mut self, playback_id: &str) -> Result<()> {
        self.stop_playback(playback_id).map_err(|e| e.into())
    }

    fn stop_all_audio(&mut self) -> Result<()> {
        self.stop_all_playbacks();
        Ok(())
    }

    fn pause_audio(&mut self, playback_id: &str) -> Result<()> {
        self.pause_playback(playback_id).map_err(|e| e.into())
    }

    fn resume_audio(&mut self, playback_id: &str) -> Result<()> {
        self.resume_playback(playback_id).map_err(|e| e.into())
    }

    fn set_volume(&mut self, playback_id: &str, volume: f32) -> Result<()> {
        self.set_audio_volume(playback_id, volume)
            .map_err(|e| e.into())
    }

    fn get_active_playbacks(&self) -> Result<Vec<PlaybackHandle>> {
        Ok(RodioAudioService::get_active_playbacks(self))
    }

    fn cleanup_finished(&mut self) -> Result<()> {
        self.cleanup_finished_playbacks();
        Ok(())
    }

    fn get_library(&self) -> &AudioLibrary {
        RodioAudioService::get_library(self)
    }

    fn play_notification(
        &mut self,
        asset_id: &str,
        volume: f32,
    ) -> Result<PlaybackHandle> {
        RodioAudioService::play_notification(self, asset_id, volume)
            .map_err(|e| e.into())
    }

    fn play_background_audio(
        &mut self,
        asset_id: &str,
        volume: f32,
    ) -> Result<PlaybackHandle> {
        RodioAudioService::play_background_audio(self, asset_id, volume)
            .map_err(|e| e.into())
    }

    fn stop_background_audio(&mut self) -> Result<()> {
        RodioAudioService::stop_background_audio(self);
        Ok(())
    }

    fn add_asset(&mut self, asset: AudioAsset) {
        RodioAudioService::add_asset(self, asset);
    }

    fn remove_asset(&mut self, asset_id: &str) -> Option<AudioAsset> {
        RodioAudioService::remove_asset(self, asset_id)
    }
}
