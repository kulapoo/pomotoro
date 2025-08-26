use domain::{TimerState, TimerTick, event_names};
use event_names::ui_listeners::timer as timer_event_names;
use leptos::prelude::WriteSignal;
use wasm_bindgen::prelude::*;
use js_sys;

use leptos::prelude::*;
use leptos::task::spawn_local;

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
    let result = invoke(command, args).await;
    
    // Check if the result is a string (which could be an error message)
    if result.is_string() {
        let result_str = result.as_string().unwrap_or_default();
        // Check if it looks like an error message (common patterns)
        if result_str.contains("Error") || 
           result_str.contains("failed") || 
           result_str.contains("not allowed") {
            web_sys::console::error_1(&format!(
                "Command '{}' failed: {}",
                command, result_str
            ).into());
            return Err(JsValue::from_str(&result_str));
        }
    }
    
    Ok(result)
}

pub async fn invoke_command_no_args(command: &str) -> Result<JsValue, JsValue> {
    invoke_command(command, JsValue::NULL).await
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
