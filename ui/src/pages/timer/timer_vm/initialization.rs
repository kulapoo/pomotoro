use domain::{Task, Timer, TimerState, event_names::{ui_listeners::timer as timer_event_names, commands}};
use domain::TimerTick;
use js_sys;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

use crate::components::{ErrorInfo, handle_command_error};
use crate::utils::invoke;

use super::TimerViewModel;
use super::task_ops;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(
        event: &str,
        callback: &Closure<dyn Fn(JsValue)>,
    ) -> JsValue;
}

// Initialization & Event Listeners
impl TimerViewModel {
    pub(super) fn initialize(&self) {
        let set_timer_state = self.set_timer_state;
        let set_active_task = self.set_active_task;
        let set_error_state = self.set_error_state;

        Effect::new(move |_| {
            Self::setup_initial_state(
                set_timer_state,
                set_active_task,
                set_error_state,
            );
            Self::setup_timer_tick_listener(set_timer_state);
            Self::setup_timer_status_changed_listener(
                set_timer_state,
                set_active_task,
            );
            Self::setup_phase_event_listeners(set_timer_state, set_active_task);
        });
    }

    fn setup_initial_state(
        set_timer_state: WriteSignal<TimerState>,
        set_active_task: WriteSignal<Option<Task>>,
        set_error_state: WriteSignal<Option<ErrorInfo>>,
    ) {
        spawn_local(async move {
            invoke::<Timer, ()>(commands::timer::GET_STATE, None).await
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok()
                .and_then(|timer| {
                    // Try extracting from timer object first

                    set_timer_state.set(timer.state().clone());
                    let active_task_id = timer.active_task_id().map(|id| id.to_string()).unwrap_or_default();

                    spawn_local(async move {
                        task_ops::fetch_task_by_id(&active_task_id, set_active_task).await;
                    });

                    Some(())
                });
        });
    }

    fn setup_timer_tick_listener(set_timer_state: WriteSignal<TimerState>) {
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                if let Ok(timer_tick) =
                    serde_wasm_bindgen::from_value::<TimerTick>(payload)
                {
                    set_timer_state.update(|state| {
                        *state = state.with_remaining_seconds(
                            timer_tick.remaining_seconds,
                        );
                    });
                }
            });

            listen(timer_event_names::TICK, &callback).await;

            callback.forget();
        });
    }

    fn setup_timer_status_changed_listener(
        set_timer_state: WriteSignal<TimerState>,
        set_active_task: WriteSignal<Option<Task>>,
    ) {
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                if let Ok(state) =
                    serde_wasm_bindgen::from_value::<TimerState>(payload)
                {
                    set_timer_state.set(state.clone());

                    // Fetch updated active task info
                    let set_active_task_clone = set_active_task;
                    spawn_local(async move {
                        task_ops::fetch_active_task(set_active_task_clone).await;
                    });
                }
            });

            listen(timer_event_names::STATUS_CHANGED, &callback).await;

            callback.forget();
        });
    }

    fn setup_phase_event_listeners(
        set_timer_state: WriteSignal<TimerState>,
        _set_active_task: WriteSignal<Option<Task>>,
    ) {
        // Listen for phase completed events
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                // Log phase completion for debugging
                web_sys::console::log_1(
                    &format!("Phase completed event received: {:?}", payload)
                        .into(),
                );

                // Update timer state if provided in the event
                if let Ok(state) =
                    serde_wasm_bindgen::from_value::<TimerState>(payload)
                {
                    set_timer_state.set(state);
                }
            });

            listen(timer_event_names::PHASE_COMPLETED, &callback).await;
            callback.forget();
        });

        // Listen for phase skipped events
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                // Log phase skip for debugging
                web_sys::console::log_1(
                    &format!("Phase skipped event received: {:?}", payload)
                        .into(),
                );

                // Update timer state if provided in the event
                if let Ok(state) =
                    serde_wasm_bindgen::from_value::<TimerState>(payload)
                {
                    set_timer_state.set(state);
                }
            });

            listen(timer_event_names::PHASE_SKIPPED, &callback).await;
            callback.forget();
        });

        // Listen for status changed events
        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                web_sys::console::log_1(
                    &format!("Timer status changed: {:?}", payload).into(),
                );

                if let Ok(state) =
                    serde_wasm_bindgen::from_value::<TimerState>(payload)
                {
                    set_timer_state.set(state);
                }
            });

            listen(timer_event_names::STATUS_CHANGED, &callback).await;
            callback.forget();
        });
    }
}
