use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// Unified invoke function that handles serialization, deserialization, and logging
///
/// # Arguments
/// * `cmd` - The Tauri command to invoke
/// * `args` - Optional arguments to pass to the command
///
/// # Returns
/// * `Result<T, String>` - The deserialized result or an error message
///
/// # Example
/// ```
/// // Without arguments:
/// let state: TimerState = invoke(commands::timer::GET_STATE, None::<()>).await?;
///
/// // With arguments:
/// let task: Task = invoke(commands::task::GET_BY_ID, Some(&task_id)).await?;
/// ```
pub async fn invoke<T, A>(cmd: &str, args: Option<A>) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
    A: Serialize,
{
    // Log the invocation
    web_sys::console::log_1(&format!("Invoking command: {}", cmd).into());

    // Prepare arguments
    let js_args = match args {
        Some(a) => serde_wasm_bindgen::to_value(&a).map_err(|e| {
            format!("Failed to serialize args for '{}': {:?}", cmd, e)
        })?,
        None => js_sys::Object::new().into(),
    };

    // Call the Tauri command
    let result = tauri_invoke(cmd, js_args).await;

    // Check for string errors (Tauri v2 error format)
    if result.is_string() {
        let result_str = result.as_string().unwrap_or_default();
        if result_str.contains("Error")
            || result_str.contains("failed")
            || result_str.contains("not allowed")
        {
            web_sys::console::error_1(
                &format!("Command '{}' failed: {}", cmd, result_str).into(),
            );
            return Err(result_str);
        }
    }

    // Handle unit "()" return type - check if result is null or empty string
    if result.is_null()
        || result.is_undefined()
        || (result.is_string()
            && result.as_string().unwrap_or_default().is_empty())
    {
        // Try to deserialize from null - this works for unit type ()
        return serde_wasm_bindgen::from_value(JsValue::NULL).map_err(|e| {
            let error_msg =
                format!("Failed to parse empty result from '{}': {:?}", cmd, e);
            web_sys::console::error_1(&error_msg.clone().into());
            error_msg
        });
    }

    // Deserialize the result
    serde_wasm_bindgen::from_value(result).map_err(|e| {
        let error_msg =
            format!("Failed to parse result from '{}': {:?}", cmd, e);
        web_sys::console::error_1(&error_msg.clone().into());
        error_msg
    })
}
