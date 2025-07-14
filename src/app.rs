use leptos::task::spawn_local;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::store::{TimerState, setup_timer_events, setup_phase_complete_events};
use crate::components::{TimerDisplay, TimerControls};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn App() -> impl IntoView {
    let (timer_state, set_timer_state) = signal(TimerState::default());

    Effect::new(move |_| {
        spawn_local(async move {
            let result = invoke("get_timer_state", JsValue::NULL).await;
            if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                set_timer_state.set(state);
            }
        });
    });

    setup_timer_events(set_timer_state);
    setup_phase_complete_events();

    view! {
        <main class="app">
            <div class="timer-container">
                <TimerDisplay timer_state=timer_state />
                <TimerControls
                    timer_state=timer_state
                    set_timer_state=set_timer_state
                />
            </div>
        </main>
    }
}
