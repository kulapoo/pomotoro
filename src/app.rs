use leptos::task::spawn_local;
use leptos::prelude::*;
use leptos::context::Provider;
use wasm_bindgen::prelude::*;

use crate::store::{TimerState, setup_timer_events, setup_phase_complete_events, ConfigResource};
use pomotoro_domain::TimerStateWithTask;
use crate::components::{TimerDisplay, TimerControls, TaskList, TaskResource};
use pomotoro_domain::events;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn App() -> impl IntoView {
    let (timer_state, set_timer_state) = signal(TimerState::default());
    let (timer_with_task, set_timer_with_task) = signal(TimerStateWithTask::new(TimerState::default(), None));
    let config_resource = ConfigResource::new();
    let task_resource = TaskResource::new();
    
    // Connect task resource updates to timer state updates
    Effect::new(move |_| {
        // When active task changes, refetch timer state with task
        let _active_task = task_resource.active_task.get();
        spawn_local(async move {
            let result = invoke(events::timer::GET_STATE_WITH_TASK, JsValue::NULL).await;
            if let Ok(state_with_task) = serde_wasm_bindgen::from_value::<TimerStateWithTask>(result) {
                set_timer_with_task.set(state_with_task.clone());
                set_timer_state.set(state_with_task.timer_state);
            }
        });
    });

    Effect::new(move |_| {
        spawn_local(async move {
            let result = invoke(events::timer::GET_STATE_WITH_TASK, JsValue::NULL).await;
            if let Ok(state_with_task) = serde_wasm_bindgen::from_value::<TimerStateWithTask>(result) {
                set_timer_with_task.set(state_with_task.clone());
                set_timer_state.set(state_with_task.timer_state);
            }
        });
    });

    setup_timer_events(set_timer_state);
    setup_phase_complete_events();

    view! {
        <main class="app">
            <Provider value=config_resource>
                <div class="app-layout">
                    <TaskList task_resource=task_resource.clone() />
                    <div class="main-content">
                        <div class="timer-container">
                            <TimerDisplay 
                                timer_state=timer_state
                                timer_with_task=timer_with_task
                            />
                            <TimerControls
                                timer_state=timer_state
                                set_timer_state=set_timer_state
                            />
                        </div>
                    </div>
                </div>
            </Provider>
        </main>
    }
}
