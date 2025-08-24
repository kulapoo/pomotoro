use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use domain::*;
use domain::events;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub async fn get_global_config() -> std::result::Result<Config, String> {
    let result = invoke(events::config::GET_GLOBAL, JsValue::NULL).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to deserialize Config: {e}"))
}

#[allow(dead_code)]
pub async fn save_global_config(config: Config) -> std::result::Result<(), String> {
    let args = to_value(&config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;

    let result = invoke(events::config::SAVE_GLOBAL, args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to save Config: {e}"))
}

#[allow(dead_code)]
pub async fn update_general(general: GeneralConfig) -> std::result::Result<Config, String> {
    let args = to_value(&general)
        .map_err(|e| format!("Failed to serialize general Config: {e}"))?;

    let result = invoke(events::config::UPDATE_GENERAL, args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update general Config: {e}"))
}

#[allow(dead_code)]
pub async fn update_notification_preferences(preferences: NotificationConfig) -> std::result::Result<Config, String> {
    let args = to_value(&preferences)
        .map_err(|e| format!("Failed to serialize notification preferences: {e}"))?;

    let result = invoke(events::config::UPDATE_NOTIFICATIONS, args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update notification preferences: {e}"))
}

#[allow(dead_code)]
pub async fn update_appearance(appearance: AppearanceConfig) -> std::result::Result<Config, String> {
    let args = to_value(&appearance)
        .map_err(|e| format!("Failed to serialize appearance: {e}"))?;

    let result = invoke(events::config::UPDATE_APPEARANCE, args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update appearance: {e}"))
}

#[allow(dead_code)]
pub async fn update_audio_config(audio_config: AudioConfig) -> std::result::Result<Config, String> {
    let args = to_value(&audio_config)
        .map_err(|e| format!("Failed to serialize audio config: {e}"))?;

    let result = invoke(events::config::UPDATE_AUDIO, args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update audio config: {e}"))
}

#[allow(dead_code)]
pub async fn update_default_timings(
    work_minutes: u32,
    short_break_minutes: u32,
    long_break_minutes: u32,
) -> std::result::Result<Config, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "workMinutes": work_minutes,
        "shortBreakMinutes": short_break_minutes,
        "longBreakMinutes": long_break_minutes,
    })).map_err(|e| format!("Failed to serialize timing args: {e}"))?;

    let result = invoke(events::config::UPDATE_TIMINGS, args).await;

    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to update default timings: {e}"))
}

#[allow(dead_code)]
pub async fn reset_global_config_to_defaults() -> std::result::Result<Config, String> {
    let result = invoke(events::config::RESET_TO_DEFAULTS, JsValue::NULL).await;

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
    pub async fn update_and_refetch<F, R>(&self, update_fn: F) -> std::result::Result<(), String>
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

