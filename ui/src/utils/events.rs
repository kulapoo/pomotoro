use wasm_bindgen::prelude::*;
use serde::Serialize;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub async fn invoke_command(
    command: &str,
    args: JsValue,
) -> Result<JsValue, JsValue> {
    web_sys::console::log_1(&format!("INVOKE: Calling command '{}' with args: {:?}", command, args).into());
    
    let result = tauri_invoke(command, args).await;
    
    web_sys::console::log_1(&format!("INVOKE: Command '{}' returned: {:?}", command, result).into());

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
    // Create an empty object instead of NULL for Tauri v2
    let args = js_sys::Object::new();
    invoke_command(command, args.into()).await
}

/// Helper function to invoke a command with a single named parameter
/// This ensures the argument is properly wrapped in an object with the correct field name
pub async fn invoke_with_param(
    command: &str,
    param_name: &str,
    value: JsValue,
) -> Result<JsValue, JsValue> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str(param_name), &value)
        .map_err(|_| JsValue::from_str("Failed to set parameter"))?;

    web_sys::console::log_1(&format!(
        "INVOKE_WITH_PARAM: Calling '{}' with param '{}': {:?}",
        command, param_name, value
    ).into());

    invoke_command(command, args.into()).await
}

/// Generic invoke function that automatically handles serialization
/// Use this instead of manually calling serde_wasm_bindgen::to_value
pub async fn invoke<T>(command: &str, args: T) -> Result<JsValue, JsValue>
where
    T: Serialize,
{
    web_sys::console::log_1(&format!("INVOKE: Preparing to call command '{}'", command).into());

    match serde_wasm_bindgen::to_value(&args) {
        Ok(args_value) => {
            invoke_command(command, args_value).await
        }
        Err(err) => {
            let error_msg = format!("Failed to serialize arguments for command '{}': {:?}", command, err);
            web_sys::console::error_1(&JsValue::from_str(&error_msg));
            Err(JsValue::from_str(&error_msg))
        }
    }
}
