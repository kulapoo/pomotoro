use domain::event_names::ui_listeners::timer as timer_event_names;
use domain::{Timer, TimerState, TimerTick, event_names::commands};
use js_sys;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

use crate::components::handle_command_error;
use crate::utils::invoke;

use super::AppViewModel;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(
        event: &str,
        callback: &Closure<dyn Fn(JsValue)>,
    ) -> JsValue;
}

impl AppViewModel {
    pub(super) fn initialize(&self) {
        self.load_initial_timer_state();
        self.setup_timer_listeners();
    }

    fn load_initial_timer_state(&self) {
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            invoke::<Timer, ()>(commands::timer::GET_STATE, None).await
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok()
                .and_then(|timer| {
                    set_timer_state.set(timer.state().clone());
                    Some(())
                });
        });
    }

    fn setup_timer_listeners(&self) {
        self.setup_timer_tick_listener();
        self.setup_timer_status_listener();
        self.setup_phase_completed_listener();
        self.setup_phase_skipped_listener();
    }

    fn setup_timer_tick_listener(&self) {
        let set_timer_state = self.set_timer_state;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                if let Ok(timer_tick) = serde_wasm_bindgen::from_value::<TimerTick>(payload) {
                    set_timer_state.update(|state| {
                        *state = (timer_tick.phase, timer_tick.remaining_seconds).into();
                    });
                }
            });

            listen(timer_event_names::TICK, &callback).await;
            callback.forget();
        });
    }

    fn setup_timer_status_listener(&self) {
        let set_timer_state = self.set_timer_state;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(payload) {
                    set_timer_state.set(state);
                }
            });

            listen(timer_event_names::STATUS_CHANGED, &callback).await;
            callback.forget();
        });
    }

    fn setup_phase_completed_listener(&self) {
        let set_timer_state = self.set_timer_state;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                // Log phase completion for debugging
                web_sys::console::log_1(&"App: Phase completed".into());

                if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(payload) {
                    set_timer_state.set(state);
                }
            });

            listen(timer_event_names::PHASE_COMPLETED, &callback).await;
            callback.forget();
        });
    }

    fn setup_phase_skipped_listener(&self) {
        let set_timer_state = self.set_timer_state;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                // Log phase skip for debugging
                web_sys::console::log_1(&"App: Phase skipped".into());

                if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(payload) {
                    set_timer_state.set(state);
                }
            });

            listen(timer_event_names::PHASE_SKIPPED, &callback).await;
            callback.forget();
        });
    }
}