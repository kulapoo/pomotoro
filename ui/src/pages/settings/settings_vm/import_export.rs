use domain::event_names;
use domain::*;
use js_sys;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::utils::invoke;

use super::SettingsViewModel;

impl SettingsViewModel {
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
                                            #[derive(serde::Serialize)]
                                            struct Args {
                                                config: Config,
                                            }

                                            invoke::<(), _>(event_names::config::SAVE_GLOBAL, Some(Args { config })).await
                                                .ok();
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
}
