use domain::{AudioAsset, AudioLibrary, AudioService, PlaybackHandle, PlaybackRequest, Result};
use std::sync::Mutex;

pub struct AudioServiceWrapper {
    inner: Mutex<Box<dyn AudioService>>,
}

unsafe impl Send for AudioServiceWrapper {}
unsafe impl Sync for AudioServiceWrapper {}

impl AudioServiceWrapper {
    pub fn new(service: Box<dyn AudioService>) -> Self {
        Self {
            inner: Mutex::new(service),
        }
    }

    pub fn play_audio(&self, request: PlaybackRequest) -> Result<PlaybackHandle> {
        self.inner.lock().unwrap().play_audio(request)
    }

    pub fn stop_audio(&self, playback_id: &str) -> Result<()> {
        self.inner.lock().unwrap().stop_audio(playback_id)
    }

    pub fn stop_all_audio(&self) -> Result<()> {
        self.inner.lock().unwrap().stop_all_audio()
    }

    pub fn pause_audio(&self, playback_id: &str) -> Result<()> {
        self.inner.lock().unwrap().pause_audio(playback_id)
    }

    pub fn resume_audio(&self, playback_id: &str) -> Result<()> {
        self.inner.lock().unwrap().resume_audio(playback_id)
    }

    pub fn set_volume(&self, playback_id: &str, volume: f32) -> Result<()> {
        self.inner.lock().unwrap().set_volume(playback_id, volume)
    }

    pub fn get_active_playbacks(&self) -> Result<Vec<PlaybackHandle>> {
        self.inner.lock().unwrap().get_active_playbacks()
    }

    pub fn cleanup_finished(&self) -> Result<()> {
        self.inner.lock().unwrap().cleanup_finished()
    }

    pub fn get_library(&self) -> AudioLibrary {
        self.inner.lock().unwrap().get_library().clone()
    }

    pub fn play_notification(&self, asset_id: &str, volume: f32) -> Result<PlaybackHandle> {
        self.inner.lock().unwrap().play_notification(asset_id, volume)
    }

    pub fn play_background_audio(&self, asset_id: &str, volume: f32) -> Result<PlaybackHandle> {
        self.inner.lock().unwrap().play_background_audio(asset_id, volume)
    }

    pub fn stop_background_audio(&self) -> Result<()> {
        self.inner.lock().unwrap().stop_background_audio()
    }

    pub fn add_asset(&self, asset: AudioAsset) {
        self.inner.lock().unwrap().add_asset(asset)
    }

    pub fn remove_asset(&self, asset_id: &str) -> Option<AudioAsset> {
        self.inner.lock().unwrap().remove_asset(asset_id)
    }
}