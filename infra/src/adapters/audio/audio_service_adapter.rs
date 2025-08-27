use domain::{AudioService, PlaybackHandle, PlaybackRequest, Result};
use std::sync::{Arc, Mutex as StdMutex};
use super::RodioAudioService;

pub struct AudioServiceAdapter {
    inner: Arc<StdMutex<RodioAudioService>>,
}

impl AudioServiceAdapter {
    pub fn new(service: Arc<RodioAudioService>) -> Self {
        Self {
            inner: Arc::new(StdMutex::new(Arc::try_unwrap(service).unwrap_or_else(|arc| (*arc).clone()))),
        }
    }
}

impl AudioService for AudioServiceAdapter {
    fn play_audio(&mut self, request: PlaybackRequest) -> Result<PlaybackHandle> {
        let mut service = self.inner.lock().unwrap();
        service.play_audio(request)
    }

    fn stop_audio(&mut self, playback_id: &str) -> Result<()> {
        let mut service = self.inner.lock().unwrap();
        service.stop_audio(playback_id)
    }

    fn stop_all_audio(&mut self) -> Result<()> {
        let mut service = self.inner.lock().unwrap();
        service.stop_all_audio()
    }

    fn pause_audio(&mut self, playback_id: &str) -> Result<()> {
        let mut service = self.inner.lock().unwrap();
        service.pause_audio(playback_id)
    }

    fn resume_audio(&mut self, playback_id: &str) -> Result<()> {
        let mut service = self.inner.lock().unwrap();
        service.resume_audio(playback_id)
    }

    fn set_volume(&mut self, playback_id: &str, volume: f32) -> Result<()> {
        let mut service = self.inner.lock().unwrap();
        service.set_volume(playback_id, volume)
    }

    fn get_active_playbacks(&self) -> Result<Vec<PlaybackHandle>> {
        let service = self.inner.lock().unwrap();
        service.get_active_playbacks()
    }

    fn cleanup_finished(&mut self) -> Result<()> {
        let mut service = self.inner.lock().unwrap();
        service.cleanup_finished()
    }
}