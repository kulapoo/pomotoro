use domain::event_names;
use domain::*;
use leptos::prelude::{Get, Set};
use leptos::task::spawn_local;

use crate::components::error_toast::handle_command_error;
use crate::utils::invoke;

use super::SettingsViewModel;

impl SettingsViewModel {
    pub fn save_config(&self, config: Config) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;
        let set_error_state = self.set_error_state;

        set_is_saving.set(true);
        let config_clone = config.clone();

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct Args {
                config: Config,
            }

            let args = Args { config };

            // Debug: Log the serialized args
            if let Ok(js_val) = serde_wasm_bindgen::to_value(&args) {
                web_sys::console::log_2(&"Serialized args for SAVE_GLOBAL:".into(), &js_val);
            }

            invoke::<(), _>(event_names::config::SAVE_GLOBAL, Some(args)).await
                .map(|_| {
                    set_config.set(Some(config_clone));
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to save config: {}", e), set_error_state);
                })
                .ok();
            set_is_saving.set(false);
        });
    }

    pub fn update_general(&self, general: GeneralConfig) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;
        let set_error_state = self.set_error_state;

        set_is_saving.set(true);

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct Args {
                preferences: GeneralConfig,
            }

            invoke::<Config, _>(event_names::config::UPDATE_GENERAL, Some(Args { preferences: general })).await
                .map(|config| {
                    set_config.set(Some(config));
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to update general config: {}", e), set_error_state);
                })
                .ok();
            set_is_saving.set(false);
        });
    }

    pub fn update_notifications(&self, notifications: NotificationConfig) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;
        let set_error_state = self.set_error_state;

        set_is_saving.set(true);

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct Args {
                preferences: NotificationConfig,
            }

            invoke::<Config, _>(event_names::config::UPDATE_NOTIFICATIONS, Some(Args { preferences: notifications })).await
                .map(|config| {
                    set_config.set(Some(config));
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to update notifications: {}", e), set_error_state);
                })
                .ok();
            set_is_saving.set(false);
        });
    }

    pub fn update_appearance(&self, appearance: AppearanceConfig) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;
        let set_error_state = self.set_error_state;

        set_is_saving.set(true);

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct Args {
                preferences: AppearanceConfig,
            }

            invoke::<Config, _>(event_names::config::UPDATE_APPEARANCE, Some(Args { preferences: appearance })).await
                .map(|config| {
                    set_config.set(Some(config));
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to update appearance: {}", e), set_error_state);
                })
                .ok();
            set_is_saving.set(false);
        });
    }

    pub fn update_audio(&self, audio: AudioConfig) {
        if let Some(mut config) = self.config.get() {
            config.audio = audio;
            self.save_config(config);
        }
    }

    pub fn update_timer(&self, timer: TimerConfiguration) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;
        let set_error_state = self.set_error_state;

        set_is_saving.set(true);

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct Args {
                timer: TimerConfiguration,
            }

            invoke::<Config, _>(event_names::config::UPDATE_TIMINGS, Some(Args { timer })).await
                .map(|config| {
                    set_config.set(Some(config));
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to update timer config: {}", e), set_error_state);
                })
                .ok();
            set_is_saving.set(false);
        });
    }

    pub fn reset_to_defaults(&self) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;
        let set_error_state = self.set_error_state;

        set_is_saving.set(true);

        spawn_local(async move {
            invoke::<Config, ()>(event_names::config::RESET_TO_DEFAULTS, None).await
                .map(|config| {
                    set_config.set(Some(config));
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to reset config: {}", e), set_error_state);
                })
                .ok();
            set_is_saving.set(false);
        });
    }

    pub fn save_settings(&self) -> std::result::Result<(), String> {
        if let Some(config) = self.config.get() {
            if let Err(_) = config.validate() {
                return Err("Invalid configuration settings".to_string());
            }
            self.save_config(config);
            Ok(())
        } else {
            Err("No configuration loaded".to_string())
        }
    }

    pub fn test_audio_preview(&self, sound_type: &str) {
        let _sound_type = sound_type.to_string();
        spawn_local(async move {
            let _ = invoke::<(), ()>(event_names::commands::audio::TEST_PREVIEW, None).await;
        });
    }
}
