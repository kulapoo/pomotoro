use serde_json::{json, Value};
use domain::event_names::commands::audio as audio_commands;
use super::app_handle::MockAppHandle;

/// Audio UI actions
#[derive(Clone)]
pub struct AudioUiActions {
    app_handle: MockAppHandle,
}

impl AudioUiActions {
    pub fn new(app_handle: MockAppHandle) -> Self {
        Self { app_handle }
    }

    /// Test audio preview
    pub async fn test_preview(&self, sound_file: &str) -> Value {
        self.app_handle.emit(audio_commands::TEST_PREVIEW, json!({
            "file": sound_file
        })).unwrap();

        json!({
            "playing": true,
            "file": sound_file
        })
    }

    /// Play notification sound
    pub async fn play_notification(&self) -> Value {
        self.app_handle.emit(audio_commands::PLAY_NOTIFICATION, json!({})).unwrap();

        json!({
            "played": true,
            "type": "notification"
        })
    }

    /// Play background audio
    pub async fn play_background(&self, file: &str) -> Value {
        self.app_handle.emit(audio_commands::PLAY_BACKGROUND, json!({
            "file": file,
            "loop": true
        })).unwrap();

        json!({
            "playing": true,
            "background": file
        })
    }

    /// Stop background audio
    pub async fn stop_background(&self) -> Value {
        self.app_handle.emit(audio_commands::STOP_BACKGROUND, json!({})).unwrap();

        json!({
            "stopped": true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_actions() {
        let app_handle = MockAppHandle::new();
        let audio_actions = AudioUiActions::new(app_handle.clone());

        // Test audio preview
        let result = audio_actions.test_preview("bell.wav").await;
        assert_eq!(result["playing"], true);
        assert_eq!(result["file"], "bell.wav");

        // Test notification sound
        let result = audio_actions.play_notification().await;
        assert_eq!(result["played"], true);

        // Test background audio
        let result = audio_actions.play_background("ambient.mp3").await;
        assert_eq!(result["playing"], true);

        let result = audio_actions.stop_background().await;
        assert_eq!(result["stopped"], true);
    }
}