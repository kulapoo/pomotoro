use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;

use super::timer_state::TimerState;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(event: &str, handler: &js_sys::Function) -> JsValue;
}

pub fn setup_timer_events(set_timer_state: WriteSignal<TimerState>) {
    Effect::new(move |_| {
        let timer_state_setter = set_timer_state;
        spawn_local(async move {
            let timer_update_callback = Closure::wrap(Box::new(move |event: JsValue| {
                if let Ok(payload) = js_sys::Reflect::get(&event, &JsValue::from_str("payload")) {
                    if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(payload) {
                        timer_state_setter.set(state);
                    }
                }
            }) as Box<dyn Fn(JsValue)>);
            
            let _ = listen("timer-update", timer_update_callback.as_ref().unchecked_ref()).await;
            
            timer_update_callback.forget();
        });
    });
}

pub fn setup_phase_complete_events() {
    Effect::new(move |_| {
        spawn_local(async move {
            let phase_complete_callback = Closure::wrap(Box::new(move |event: JsValue| {
                web_sys::console::log_1(&JsValue::from_str("Phase completed!"));
                if let Ok(payload) = js_sys::Reflect::get(&event, &JsValue::from_str("payload")) {
                    web_sys::console::log_2(&JsValue::from_str("Phase transition:"), &payload);
                }
            }) as Box<dyn Fn(JsValue)>);
            
            let _ = listen("phase-complete", phase_complete_callback.as_ref().unchecked_ref()).await;
            
            phase_complete_callback.forget();
        });
    });
}