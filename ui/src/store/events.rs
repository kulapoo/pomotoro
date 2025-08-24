use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen;

use domain::{event_names::timer::TICK, TimerState, TimerTick};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, callback: &Closure<dyn Fn(JsValue)>) -> JsValue;
}

pub fn setup_timer_events(set_timer_state: WriteSignal<TimerState>) {
    spawn_local(async move {
        let callback = Closure::new(move |event: JsValue| {
            let payload = js_sys::Reflect::get(&event, &"payload".into()).unwrap_or(JsValue::NULL);

            if let Ok(timer_tick) = serde_wasm_bindgen::from_value::<TimerTick>(payload) {
                set_timer_state.update(|_state| {
                    web_sys::console::log_1(&format!("Timer tick: {} seconds remaining", timer_tick.remaining_seconds).into());
                });
            }
        });

        listen(TICK, &callback).await;

        callback.forget();
    });
}

pub fn setup_phase_complete_events() {
}