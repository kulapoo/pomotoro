use domain::event_names;
use domain::event_names::ui_listeners::config as config_event_names;
use domain::event_names::config::CONFIG_UPDATED_UI;
use domain::*;
use js_sys;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;

use crate::components::error_toast::{ErrorInfo, handle_command_error};
use crate::utils::invoke;

use super::SettingsViewModel;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(
        event: &str,
        callback: &Closure<dyn Fn(JsValue)>,
    ) -> JsValue;
}

impl SettingsViewModel {
    pub(super) fn initialize(&self) {
        self.load_config();
        self.setup_event_listeners();
    }

    fn load_config(&self) {
        let set_config = self.set_config;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            web_sys::console::log_1(&"Loading global config...".into());
            invoke::<Config, ()>(event_names::config::GET_GLOBAL, None).await
                .map(|config| {
                    web_sys::console::log_1(&"Successfully loaded config".into());
                    set_config.set(Some(config));
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(format!("Failed to load config: {}", e), set_error_state);
                })
                .ok();
        });
    }

    pub fn refetch_config(&self) {
        self.load_config();
    }

    fn setup_event_listeners(&self) {
        let set_config = self.set_config;
        let set_error_state = self.set_error_state;

        // Listen for ConfigUpdated events
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("ConfigUpdated event received: {:?}", payload).into(),
                );

                // Update the config when it changes
                from_value::<Config>(payload)
                    .ok()
                    .map(|new_config| set_config.set(Some(new_config)))
                    .unwrap_or_else(|| {
                        // If parsing fails, reload the config from backend
                        let set_config_clone = set_config;
                        let set_error_state_clone = set_error_state;
                        spawn_local(async move {
                            invoke::<Config, ()>(event_names::config::GET_GLOBAL, None).await
                                .map(|config| {
                                    set_config_clone.set(Some(config));
                                    // Clear any existing errors on success
                                    set_error_state_clone.set(None);
                                })
                                .map_err(|e| {
                                    handle_command_error(format!("Failed to reload config: {}", e), set_error_state_clone);
                                })
                                .ok();
                        });
                    });
            });

            listen(CONFIG_UPDATED_UI, &callback).await;
            callback.forget();
        });

        // Listen for ConfigReset events
        let set_config_for_reset = self.set_config;
        let set_error_state_for_reset = self.set_error_state;
        spawn_local(async move {
            let callback = Closure::new(move |_event: JsValue| {
                web_sys::console::log_1(&"ConfigReset event received".into());

                // Reload the default config
                let set_config_clone = set_config_for_reset;
                let set_error_state_clone = set_error_state_for_reset;
                spawn_local(async move {
                    invoke::<Config, ()>(event_names::config::GET_GLOBAL, None).await
                        .map(|config| {
                            set_config_clone.set(Some(config));
                            web_sys::console::log_1(&"Config reset to defaults".into());
                            // Clear any existing errors on success
                            set_error_state_clone.set(None);
                        })
                        .map_err(|e| {
                            handle_command_error(format!("Failed to reload config: {}", e), set_error_state_clone);
                        })
                        .ok();
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
                from_value::<String>(payload)
                    .ok()
                    .map(|theme_str| {
                        let theme = match theme_str.as_str() {
                            "Light" => Theme::Light,
                            "Dark" => Theme::Dark,
                            _ => Theme::System,
                        };

                        config_for_theme.get_untracked()
                            .map(|mut current_config| {
                                current_config.appearance.theme = theme;
                                set_config_for_theme.set(Some(current_config));
                            });
                    })
                    .unwrap_or_else(|| {
                        web_sys::console::error_1(&"Failed to parse theme change event".into());
                    });
            });

            listen(config_event_names::THEME_CHANGED, &callback).await;
            callback.forget();
        });
    }
}
