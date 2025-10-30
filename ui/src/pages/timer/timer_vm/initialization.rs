use domain::{Task, Timer, event_names::{ui_listeners::timer as timer_event_names, commands, ui_listeners::task as task_event_names}};
use leptos::prelude::*;
use leptos::task::spawn_local;
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
        let set_active_task = self.set_active_task;
        let set_error_state = self.set_error_state;

        Effect::new(move |_| {
            // Load initial active task
            Self::setup_initial_active_task(
                set_active_task,
                set_error_state,
            );
            // Listen for active task changes only (timer state is handled by AppViewModel)
            Self::setup_active_task_listener(set_active_task);
        });
    }

    fn setup_initial_active_task(
        set_active_task: WriteSignal<Option<Task>>,
        set_error_state: WriteSignal<Option<ErrorInfo>>,
    ) {
        spawn_local(async move {
            // Get the timer to extract active task ID
            invoke::<Timer, ()>(commands::timer::GET_STATE, None).await
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok()
                .and_then(|timer| {
                    let active_task_id = timer.active_task_id().map(|id| id.to_string()).unwrap_or_default();

                    spawn_local(async move {
                        task_ops::fetch_task_by_id(&active_task_id, set_active_task).await;
                    });

                    Some(())
                });
        });
    }

    fn setup_active_task_listener(set_active_task: WriteSignal<Option<Task>>) {
        // Listen for active task changed events
        spawn_local(async move {
            let callback = Closure::new(move |_event: JsValue| {
                spawn_local(async move {
                    task_ops::fetch_active_task(set_active_task).await;
                });
            });

            listen(task_event_names::ACTIVE_CHANGED, &callback).await;
            callback.forget();
        });

        // Also listen for phase completed to update task progress
        spawn_local(async move {
            let callback = Closure::new(move |_event: JsValue| {
                spawn_local(async move {
                    task_ops::fetch_active_task(set_active_task).await;
                });
            });

            listen(timer_event_names::PHASE_COMPLETED, &callback).await;
            callback.forget();
        });
    }
}
