use domain::event_names::commands;
use domain::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::utils::{invoke, invoke_command_no_args};

pub async fn get_global_config() -> std::result::Result<Config, String> {
    let result = invoke_command_no_args(commands::config::GET_GLOBAL).await
        .map_err(|e| format!("Failed to get config: {:?}", e))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to deserialize Config: {e}"))
}

#[allow(dead_code)]
pub async fn save_global_config(
    config: Config,
) -> std::result::Result<(), String> {
    let result = invoke(commands::config::SAVE_GLOBAL, config).await
        .map_err(|e| format!("Failed to save config: {:?}", e))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to save Config: {e}"))
}

#[allow(dead_code)]
pub async fn update_general(
    general: GeneralConfig,
) -> std::result::Result<Config, String> {
    let result = invoke(commands::config::UPDATE_GENERAL, general).await
        .map_err(|e| format!("Failed to update general config: {:?}", e))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update general Config: {e}"))
}

#[allow(dead_code)]
pub async fn update_notification_preferences(
    preferences: NotificationConfig,
) -> std::result::Result<Config, String> {
    let result = invoke(commands::config::UPDATE_NOTIFICATIONS, preferences).await
        .map_err(|e| format!("Failed to update notification preferences: {:?}", e))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update notification preferences: {e}"))
}

#[allow(dead_code)]
pub async fn update_appearance(
    appearance: AppearanceConfig,
) -> std::result::Result<Config, String> {
    let result = invoke(commands::config::UPDATE_APPEARANCE, appearance).await
        .map_err(|e| format!("Failed to update appearance: {:?}", e))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update appearance: {e}"))
}

#[allow(dead_code)]
pub async fn update_audio_config(
    audio_config: AudioConfig,
) -> std::result::Result<Config, String> {
    let result = invoke(commands::config::UPDATE_AUDIO, audio_config).await
        .map_err(|e| format!("Failed to update audio config: {:?}", e))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update audio config: {e}"))
}

#[allow(dead_code)]
pub async fn update_default_timings(
    work_minutes: u32,
    short_break_minutes: u32,
    long_break_minutes: u32,
) -> std::result::Result<Config, String> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct UpdateTimingsArgs {
        #[serde(rename = "workMinutes")]
        work_minutes: u32,
        #[serde(rename = "shortBreakMinutes")]
        short_break_minutes: u32,
        #[serde(rename = "longBreakMinutes")]
        long_break_minutes: u32,
    }

    let args = UpdateTimingsArgs {
        work_minutes,
        short_break_minutes,
        long_break_minutes,
    };

    let result = invoke(commands::config::UPDATE_TIMINGS, args).await
        .map_err(|e| format!("Failed to update timings: {:?}", e))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update default timings: {e}"))
}

#[allow(dead_code)]
pub async fn reset_global_config_to_defaults()
-> std::result::Result<Config, String> {
    let result =
        invoke_command_no_args(commands::config::RESET_TO_DEFAULTS).await
        .map_err(|e| format!("Failed to reset config: {:?}", e))?;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to reset config: {e}"))
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct ConfigResource {
    pub config: ReadSignal<Option<Config>>,
    set_config: WriteSignal<Option<Config>>,
}

impl ConfigResource {
    pub fn new() -> Self {
        let (config, set_config) = signal(None::<Config>);

        let set_config_clone = set_config;
        spawn_local(async move {
            if let Ok(initial_config) = get_global_config().await {
                set_config_clone.set(Some(initial_config));
            }
        });

        Self { config, set_config }
    }

    #[allow(dead_code)]
    pub async fn update_and_refetch<F, R>(
        &self,
        update_fn: F,
    ) -> std::result::Result<(), String>
    where
        F: FnOnce() -> R + 'static,
        R: std::future::Future<Output = std::result::Result<Config, String>>,
    {
        let new_config = update_fn().await?;
        self.set_config.set(Some(new_config));
        Ok(())
    }

    #[allow(dead_code)]
    pub fn refetch(&self) {
        let set_config = self.set_config;
        spawn_local(async move {
            if let Ok(config) = get_global_config().await {
                set_config.set(Some(config));
            }
        });
    }
}
