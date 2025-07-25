use pomotoro_domain::{AudioAsset, AudioLibrary, PlaybackRequest, PlaybackHandle, AudioError};
use crate::infrastructure::audio_asset_provider::DefaultAudioAssetProvider;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

pub struct AudioService {
    stream_handle: OutputStream,
    library: AudioLibrary,
    active_playbacks: Arc<Mutex<HashMap<String, AudioPlayback>>>,
}

unsafe impl Send for AudioService {}
unsafe impl Sync for AudioService {}

struct AudioPlayback {
    sink: Sink,
    handle: PlaybackHandle,
}

impl AudioService {
    pub fn new() -> Result<Self, AudioError> {
        let stream_handle = OutputStreamBuilder::open_default_stream()
            .map_err(|e| AudioError::PlaybackFailed(format!("Failed to create audio stream: {}", e)))?;

        Ok(Self {
            stream_handle,
            library: DefaultAudioAssetProvider::create_library_with_default_assets(),
            active_playbacks: Arc::new(Mutex::new(HashMap::new())),
        })
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

    pub fn play(&self, request: PlaybackRequest) -> Result<PlaybackHandle, AudioError> {
        let asset = self.library
            .get_asset(&request.asset_id)
            .ok_or_else(|| AudioError::AssetNotFound(request.asset_id.clone()))?;

        let file = File::open(&asset.file_path)
            .map_err(|_| AudioError::InvalidFile(asset.file_path.to_string_lossy().to_string()))?;

        let reader = BufReader::new(file);
        let decoder = Decoder::new(reader)
            .map_err(|e| AudioError::PlaybackFailed(format!("Failed to decode audio: {}", e)))?;

        let sink = Sink::connect_new(self.stream_handle.mixer());

        if let Some(fade_in_ms) = request.fade_in_ms {
            let source: Box<dyn Source<Item = f32> + Send> = if request.looped {
                Box::new(decoder.repeat_infinite().amplify(request.volume).fade_in(Duration::from_millis(fade_in_ms as u64)))
            } else {
                Box::new(decoder.amplify(request.volume).fade_in(Duration::from_millis(fade_in_ms as u64)))
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

    pub fn stop_playback(&self, handle_id: &str) -> Result<(), AudioError> {
        if let Ok(mut active_playbacks) = self.active_playbacks.lock() {
            if let Some(playback) = active_playbacks.remove(handle_id) {
                playback.sink.stop();
                return Ok(());
            }
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    pub fn pause_playback(&self, handle_id: &str) -> Result<(), AudioError> {
        if let Ok(active_playbacks) = self.active_playbacks.lock() {
            if let Some(playback) = active_playbacks.get(handle_id) {
                playback.sink.pause();
                return Ok(());
            }
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    pub fn resume_playback(&self, handle_id: &str) -> Result<(), AudioError> {
        if let Ok(active_playbacks) = self.active_playbacks.lock() {
            if let Some(playback) = active_playbacks.get(handle_id) {
                playback.sink.play();
                return Ok(());
            }
        }
        Err(AudioError::AssetNotFound(handle_id.to_string()))
    }

    pub fn set_volume(&self, handle_id: &str, volume: f32) -> Result<(), AudioError> {
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

    pub fn play_notification(&self, asset_id: &str, volume: f32) -> Result<PlaybackHandle, AudioError> {
        let request = PlaybackRequest::new(asset_id.to_string(), volume)?;
        self.play(request)
    }

    pub fn play_background_audio(&self, asset_id: &str, volume: f32) -> Result<PlaybackHandle, AudioError> {
        let request = PlaybackRequest::new(asset_id.to_string(), volume)?
            .with_loop()
            .with_fade_in(2000);
        self.play(request)
    }

    pub fn stop_background_audio(&self) {
        let handles: Vec<String> = if let Ok(active_playbacks) = self.active_playbacks.lock() {
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
}