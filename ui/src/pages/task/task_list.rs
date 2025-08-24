use leptos::prelude::*;
use leptos::callback::Callback;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use super::TaskResource;
use crate::pages::task::TaskCreationForm;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn TaskList(task_resource: TaskResource) -> impl IntoView {
    let (show_creation_form, set_show_creation_form) = signal(false);
    let task_resource_signal = StoredValue::new(task_resource);
    
    view! {
        <div>
            <Show when=move || show_creation_form.get()>
                {move || {
                    let task_resource = task_resource_signal.get_value();
                    view! {
                        <div class="setting-group">
                            <TaskCreationForm 
                                task_resource=task_resource
                                on_close=Callback::new(move |_| set_show_creation_form.set(false))
                            />
                        </div>
                    }
                }}
            </Show>
            
            <div>
                {move || {
                    let task_resource = task_resource_signal.get_value();
                    let tasks = task_resource.tasks.get();
                    let active_task_id = task_resource.active_task.get().map(|t| t.id);
                    let tasks_clone = tasks.clone();
                    
                    view! {
                        <>
                            <Show when=move || tasks.is_empty() && !show_creation_form.get()>
                                <div class="task-item">
                                    <div style="text-align: center; opacity: 0.7;">
                                        <p>"No tasks yet. Create your first task to get started!"</p>
                                    </div>
                                </div>
                            </Show>

                            <For
                                each=move || tasks_clone.clone()
                                key=|task| task.id
                                children=move |task| {
                                    let task_id = task.id;
                                    let is_active = active_task_id == Some(task_id);
                                    let task_resource_for_click = task_resource_signal.get_value();
                                    
                                    let progress_percentage = if task.max_sessions > 0 {
                                        (task.current_sessions as f64 / task.max_sessions as f64) * 100.0
                                    } else {
                                        0.0
                                    };
                                    
                                    let task_classes = if is_active {
                                        "task-item active-task"
                                    } else {
                                        "task-item"
                                    };
                                    
                                    view! {
                                        <div 
                                            class=task_classes
                                            on:click=move |_| {
                                                let task_resource = task_resource_for_click.clone();
                                                spawn_local(async move {
                                                    if let Err(e) = task_resource.switch_task(task_id).await {
                                                        web_sys::console::error_1(&format!("Failed to switch task: {e}").into());
                                                    }
                                                });
                                            }
                                        >
                                                            <div class="task-header">
                                                <h3>{task.name.clone()}</h3>
                                                <span class="task-status">
                                                    {if is_active { "Active" } else { "Pending" }}
                                                </span>
                                            </div>

                                                            {task.description.clone().map(|desc| {
                                                if !desc.is_empty() {
                                                    view! {
                                                        <p style="opacity: 0.8; margin: 10px 0;">{desc}</p>
                                                    }.into_any()
                                                } else {
                                                    ().into_any()
                                                }
                                            })}

                                                            <div class="task-meta">
                                                <div class="pomodoro-progress">
                                                    <span class="progress-text">
                                                        {format!("{} of {} pomodoros completed", task.current_sessions, task.max_sessions)}
                                                    </span>
                                                    <div class="progress-bar">
                                                        <div 
                                                            class="progress-fill"
                                                            style=format!("width: {}%", progress_percentage)
                                                        ></div>
                                                    </div>
                                                </div>
                                                <button class="btn-select">
                                                    {if is_active { "Currently Active" } else { "Select Task" }}
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }
                            />

                            <div class="task-item">
                                <div style="text-align: center;">
                                    <button 
                                        class="btn btn-secondary"
                                        on:click=move |_| set_show_creation_form.update(|show| *show = !*show)
                                    >
                                        "Add New Task"
                                    </button>
                                </div>
                            </div>
                        </>
                    }
                }}
            </div>
        </div>
    }
}