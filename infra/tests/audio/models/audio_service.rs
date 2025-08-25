use domain::{AudioError, AudioLibrary, PlaybackHandle, PlaybackRequest};
use infra::adapters::DefaultAudioAssetProvider;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MockAudioManager {
    library: AudioLibrary,
    active_playbacks: Arc<Mutex<HashMap<String, MockPlayback>>>,
    playback_counter: Arc<Mutex<usize>>,
    asset_play_counts: Arc<Mutex<HashMap<String, usize>>>,
}

#[derive(Debug, Clone)]
pub struct MockPlayback {
    pub handle: PlaybackHandle,
    pub volume: f32,
    pub is_playing: bool,
    pub is_paused: bool,
    pub is_looped: bool,
}

impl MockAudioManager {
    pub fn new() -> Result<Self, AudioError> {
        Ok(Self {
            library:
                DefaultAudioAssetProvider::create_library_with_default_assets(),
            active_playbacks: Arc::new(Mutex::new(HashMap::new())),
            playback_counter: Arc::new(Mutex::new(0)),
            asset_play_counts: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn get_library(&self) -> &AudioLibrary {
        &self.library
    }

    pub fn play(
        &self,
        request: PlaybackRequest,
    ) -> Result<PlaybackHandle, AudioError> {
        let mut counter = self.playback_counter.lock().unwrap();
        *counter += 1;
        let handle_id = format!("mock_handle_{}", *counter);
        drop(counter);

        let handle = PlaybackHandle {
            id: handle_id.clone(),
            asset_id: request.asset_id.clone(),
            volume: request.volume,
            is_playing: true,
            is_looped: request.looped,
        };

        let playback = MockPlayback {
            handle: handle.clone(),
            volume: request.volume,
            is_playing: true,
            is_paused: false,
            is_looped: request.looped,
        };

        // Track play count for this asset
        let mut play_counts = self.asset_play_counts.lock().unwrap();
        *play_counts.entry(request.asset_id).or_insert(0) += 1;
        drop(play_counts);

        let mut playbacks = self.active_playbacks.lock().unwrap();
        playbacks.insert(handle_id, playback);

        Ok(handle)
    }

    pub fn stop_playback(&self, handle_id: &str) -> Result<(), AudioError> {
        let mut playbacks = self.active_playbacks.lock().unwrap();
        if let Some(playback) = playbacks.get_mut(handle_id) {
            playback.is_playing = false;
            playback.is_paused = false;
        }
        Ok(())
    }

    pub fn pause_playback(&self, handle_id: &str) -> Result<(), AudioError> {
        let mut playbacks = self.active_playbacks.lock().unwrap();
        if let Some(playback) = playbacks.get_mut(handle_id) {
            playback.is_paused = true;
            playback.is_playing = false;
        }
        Ok(())
    }

    pub fn resume_playback(&self, handle_id: &str) -> Result<(), AudioError> {
        let mut playbacks = self.active_playbacks.lock().unwrap();
        if let Some(playback) = playbacks.get_mut(handle_id) {
            playback.is_paused = false;
            playback.is_playing = true;
        }
        Ok(())
    }

    pub fn set_volume(
        &self,
        handle_id: &str,
        volume: f32,
    ) -> Result<(), AudioError> {
        let mut playbacks = self.active_playbacks.lock().unwrap();
        if let Some(playback) = playbacks.get_mut(handle_id) {
            playback.volume = volume;
            playback.handle.volume = volume;
        }
        Ok(())
    }

    pub fn stop_all_playbacks(&self) {
        let mut playbacks = self.active_playbacks.lock().unwrap();
        for playback in playbacks.values_mut() {
            playback.is_playing = false;
            playback.is_paused = false;
        }
    }

    pub fn play_notification(
        &self,
        asset_id: &str,
        volume: f32,
    ) -> Result<PlaybackHandle, AudioError> {
        let request = PlaybackRequest::new(asset_id.to_string(), volume)
            .map_err(|e| AudioError::PlaybackFailed(e.to_string()))?;
        self.play(request)
    }

    pub fn play_background_audio(
        &self,
        asset_id: &str,
        volume: f32,
    ) -> Result<PlaybackHandle, AudioError> {
        let request = PlaybackRequest::new(asset_id.to_string(), volume)
            .map_err(|e| AudioError::PlaybackFailed(e.to_string()))?
            .with_loop()
            .with_fade_in(1000);
        self.play(request)
    }

    pub fn stop_background_audio(&self) {
        let mut playbacks = self.active_playbacks.lock().unwrap();
        for playback in playbacks.values_mut() {
            if playback.is_looped {
                playback.is_playing = false;
                playback.is_paused = false;
            }
        }
    }

    pub fn cleanup_finished_playbacks(&self) {
        let mut playbacks = self.active_playbacks.lock().unwrap();
        playbacks
            .retain(|_, playback| playback.is_playing || playback.is_paused);
    }

    pub fn get_playback_count(&self) -> usize {
        let playbacks = self.active_playbacks.lock().unwrap();
        playbacks.len()
    }

    pub fn get_playing_count(&self) -> usize {
        let playbacks = self.active_playbacks.lock().unwrap();
        playbacks.values().filter(|p| p.is_playing).count()
    }

    pub fn get_paused_count(&self) -> usize {
        let playbacks = self.active_playbacks.lock().unwrap();
        playbacks.values().filter(|p| p.is_paused).count()
    }

    pub fn is_playing(&self, handle_id: &str) -> bool {
        let playbacks = self.active_playbacks.lock().unwrap();
        playbacks
            .get(handle_id)
            .map(|p| p.is_playing)
            .unwrap_or(false)
    }

    pub fn is_paused(&self, handle_id: &str) -> bool {
        let playbacks = self.active_playbacks.lock().unwrap();
        playbacks
            .get(handle_id)
            .map(|p| p.is_paused)
            .unwrap_or(false)
    }

    pub fn get_playback_volume(&self, handle_id: &str) -> Option<f32> {
        let playbacks = self.active_playbacks.lock().unwrap();
        playbacks.get(handle_id).map(|p| p.volume)
    }

    pub fn get_asset_playback_count(&self, asset_id: &str) -> usize {
        let play_counts = self.asset_play_counts.lock().unwrap();
        *play_counts.get(asset_id).unwrap_or(&0)
    }
}

impl Default for MockAudioManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

unsafe impl Send for MockAudioManager {}
unsafe impl Sync for MockAudioManager {}
