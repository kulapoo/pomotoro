use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;
use domain::{Result, Error, audio::{AudioService, AudioAsset, AudioLibrary, PlaybackRequest, PlaybackHandle}};

/// Mock audio service for testing
pub struct MockAudioService {
    play_count: AtomicUsize,
    stop_count: AtomicUsize,
    pause_count: AtomicUsize,
    resume_count: AtomicUsize,
    active_playbacks: Mutex<HashMap<String, PlaybackHandle>>,
    library: Mutex<AudioLibrary>,
    volume_calls: Mutex<Vec<(String, f32)>>,
}

impl MockAudioService {
    pub fn new() -> Self {
        Self {
            play_count: AtomicUsize::new(0),
            stop_count: AtomicUsize::new(0),
            pause_count: AtomicUsize::new(0),
            resume_count: AtomicUsize::new(0),
            active_playbacks: Mutex::new(HashMap::new()),
            library: Mutex::new(AudioLibrary::new()),
            volume_calls: Mutex::new(Vec::new()),
        }
    }

    pub fn play_count(&self) -> usize {
        self.play_count.load(Ordering::SeqCst)
    }

    pub fn stop_count(&self) -> usize {
        self.stop_count.load(Ordering::SeqCst)
    }

    pub fn pause_count(&self) -> usize {
        self.pause_count.load(Ordering::SeqCst)
    }

    pub fn resume_count(&self) -> usize {
        self.resume_count.load(Ordering::SeqCst)
    }

    pub fn active_playback_count(&self) -> usize {
        self.active_playbacks.lock().unwrap().len()
    }

    pub fn volume_calls(&self) -> Vec<(String, f32)> {
        self.volume_calls.lock().unwrap().clone()
    }

    pub fn reset_counts(&self) {
        self.play_count.store(0, Ordering::SeqCst);
        self.stop_count.store(0, Ordering::SeqCst);
        self.pause_count.store(0, Ordering::SeqCst);
        self.resume_count.store(0, Ordering::SeqCst);
        self.active_playbacks.lock().unwrap().clear();
        self.volume_calls.lock().unwrap().clear();
    }
}

impl AudioService for MockAudioService {
    fn play_audio(&mut self, request: PlaybackRequest) -> Result<PlaybackHandle> {
        self.play_count.fetch_add(1, Ordering::SeqCst);
        let handle = PlaybackHandle {
            id: format!("mock_playback_{}", self.play_count.load(Ordering::SeqCst)),
            asset_id: request.asset_id.clone(),
            is_playing: true,
            is_looped: request.looped,
        };
        self.active_playbacks.lock().unwrap()
            .insert(handle.id.clone(), handle.clone());
        Ok(handle)
    }

    fn stop_audio(&mut self, playback_id: &str) -> Result<()> {
        self.stop_count.fetch_add(1, Ordering::SeqCst);
        self.active_playbacks.lock().unwrap().remove(playback_id);
        Ok(())
    }

    fn stop_all_audio(&mut self) -> Result<()> {
        self.stop_count.fetch_add(1, Ordering::SeqCst);
        self.active_playbacks.lock().unwrap().clear();
        Ok(())
    }

    fn pause_audio(&mut self, playback_id: &str) -> Result<()> {
        self.pause_count.fetch_add(1, Ordering::SeqCst);
        if let Some(handle) = self.active_playbacks.lock().unwrap().get_mut(playback_id) {
            handle.is_playing = false;
        }
        Ok(())
    }

    fn resume_audio(&mut self, playback_id: &str) -> Result<()> {
        self.resume_count.fetch_add(1, Ordering::SeqCst);
        if let Some(handle) = self.active_playbacks.lock().unwrap().get_mut(playback_id) {
            handle.is_playing = true;
        }
        Ok(())
    }

    fn set_volume(&mut self, playback_id: &str, volume: f32) -> Result<()> {
        self.volume_calls.lock().unwrap()
            .push((playback_id.to_string(), volume));
        Ok(())
    }

    fn get_active_playbacks(&self) -> Result<Vec<PlaybackHandle>> {
        Ok(self.active_playbacks.lock().unwrap()
            .values()
            .cloned()
            .collect())
    }

    fn cleanup_finished(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_library(&self) -> &AudioLibrary {
        // This is a bit of a hack but works for testing
        // In real code, we'd need a different approach
        unsafe {
            &*(self.library.lock().unwrap().as_ref() as *const AudioLibrary)
        }
    }

    fn play_notification(&mut self, asset_id: &str, volume: f32) -> Result<PlaybackHandle> {
        self.play_audio(PlaybackRequest {
            asset_id: asset_id.to_string(),
            volume,
            looped: false,
        })
    }

    fn play_background_audio(&mut self, asset_id: &str, volume: f32) -> Result<PlaybackHandle> {
        self.play_audio(PlaybackRequest {
            asset_id: asset_id.to_string(),
            volume,
            looped: true,
        })
    }

    fn stop_background_audio(&mut self) -> Result<()> {
        // Stop all looped playbacks
        let looped_ids: Vec<String> = self.active_playbacks.lock().unwrap()
            .iter()
            .filter(|(_, h)| h.is_looped)
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in looped_ids {
            self.stop_audio(&id)?;
        }
        Ok(())
    }

    fn add_asset(&mut self, asset: AudioAsset) {
        self.library.lock().unwrap().add_asset(asset);
    }

    fn remove_asset(&mut self, asset_id: &str) -> Option<AudioAsset> {
        self.library.lock().unwrap().remove_asset(asset_id)
    }
}

impl Default for MockAudioService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_play_count() {
        let mut service = MockAudioService::new();
        let request = PlaybackRequest {
            asset_id: "test".to_string(),
            volume: 0.5,
            looped: false,
        };
        
        service.play_audio(request.clone()).unwrap();
        service.play_audio(request).unwrap();
        
        assert_eq!(service.play_count(), 2);
        assert_eq!(service.active_playback_count(), 2);
    }

    #[test]
    fn tracks_stop_operations() {
        let mut service = MockAudioService::new();
        let handle = service.play_audio(PlaybackRequest {
            asset_id: "test".to_string(),
            volume: 0.5,
            looped: false,
        }).unwrap();
        
        service.stop_audio(&handle.id).unwrap();
        assert_eq!(service.stop_count(), 1);
        assert_eq!(service.active_playback_count(), 0);
    }

    #[test]
    fn resets_counts() {
        let mut service = MockAudioService::new();
        service.play_audio(PlaybackRequest {
            asset_id: "test".to_string(),
            volume: 0.5,
            looped: false,
        }).unwrap();
        
        service.stop_all_audio().unwrap();
        assert_eq!(service.play_count(), 1);
        assert_eq!(service.stop_count(), 1);
        
        service.reset_counts();
        assert_eq!(service.play_count(), 0);
        assert_eq!(service.stop_count(), 0);
    }
}