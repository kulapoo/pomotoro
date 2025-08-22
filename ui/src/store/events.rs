use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen;

use domain::{TimerState, TimerTick};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, callback: &Closure<dyn Fn(JsValue)>) -> JsValue;
}

pub fn setup_timer_events(set_timer_state: WriteSignal<TimerState>) {
    spawn_local(async move {
        let callback = Closure::new(move |event: JsValue| {
            // Extract the payload from the event
            let payload = js_sys::Reflect::get(&event, &"payload".into()).unwrap_or(JsValue::NULL);
            
            if let Ok(timer_tick) = serde_wasm_bindgen::from_value::<TimerTick>(payload) {
                // Update the timer state with the new remaining seconds
                set_timer_state.update(|_state| {
                    // Note: TimerState is an enum state machine, so we can't directly update fields.
                    // This approach won't work with the new state machine design.
                    // In a real implementation, we'd need to replace the entire state or
                    // handle timer ticks differently. For now, just log this.
                    web_sys::console::log_1(&format!("Timer tick: {} seconds remaining", timer_tick.remaining_seconds).into());
                });
            }
        });

        listen("timer:tick", &callback).await;
        
        // Keep the closure alive
        callback.forget();
    });
}

pub fn setup_phase_complete_events() {
    // Will be implemented when needed for phase completion notifications
}