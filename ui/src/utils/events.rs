use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
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
