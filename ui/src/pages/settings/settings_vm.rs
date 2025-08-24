use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::to_value;
use domain::*;
use domain::event_names;
use crate::shared::{invoke_command, invoke_command_no_args, ViewModel};

pub struct SettingsViewModel {
    config: ReadSignal<Option<Config>>,
    set_config: WriteSignal<Option<Config>>,
    is_saving: ReadSignal<bool>,
    set_is_saving: WriteSignal<bool>,
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
            if let Ok(result) = invoke_command_no_args(event_names::config::GET_GLOBAL).await {
                if let Ok(config) = serde_wasm_bindgen::from_value::<Config>(result) {
                    set_config.set(Some(config));
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
                if let Ok(result) = invoke_command(event_names::config::SAVE_GLOBAL, args).await {
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
                if let Ok(result) = invoke_command(event_names::config::UPDATE_GENERAL, args).await {
                    if let Ok(config) = serde_wasm_bindgen::from_value::<Config>(result) {
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
                if let Ok(result) = invoke_command(event_names::config::UPDATE_NOTIFICATIONS, args).await {
                    if let Ok(config) = serde_wasm_bindgen::from_value::<Config>(result) {
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
                if let Ok(result) = invoke_command(event_names::config::UPDATE_APPEARANCE, args).await {
                    if let Ok(config) = serde_wasm_bindgen::from_value::<Config>(result) {
                        set_config.set(Some(config));
                    }
                }
            }
            set_is_saving.set(false);
        });
    }

    pub fn update_audio(&self, audio: AudioConfig) {
        let set_config = self.set_config;
        let set_is_saving = self.set_is_saving;

        set_is_saving.set(true);

        spawn_local(async move {
            if let Ok(args) = to_value(&audio) {
                if let Ok(result) = invoke_command(event_names::config::UPDATE_AUDIO, args).await {
                    if let Ok(config) = serde_wasm_bindgen::from_value::<Config>(result) {
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
            if let Ok(result) = invoke_command_no_args(event_names::config::RESET_TO_DEFAULTS).await {
                if let Ok(config) = serde_wasm_bindgen::from_value::<Config>(result) {
                    set_config.set(Some(config));
                }
            }
            set_is_saving.set(false);
        });
    }

    pub fn refetch_config(&self) {
        self.load_config();
    }
}