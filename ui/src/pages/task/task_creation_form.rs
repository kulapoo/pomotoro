use leptos::prelude::*;
use leptos::callback::Callback;
use leptos::task::spawn_local;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use domain::events;
use super::TaskResource;

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
    config: Option<domain::TaskConfig>,
    audio_config: Option<domain::AudioConfig>,
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
                    
                    match serde_wasm_bindgen::from_value::<domain::Task>(result.clone()) {
                        Ok(task) => {
                            web_sys::console::log_1(&format!("Task created successfully: {}", task.name).into());
                            task_resource_clone.refetch();
                            on_close.run(());
                        }
                        Err(_) => {
                            match serde_wasm_bindgen::from_value::<String>(result) {
                                Ok(error) => {
                                    web_sys::console::error_1(&format!("Task creation failed: {error}").into());
                                }
                                Err(parse_error) => {
                                    web_sys::console::error_1(&format!("Failed to parse result: {parse_error}").into());
                                }
                            }
                        }
                    }
                }
                Err(serialization_error) => {
                    web_sys::console::error_1(&format!("Failed to serialize request: {serialization_error}").into());
                }
            }
            set_is_creating.set(false);
        });
    };
    
    let test_backend = move |_| {
        spawn_local(async move {
            web_sys::console::log_1(&"Testing backend connection...".into());
            
            let result = invoke(events::task::GET_ALL, JsValue::NULL).await;
            web_sys::console::log_1(&format!("GET_ALL result: {result:?}").into());
            
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
                web_sys::console::log_1(&format!("CREATE result: {result:?}").into());
            }
        });
    };

    view! {
        <div class="bg-gray-50 dark:bg-slate-800 border border-gray-200 dark:border-slate-600 rounded-xl p-4 mb-4">
            <h4 class="m-0 mb-4 text-base font-semibold text-gray-900 dark:text-white">"Create New Task"</h4>
            
            <div class="flex justify-end mb-5">
                <button
                    class="bg-gradient-to-br from-blue-500 to-blue-600 text-white border-none rounded-lg px-4 py-2 text-sm font-medium cursor-pointer transition-all duration-200 hover:from-blue-600 hover:to-blue-700 hover:-translate-y-0.5"
                    on:click=test_backend
                >
                    "Test Backend"
                </button>
            </div>
            
            <div class="mb-3">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Task Name"</label>
                <input
                    type="text"
                    class="w-full px-3 py-2 border border-gray-300 dark:border-slate-600 rounded-md text-sm bg-white dark:bg-slate-700 text-gray-900 dark:text-white transition-colors focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-200 dark:focus:ring-blue-300"
                    placeholder="Enter task name..."
                    prop:value=move || task_name.get()
                    on:input=move |ev| {
                        set_task_name.set(event_target_value(&ev));
                    }
                />
            </div>
            
            <div class="mb-3">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description (Optional)"</label>
                <textarea
                    class="w-full px-3 py-2 border border-gray-300 dark:border-slate-600 rounded-md text-sm bg-white dark:bg-slate-700 text-gray-900 dark:text-white transition-colors resize-y min-h-15 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-200 dark:focus:ring-blue-300"
                    placeholder="Enter task description..."
                    prop:value=move || task_description.get()
                    on:input=move |ev| {
                        set_task_description.set(event_target_value(&ev));
                    }
                ></textarea>
            </div>
            
            <div class="mb-3">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Max Sessions"</label>
                <input
                    type="number"
                    min="1"
                    max="20"
                    class="w-full px-3 py-2 border border-gray-300 dark:border-slate-600 rounded-md text-sm bg-white dark:bg-slate-700 text-gray-900 dark:text-white transition-colors focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-200 dark:focus:ring-blue-300"
                    prop:value=move || max_sessions.get().to_string()
                    on:input=move |ev| {
                        if let Ok(value) = event_target_value(&ev).parse::<u8>() {
                            if (1..=20).contains(&value) {
                                set_max_sessions.set(value);
                            }
                        }
                    }
                />
            </div>
            
            <div class="mb-3">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Tags (comma-separated)"</label>
                <input
                    type="text"
                    class="w-full px-3 py-2 border border-gray-300 dark:border-slate-600 rounded-md text-sm bg-white dark:bg-slate-700 text-gray-900 dark:text-white transition-colors focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-200 dark:focus:ring-blue-300"
                    placeholder="work, personal, urgent..."
                    prop:value=move || tags_input.get()
                    on:input=move |ev| {
                        set_tags_input.set(event_target_value(&ev));
                    }
                />
            </div>
            
            <div class="flex gap-2 justify-end mt-4">
                <button
                    class="bg-gradient-to-br from-blue-500 to-blue-600 text-white border-none rounded-lg px-4 py-2 text-sm font-medium cursor-pointer transition-all duration-200 hover:from-blue-600 hover:to-blue-700 hover:-translate-y-0.5 disabled:opacity-60 disabled:cursor-not-allowed disabled:transform-none"
                    prop:disabled=move || is_creating.get() || task_name.get().trim().is_empty()
                    on:click=create_task
                >
                    {move || if is_creating.get() { "Creating..." } else { "Create Task" }}
                </button>
                
                <button
                    class="bg-gray-100 dark:bg-slate-600 text-gray-700 dark:text-slate-200 border border-gray-200 dark:border-slate-500 rounded-lg px-4 py-2 text-sm font-medium cursor-pointer transition-all duration-200 hover:bg-gray-200 dark:hover:bg-slate-500 disabled:opacity-60 disabled:cursor-not-allowed"
                    prop:disabled=move || is_creating.get()
                    on:click=move |_| on_close.run(())
                >
                    "Cancel"
                </button>
            </div>
        </div>
    }
}