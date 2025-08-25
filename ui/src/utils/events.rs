use domain::{TimerState, TimerTick, event_names};
use event_names::ui_listeners::timer as timer_event_names;
use leptos::prelude::WriteSignal;
use wasm_bindgen::prelude::*;

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    async fn listen(
        event: &str,
        callback: &Closure<dyn Fn(JsValue)>,
    ) -> JsValue;
}

pub async fn invoke_command(
    command: &str,
    args: JsValue,
) -> Result<JsValue, JsValue> {
    Ok(invoke(command, args).await)
}

pub async fn invoke_command_no_args(command: &str) -> Result<JsValue, JsValue> {
    Ok(invoke(command, JsValue::NULL).await)
}

pub fn setup_timer_events(set_timer_state: WriteSignal<TimerState>) {
    spawn_local(async move {
        let callback = Closure::new(move |event: JsValue| {
            let payload = js_sys::Reflect::get(&event, &"payload".into())
                .unwrap_or(JsValue::NULL);

            if let Ok(timer_tick) =
                serde_wasm_bindgen::from_value::<TimerTick>(payload)
            {
                set_timer_state.update(|_state| {
                    web_sys::console::log_1(
                        &format!(
                            "Timer tick: {} seconds remaining",
                            timer_tick.remaining_seconds
                        )
                        .into(),
                    );
                });
            }
        });

        listen(timer_event_names::TICK, &callback).await;

        callback.forget();
    });
}

pub fn setup_phase_complete_events() {
    // Implement phase complete events
}
