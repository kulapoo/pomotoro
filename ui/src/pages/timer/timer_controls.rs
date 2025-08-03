use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use domain::{TimerState, TimerStatus};
use domain::events;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn TimerControls(
    timer_state: ReadSignal<TimerState>,
    set_timer_state: WriteSignal<TimerState>,
) -> impl IntoView {
    let start_pause_timer = move |_| {
        let current_state = timer_state.get_untracked();
        spawn_local(async move {
            let command = match current_state.status() {
                TimerStatus::Running => events::timer::PAUSE,
                _ => events::timer::START,
            };
            let result = invoke(command, JsValue::NULL).await;
            if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                web_sys::console::log_1(&JsValue::from(state.get_phase_name()));
                set_timer_state.set(state);
            }
        });
    };

    let reset_timer = move |_| {
        spawn_local(async move {
            let result = invoke(events::timer::RESET, JsValue::NULL).await;
            if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                set_timer_state.set(state);
            }
        });
    };

    let skip_phase = move |_| {
        spawn_local(async move {
            let result = invoke(events::timer::SKIP_PHASE, JsValue::NULL).await;
            if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                set_timer_state.set(state);
            }
        });
    };

    view! {
        <>
            // Primary Start/Pause button
            <button
                class="btn btn-primary"
                on:click=start_pause_timer
            >
                {move || match timer_state.get().status() {
                    TimerStatus::Running => "Pause",
                    _ => "Start"
                }}
            </button>

            // Secondary Reset button
            <button
                class="btn btn-secondary"
                on:click=reset_timer
            >
                "Reset"
            </button>

            // Secondary Skip button
            <button
                class="btn btn-secondary"
                on:click=skip_phase
            >
                "Skip Task"
            </button>
        </>
    }
}