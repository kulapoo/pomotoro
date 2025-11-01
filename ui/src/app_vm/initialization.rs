use domain::TaskSwitchWorkflowCompleted;
use domain::event_names::ui_listeners::{timer as timer_event_names, task as task_event_names};
use domain::{Task, Timer, TimerState, TimerTick, event_names::commands};
use leptos::prelude::*;
use leptos::task::spawn_local;
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
        self.load_initial_active_task();
        self.setup_timer_listeners();
        self.setup_active_task_listeners();
    }

    fn load_initial_timer_state(&self) {
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;
        let set_active_task = self.set_active_task;

        spawn_local(async move {
            invoke::<Timer, ()>(commands::timer::GET_STATE, None).await
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok()
                .map(|timer| {
                    set_timer_state.set(timer.state().clone());

                    // Also load the task that this timer belongs to
                    let task_id = timer.task_id();
                    let task_id_str = task_id.to_string();
                    spawn_local(async move {
                        Self::fetch_task_by_id(&task_id_str, set_active_task).await;
                    });

                    ()
                });
        });
    }

    fn load_initial_active_task(&self) {
        let set_active_task = self.set_active_task;

        spawn_local(async move {
            // Try to get the active task
            Self::fetch_active_task(set_active_task).await;
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

    // Active Task management
    fn setup_active_task_listeners(&self) {
        let set_active_task = self.set_active_task;

        spawn_local(async move {
            let callback = Closure::new(move |event: JsValue| {
                let payload = js_sys::Reflect::get(&event, &"payload".into())
                    .unwrap_or(JsValue::NULL);

                if let Ok(task_switch) = serde_wasm_bindgen::from_value::<TaskSwitchWorkflowCompleted>(payload) {
                    spawn_local(async move {
                        Self::fetch_task_by_id(&task_switch.new_task_id.to_string(), set_active_task).await;
                    });
                }
            });

            listen(task_event_names::ACTIVE_CHANGED, &callback).await;
            callback.forget();
        });
    }

    // Helper methods for fetching tasks
    async fn fetch_active_task(set_active_task: WriteSignal<Option<Task>>) {
        let active_task = invoke::<Vec<Task>, ()>(commands::task::GET_ACTIVE, None).await
            .ok()
            .and_then(|tasks| tasks.into_iter().next());

        set_active_task.set(active_task);
    }

    async fn fetch_task_by_id(task_id: &str, set_active_task: WriteSignal<Option<Task>>) {
        use serde::Serialize;

        if task_id.is_empty() {
            set_active_task.set(None);
            return;
        }

        #[derive(Serialize)]
        struct GetTaskArgs {
            id: String,
        }

        let args = GetTaskArgs {
            id: task_id.to_string(),
        };

        let task = invoke::<Option<Task>, _>(commands::task::GET, Some(args)).await
            .ok()
            .flatten();

        set_active_task.set(task);
    }
}