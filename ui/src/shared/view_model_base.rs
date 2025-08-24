use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub trait ViewModel {
    type State: Clone + 'static;
    
    fn new() -> Self;
    fn state(&self) -> ReadSignal<Self::State>;
    fn set_state(&self) -> WriteSignal<Self::State>;
}

pub async fn invoke_command(command: &str, args: JsValue) -> Result<JsValue, JsValue> {
    Ok(invoke(command, args).await)
}

pub async fn invoke_command_no_args(command: &str) -> Result<JsValue, JsValue> {
    Ok(invoke(command, JsValue::NULL).await)
}