use domain::{AudioService, Error, PlaybackHandle, PlaybackRequest, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct PlayAudioCmd {
    pub asset_id: String,
    pub volume: f32,
    pub looped: bool,
    pub fade_in_ms: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct StopAudioCmd {
    pub playback_id: String,
}

pub async fn play_audio(
    audio_service: &Arc<Mutex<dyn AudioService>>,
    cmd: PlayAudioCmd,
) -> Result<PlaybackHandle> {
    if !(0.0..=1.0).contains(&cmd.volume) {
        return Err(Error::ConfigurationError {
            message: format!(
                "Volume must be between 0.0 and 1.0, got {}",
                cmd.volume
            ),
        });
    }

    let mut request =
        PlaybackRequest::new(cmd.asset_id, cmd.volume).map_err(|e| {
            Error::ConfigurationError {
                message: format!("Failed to create playback request: {e:?}"),
            }
        })?;

    if cmd.looped {
        request = request.with_loop();
    }

    if let Some(fade_in) = cmd.fade_in_ms {
        request = request.with_fade_in(fade_in);
    }

    let mut service = audio_service.lock().await;

    service.play_audio(request)
}

pub async fn stop_audio(
    audio_service: &Arc<Mutex<dyn AudioService>>,
    cmd: StopAudioCmd,
) -> Result<()> {
    let mut service = audio_service.lock().await;

    service.stop_audio(&cmd.playback_id)
}

pub async fn stop_all_audio(
    audio_service: &Arc<Mutex<dyn AudioService>>,
) -> Result<()> {
    let mut service = audio_service.lock().await;

    service.stop_all_audio()
}

pub async fn pause_audio(
    audio_service: &Arc<Mutex<dyn AudioService>>,
    playback_id: String,
) -> Result<()> {
    let mut service = audio_service.lock().await;

    service.pause_audio(&playback_id)
}

pub async fn resume_audio(
    audio_service: &Arc<Mutex<dyn AudioService>>,
    playback_id: String,
) -> Result<()> {
    let mut service = audio_service.lock().await;

    service.resume_audio(&playback_id)
}

pub async fn set_audio_volume(
    audio_service: &Arc<Mutex<dyn AudioService>>,
    playback_id: String,
    volume: f32,
) -> Result<()> {
    if !(0.0..=1.0).contains(&volume) {
        return Err(Error::ConfigurationError {
            message: format!(
                "Volume must be between 0.0 and 1.0, got {volume}"
            ),
        });
    }

    let mut service = audio_service.lock().await;

    service.set_volume(&playback_id, volume)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{AudioAsset, AudioCategory, AudioLibrary, PlaybackHandle};
    use std::collections::HashMap;

    struct MockAudioService {
        playbacks: HashMap<String, PlaybackHandle>,
        library: AudioLibrary,
    }

    impl MockAudioService {
        fn new() -> Self {
            let mut library = AudioLibrary::new();
            library.add_asset(AudioAsset {
                id: "test-audio".to_string(),
                name: "Test Audio".to_string(),
                file_path: "/fake/path/audio.mp3".into(),
                category: AudioCategory::BackgroundAmbient,
                duration_ms: Some(5000),
            });

            Self {
                playbacks: HashMap::new(),
                library,
            }
        }
    }

    impl AudioService for MockAudioService {
        fn play_audio(
            &mut self,
            request: PlaybackRequest,
        ) -> Result<PlaybackHandle> {
            let handle = PlaybackHandle {
                id: format!("test-{}", uuid::Uuid::new_v4()),
                asset_id: request.asset_id,
                is_playing: true,
                is_looped: request.looped,
                volume: request.volume,
            };
            self.playbacks.insert(handle.id.clone(), handle.clone());
            Ok(handle)
        }

        fn stop_audio(&mut self, playback_id: &str) -> Result<()> {
            if let Some(handle) = self.playbacks.get_mut(playback_id) {
                handle.is_playing = false;
            }
            Ok(())
        }

        fn stop_all_audio(&mut self) -> Result<()> {
            for handle in self.playbacks.values_mut() {
                handle.is_playing = false;
            }
            Ok(())
        }

        fn pause_audio(&mut self, playback_id: &str) -> Result<()> {
            if let Some(handle) = self.playbacks.get_mut(playback_id) {
                handle.is_playing = false;
            }
            Ok(())
        }

        fn resume_audio(&mut self, playback_id: &str) -> Result<()> {
            if let Some(handle) = self.playbacks.get_mut(playback_id) {
                handle.is_playing = true;
            }
            Ok(())
        }

        fn set_volume(&mut self, playback_id: &str, volume: f32) -> Result<()> {
            if let Some(handle) = self.playbacks.get_mut(playback_id) {
                handle.volume = volume;
            }
            Ok(())
        }

        fn get_active_playbacks(&self) -> Result<Vec<PlaybackHandle>> {
            Ok(self
                .playbacks
                .values()
                .filter(|h| h.is_playing)
                .cloned()
                .collect())
        }

        fn cleanup_finished(&mut self) -> Result<()> {
            self.playbacks.retain(|_, handle| handle.is_playing);
            Ok(())
        }

        fn get_library(&self) -> &AudioLibrary {
            &self.library
        }

        fn play_notification(
            &mut self,
            asset_id: &str,
            volume: f32,
        ) -> Result<PlaybackHandle> {
            let request = PlaybackRequest {
                asset_id: asset_id.to_string(),
                volume,
                looped: false,
                fade_in_ms: None,
                fade_out_ms: None,
            };
            self.play_audio(request)
        }

        fn play_background_audio(
            &mut self,
            asset_id: &str,
            volume: f32,
        ) -> Result<PlaybackHandle> {
            let request = PlaybackRequest {
                asset_id: asset_id.to_string(),
                volume,
                looped: true,
                fade_in_ms: Some(1000),
                fade_out_ms: None,
            };
            self.play_audio(request)
        }

        fn stop_background_audio(&mut self) -> Result<()> {
            let looped_ids: Vec<String> = self
                .playbacks
                .values()
                .filter(|h| h.is_looped && h.is_playing)
                .map(|h| h.id.clone())
                .collect();

            for id in looped_ids {
                self.stop_audio(&id)?;
            }
            Ok(())
        }

        fn add_asset(&mut self, asset: AudioAsset) {
            self.library.add_asset(asset);
        }

        fn remove_asset(&mut self, asset_id: &str) -> Option<AudioAsset> {
            self.library.remove_asset(asset_id)
        }
    }

    #[tokio::test]
    async fn should_play_audio_successfully() {
        let service: Arc<Mutex<dyn AudioService>> =
            Arc::new(Mutex::new(MockAudioService::new()));

        let cmd = PlayAudioCmd {
            asset_id: "test-asset".to_string(),
            volume: 0.8,
            looped: false,
            fade_in_ms: None,
        };

        let handle = play_audio(&service, cmd).await.unwrap();

        assert_eq!(handle.asset_id, "test-asset");
        assert_eq!(handle.volume, 0.8);
        assert!(!handle.is_looped);
        assert!(handle.is_playing);
    }

    #[tokio::test]
    async fn should_fail_with_invalid_volume() {
        let service: Arc<Mutex<dyn AudioService>> =
            Arc::new(Mutex::new(MockAudioService::new()));

        let cmd = PlayAudioCmd {
            asset_id: "test-asset".to_string(),
            volume: 1.5,
            looped: false,
            fade_in_ms: None,
        };

        let result = play_audio(&service, cmd).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_stop_audio_successfully() {
        let service: Arc<Mutex<dyn AudioService>> =
            Arc::new(Mutex::new(MockAudioService::new()));

        let play_cmd = PlayAudioCmd {
            asset_id: "test-asset".to_string(),
            volume: 0.5,
            looped: false,
            fade_in_ms: None,
        };
        let handle = play_audio(&service, play_cmd).await.unwrap();

        let stop_cmd = StopAudioCmd {
            playback_id: handle.id,
        };
        let result = stop_audio(&service, stop_cmd).await;
        assert!(result.is_ok());
    }
}
