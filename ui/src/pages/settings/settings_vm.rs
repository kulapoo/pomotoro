use domain::event_names;
use domain::event_names::commands::{audio, storage};
use domain::event_names::ui_listeners::config as config_event_names;
use domain::event_names::config::CONFIG_UPDATED_UI;
use domain::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys;

use crate::utils::{ViewModel, invoke_command, invoke_command_no_args};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(
        event: &str,
        callback: &Closure<dyn Fn(JsValue)>,
    ) -> JsValue;
}

pub struct SettingsViewModel {
    pub config: ReadSignal<Option<Config>>,
    pub set_config: WriteSignal<Option<Config>>,
    pub is_saving: ReadSignal<bool>,
    pub set_is_saving: WriteSignal<bool>,
}

impl ViewModel for SettingsViewModel {
    type State = Option<Config>;

    fn new() -> Self {
        let (config, set_config) = signal(None::<Config>);
        let (is_saving, set_is_saving) = signal(false);

        let vm = Self {
            config,
            set_config,
            is_saving,
            set_is_saving,
        };

        vm.load_config();
        vm.setup_event_listeners();
        vm
    }

    fn state(&self) -> ReadSignal<Self::State> {
        self.config
    }

    fn set_state(&self) -> WriteSignal<Self::State> {
        self.set_config
    }
}

impl SettingsViewModel {
    fn load_config(&self) {
        let set_config = self.set_config;

        spawn_local(async move {
            web_sys::console::log_1(&"Loading global config...".into());
            match invoke_command_no_args(event_names::config::GET_GLOBAL).await {
                Ok(result) => {
                    web_sys::console::log_1(&format!("Got config result: {:?}", result).into());
                    match serde_wasm_bindgen::from_value::<Config>(result) {
                        Ok(config) => {
                            web_sys::console::log_1(&"Successfully parsed config".into());
                            set_config.set(Some(config));
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("Failed to parse config: {:?}", e).into());
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get config: {:?}", e).into());
                }
            }
        });
    }

    pub fn get_config(&self) -> Option<Config> {
        self.config.get()
    }

    pub fn is_saving(&self) -> bool {
        self.is_saving.get()
    }

    pub fn save_config(&self, config: Config) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;

        set_is_saving.set(true);

        spawn_local(async move {
            if let Ok(args) = to_value(&config) {
                if let Ok(result) =
                    invoke_command(event_names::config::SAVE_GLOBAL, args).await
                {
                    if serde_wasm_bindgen::from_value::<()>(result).is_ok() {
                        set_config.set(Some(config));
                    }
                }
            }
            set_is_saving.set(false);
        });
    }

    pub fn update_general(&self, general: GeneralConfig) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;

        set_is_saving.set(true);

        spawn_local(async move {
            if let Ok(args) = to_value(&general) {
                if let Ok(result) =
                    invoke_command(event_names::config::UPDATE_GENERAL, args)
                        .await
                {
                    if let Ok(config) =
                        serde_wasm_bindgen::from_value::<Config>(result)
                    {
                        set_config.set(Some(config));
                    }
                }
            }
            set_is_saving.set(false);
        });
    }

    pub fn update_notifications(&self, notifications: NotificationConfig) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;

        set_is_saving.set(true);

        spawn_local(async move {
            if let Ok(args) = to_value(&notifications) {
                if let Ok(result) = invoke_command(
                    event_names::config::UPDATE_NOTIFICATIONS,
                    args,
                )
                .await
                {
                    if let Ok(config) =
                        serde_wasm_bindgen::from_value::<Config>(result)
                    {
                        set_config.set(Some(config));
                    }
                }
            }
            set_is_saving.set(false);
        });
    }

    pub fn update_appearance(&self, appearance: AppearanceConfig) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;

        set_is_saving.set(true);

        spawn_local(async move {
            if let Ok(args) = to_value(&appearance) {
                if let Ok(result) =
                    invoke_command(event_names::config::UPDATE_APPEARANCE, args)
                        .await
                {
                    if let Ok(config) =
                        serde_wasm_bindgen::from_value::<Config>(result)
                    {
                        set_config.set(Some(config));
                    }
                }
            }
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

        set_is_saving.set(true);

        spawn_local(async move {
            if let Ok(args) = to_value(&timer) {
                if let Ok(result) =
                    invoke_command(event_names::config::UPDATE_TIMINGS, args)
                        .await
                {
                    if let Ok(config) =
                        serde_wasm_bindgen::from_value::<Config>(result)
                    {
                        set_config.set(Some(config));
                    }
                }
            }
            set_is_saving.set(false);
        });
    }

    pub fn reset_to_defaults(&self) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;

        set_is_saving.set(true);

        spawn_local(async move {
            if let Ok(result) =
                invoke_command_no_args(event_names::config::RESET_TO_DEFAULTS)
                    .await
            {
                if let Ok(config) =
                    serde_wasm_bindgen::from_value::<Config>(result)
                {
                    set_config.set(Some(config));
                }
            }
            set_is_saving.set(false);
        });
    }

    pub fn refetch_config(&self) {
        self.load_config();
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


    pub fn export_settings(&self) {
        if let Some(config) = self.config.get() {
            if let Ok(json) = serde_json::to_string_pretty(&config) {
                let blob = web_sys::Blob::new_with_str_sequence(
                    &js_sys::Array::of1(&json.into()),
                ).unwrap();
                let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
                
                let document = leptos::prelude::document();
                let a = document.create_element("a").unwrap();
                a.set_attribute("href", &url).unwrap();
                a.set_attribute("download", "pomotoro_settings.json").unwrap();
                let html_element = a.dyn_into::<web_sys::HtmlElement>().unwrap();
                html_element.click();
                
                web_sys::Url::revoke_object_url(&url).unwrap();
            }
        }
    }

    pub fn import_settings(&self) -> std::result::Result<(), String> {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;
        let document = leptos::prelude::document();
        let input = document.create_element("input").unwrap();
        input.set_attribute("type", "file").unwrap();
        input.set_attribute("accept", ".json").unwrap();
        
        let input_element = input.dyn_into::<web_sys::HtmlInputElement>().unwrap();
        
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let reader = web_sys::FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    
                    let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                        if let Ok(result) = reader_clone.result() {
                            if let Some(text) = result.as_string() {
                                if let Ok(config) = serde_json::from_str::<Config>(&text) {
                                    if config.validate().is_ok() {
                                        // Update local state
                                        set_config.set(Some(config.clone()));
                                        
                                        // Save to backend
                                        set_is_saving.set(true);
                                        spawn_local(async move {
                                            if let Ok(args) = to_value(&config) {
                                                let _ = invoke_command(event_names::config::SAVE_GLOBAL, args).await;
                                            }
                                            set_is_saving.set(false);
                                        });
                                    }
                                }
                            }
                        }
                    }) as Box<dyn FnMut(_)>);
                    
                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    reader.read_as_text(&file).unwrap();
                    onload.forget();
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        input_element.set_onchange(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        input_element.click();
        
        Ok(())
    }

    pub fn test_audio_preview(&self, sound_type: &str) {
        let _sound_type = sound_type.to_string();
        spawn_local(async move {
            let _ = invoke_command_no_args(audio::TEST_PREVIEW).await;
        });
    }

    pub fn get_storage_path(&self) -> String {
        String::from("/home/user/.config/pomotoro")
    }

    pub fn browse_for_directory(&self) -> Option<String> {
        None
    }

    pub fn validate_storage_path(&self, path: &str) -> std::result::Result<(), String> {
        if path.is_empty() {
            return Err("Path cannot be empty".to_string());
        }
        if !path.starts_with('/') && !path.starts_with("C:\\") {
            return Err("Path must be absolute".to_string());
        }
        Ok(())
    }

    pub fn update_storage_path(&self, _path: String) {
        
    }

    pub fn open_data_directory(&self) {
        spawn_local(async move {
            let _ = invoke_command_no_args(storage::OPEN_DATA_DIR).await;
        });
    }

    pub fn clear_all_data(&self) {
        spawn_local(async move {
            let _ = invoke_command_no_args(storage::CLEAR_ALL_DATA).await;
        });
    }

    fn setup_event_listeners(&self) {
        let set_config = self.set_config;

        // Listen for ConfigUpdated events
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("ConfigUpdated event received: {:?}", payload).into(),
                );

                // Update the config when it changes
                if let Ok(new_config) = from_value::<Config>(payload) {
                    set_config.set(Some(new_config));
                } else {
                    // If parsing fails, reload the config from backend
                    let set_config_clone = set_config;
                    spawn_local(async move {
                        match invoke_command_no_args(event_names::config::GET_GLOBAL).await {
                            Ok(result) => {
                                if let Ok(config) = from_value::<Config>(result) {
                                    set_config_clone.set(Some(config));
                                }
                            }
                            Err(e) => {
                                web_sys::console::error_1(&format!("Failed to reload config: {:?}", e).into());
                            }
                        }
                    });
                }
            });

            listen(CONFIG_UPDATED_UI, &callback).await;
            callback.forget();
        });

        // Listen for ConfigReset events
        let set_config_for_reset = self.set_config;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                web_sys::console::log_1(&"ConfigReset event received".into());

                // Reload the default config
                let set_config_clone = set_config_for_reset;
                spawn_local(async move {
                    match invoke_command_no_args(event_names::config::GET_GLOBAL).await {
                        Ok(result) => {
                            if let Ok(config) = from_value::<Config>(result) {
                                set_config_clone.set(Some(config));
                                web_sys::console::log_1(&"Config reset to defaults".into());
                            }
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("Failed to reload config after reset: {:?}", e).into());
                        }
                    }
                });
            });

            // Assuming there's a CONFIG_RESET event name
            listen("config_reset", &callback).await;
            callback.forget();
        });

        // Listen for theme changes
        let config_for_theme = self.config;
        let set_config_for_theme = self.set_config;
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("Theme changed event received: {:?}", payload).into(),
                );

                // Update just the theme in the config
                if let Ok(theme_str) = from_value::<String>(payload) {
                    let theme = match theme_str.as_str() {
                        "Light" => Theme::Light,
                        "Dark" => Theme::Dark,
                        _ => Theme::System,
                    };

                    if let Some(mut current_config) = config_for_theme.get_untracked() {
                        current_config.appearance.theme = theme;
                        set_config_for_theme.set(Some(current_config));
                    }
                }
            });

            listen(config_event_names::THEME_CHANGED, &callback).await;
            callback.forget();
        });
    }
}
