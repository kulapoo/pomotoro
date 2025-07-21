use leptos::prelude::*;
use leptos::callback::Callback;
use leptos::task::spawn_local;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use pomotoro_domain::events;
use crate::components::TaskResource;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(serde::Serialize)]
struct CreateTaskRequest {
    name: String,
    description: Option<String>,
    max_sessions: u8,
    tags: Vec<String>,
    config: Option<pomotoro_domain::TaskConfig>,
    audio_config: Option<pomotoro_domain::AudioConfig>,
}

#[component]
pub fn TaskCreationForm(
    task_resource: TaskResource,
    on_close: Callback<()>,
) -> impl IntoView {
    let (task_name, set_task_name) = signal(String::new());
    let (task_description, set_task_description) = signal(String::new());
    let (max_sessions, set_max_sessions) = signal(1u8);
    let (tags_input, set_tags_input) = signal(String::new());
    let (is_creating, set_is_creating) = signal(false);
    
    let create_task = move |_| {
        let task_name = task_name.get();
        let task_description = task_description.get();
        let max_sessions = max_sessions.get();
        let tags_input = tags_input.get();
        
        if task_name.trim().is_empty() {
            return;
        }
        
        let tags: Vec<String> = tags_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let description = if task_description.trim().is_empty() {
            None
        } else {
            Some(task_description.trim().to_string())
        };
        
        let request = CreateTaskRequest {
            name: task_name.trim().to_string(),
            description,
            max_sessions,
            tags,
            config: None,
            audio_config: None,
        };
        
        set_is_creating.set(true);
        
        let task_resource_clone = task_resource.clone();
        spawn_local(async move {
            match to_value(&request) {
                Ok(args) => {
                    web_sys::console::log_1(&format!("Creating task with request: {:?}", &request.name).into());
                    let result = invoke(events::task::CREATE, args).await;
                    
                    // Try parsing as Task directly first (Tauri success case)
                    match serde_wasm_bindgen::from_value::<pomotoro_domain::Task>(result.clone()) {
                        Ok(task) => {
                            web_sys::console::log_1(&format!("Task created successfully: {}", task.name).into());
                            task_resource_clone.refetch();
                            on_close.run(());
                        }
                        Err(_) => {
                            // Try parsing as error string (Tauri error case)
                            match serde_wasm_bindgen::from_value::<String>(result) {
                                Ok(error) => {
                                    web_sys::console::error_1(&format!("Task creation failed: {}", error).into());
                                }
                                Err(parse_error) => {
                                    web_sys::console::error_1(&format!("Failed to parse result: {}", parse_error).into());
                                }
                            }
                        }
                    }
                }
                Err(serialization_error) => {
                    web_sys::console::error_1(&format!("Failed to serialize request: {}", serialization_error).into());
                }
            }
            set_is_creating.set(false);
        });
    };
    
    let test_backend = move |_| {
        spawn_local(async move {
            web_sys::console::log_1(&"Testing backend connection...".into());
            
            // Test 1: Get all tasks
            let result = invoke(events::task::GET_ALL, JsValue::NULL).await;
            web_sys::console::log_1(&format!("GET_ALL result: {:?}", result).into());
            
            // Test 2: Simple task creation with minimal data
            let simple_request = CreateTaskRequest {
                name: "Test Task".to_string(),
                description: None,
                max_sessions: 1,
                tags: vec![],
                config: None,
                audio_config: None,
            };
            
            if let Ok(args) = serde_wasm_bindgen::to_value(&simple_request) {
                web_sys::console::log_1(&"Calling create_task with simple request...".into());
                let result = invoke(events::task::CREATE, args).await;
                web_sys::console::log_1(&format!("CREATE result: {:?}", result).into());
            }
        });
    };

    view! {
        <div class="task-creation-form">
            <h4>"Create New Task"</h4>
            
            <div class="form-actions" style="margin-bottom: 20px;">
                <button
                    class="create-btn"
                    on:click=test_backend
                >
                    "Test Backend"
                </button>
            </div>
            
            <div class="form-group">
                <label>"Task Name"</label>
                <input
                    type="text"
                    placeholder="Enter task name..."
                    prop:value=move || task_name.get()
                    on:input=move |ev| {
                        set_task_name.set(event_target_value(&ev));
                    }
                />
            </div>
            
            <div class="form-group">
                <label>"Description (Optional)"</label>
                <textarea
                    placeholder="Enter task description..."
                    prop:value=move || task_description.get()
                    on:input=move |ev| {
                        set_task_description.set(event_target_value(&ev));
                    }
                ></textarea>
            </div>
            
            <div class="form-group">
                <label>"Max Sessions"</label>
                <input
                    type="number"
                    min="1"
                    max="20"
                    prop:value=move || max_sessions.get().to_string()
                    on:input=move |ev| {
                        if let Ok(value) = event_target_value(&ev).parse::<u8>() {
                            if value >= 1 && value <= 20 {
                                set_max_sessions.set(value);
                            }
                        }
                    }
                />
            </div>
            
            <div class="form-group">
                <label>"Tags (comma-separated)"</label>
                <input
                    type="text"
                    placeholder="work, personal, urgent..."
                    prop:value=move || tags_input.get()
                    on:input=move |ev| {
                        set_tags_input.set(event_target_value(&ev));
                    }
                />
            </div>
            
            <div class="form-actions">
                <button
                    class="create-btn"
                    prop:disabled=move || is_creating.get() || task_name.get().trim().is_empty()
                    on:click=create_task
                >
                    {move || if is_creating.get() { "Creating..." } else { "Create Task" }}
                </button>
                
                <button
                    class="cancel-btn"
                    prop:disabled=move || is_creating.get()
                    on:click=move |_| on_close.run(())
                >
                    "Cancel"
                </button>
            </div>
        </div>
    }
}