use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use crate::store::{setup_timer_events, setup_phase_complete_events};
use domain::{TimerState, event_names};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub struct TimerPageState {
    pub timer_state: ReadSignal<TimerState>,
    pub set_timer_state: WriteSignal<TimerState>,
}

impl TimerPageState {
    pub fn new() -> Self {
        let (timer_state, set_timer_state) = signal(TimerState::default());

        Effect::new(move |_| {
            spawn_local(async move {
                let result = invoke(event_names::timer::GET_STATE, JsValue::NULL).await;
                if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                    set_timer_state.set(state);
                }
            });
        });
        setup_timer_events(set_timer_state);
        setup_phase_complete_events();

        Self {
            timer_state,
            set_timer_state,
        }
    }
}