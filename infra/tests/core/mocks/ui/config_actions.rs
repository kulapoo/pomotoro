use serde_json::{json, Value};
use domain::event_names::commands::config as config_commands;
use super::app_handle::MockAppHandle;

/// Configuration UI actions
#[derive(Clone)]
pub struct ConfigUiActions {
    app_handle: MockAppHandle,
}

impl ConfigUiActions {
    pub fn new(app_handle: MockAppHandle) -> Self {
        Self { app_handle }
    }

    /// Get global configuration
    pub async fn get_config(&self) -> Value {
        self.app_handle.emit(config_commands::GET_GLOBAL, json!({})).unwrap();

        json!({
            "work_duration": 25,
            "short_break_duration": 5,
            "long_break_duration": 15,
            "sessions_until_long_break": 4,
            "theme": "dark",
            "sound_enabled": true,
            "notifications_enabled": true
        })
    }

    /// Update general configuration
    pub async fn update_general(&self, settings: Value) -> Value {
        self.app_handle.emit(config_commands::UPDATE_GENERAL, settings).unwrap();

        json!({
            "updated": true,
            "section": "general"
        })
    }

    /// Update notification settings
    pub async fn update_notifications(&self, enabled: bool) -> Value {
        self.app_handle.emit(config_commands::UPDATE_NOTIFICATIONS, json!({
            "enabled": enabled,
            "desktop": enabled,
            "sound": enabled
        })).unwrap();

        json!({
            "notifications_enabled": enabled
        })
    }

    /// Update appearance/theme
    pub async fn update_theme(&self, theme: &str) -> Value {
        self.app_handle.emit(config_commands::UPDATE_APPEARANCE, json!({
            "theme": theme,
            "font_size": "medium"
        })).unwrap();

        json!({
            "theme": theme,
            "applied": true
        })
    }

    /// Update audio settings
    pub async fn update_audio(&self, volume: f32, enabled: bool) -> Value {
        self.app_handle.emit(config_commands::UPDATE_AUDIO, json!({
            "volume": volume,
            "enabled": enabled,
            "sound_file": "default.wav"
        })).unwrap();

        json!({
            "audio_enabled": enabled,
            "volume": volume
        })
    }

    /// Reset to defaults
    pub async fn reset_to_defaults(&self) -> Value {
        self.app_handle.emit(config_commands::RESET_TO_DEFAULTS, json!({})).unwrap();

        json!({
            "reset": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    }

    /// Set a configuration value directly (for builder initialization)
    pub fn set_value(&self, key: &str, value: Value) -> Value {
        // Emit a config update event with the specific key-value pair
        self.app_handle.emit(config_commands::UPDATE_GENERAL, json!({
            key: value
        })).unwrap();

        json!({
            "key": key,
            "value": value,
            "set": true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_ui_actions() {
        let app_handle = MockAppHandle::new();
        let config_actions = ConfigUiActions::new(app_handle.clone());

        // Get config
        let config = config_actions.get_config().await;
        assert_eq!(config["work_duration"], 25);
        assert!(app_handle.was_event_emitted(config_commands::GET_GLOBAL));

        // Update theme
        let result = config_actions.update_theme("light").await;
        assert_eq!(result["theme"], "light");
        assert!(app_handle.was_event_emitted(config_commands::UPDATE_APPEARANCE));

        // Update notifications
        let result = config_actions.update_notifications(false).await;
        assert_eq!(result["notifications_enabled"], false);
    }
}