use leptos::prelude::*;
use leptos::callback::Callback;
use leptos::task::spawn_local;
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;
use pomotoro_domain::{Task, TaskStatus, events};
use super::TaskResource;
use crate::pages::task::TaskCreationForm;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn TaskList(task_resource: TaskResource) -> impl IntoView {
    let (is_expanded, set_is_expanded) = signal(true);
    let (show_creation_form, set_show_creation_form) = signal(false);
    let task_resource_signal = StoredValue::new(task_resource);
    
    view! {
        <div class={move || format!("bg-glass dark:bg-glass-dark rounded-3xl p-5 shadow-[var(--shadow-custom)] max-h-[600px] overflow-y-auto transition-all duration-300 {}", 
            if is_expanded.get() { "w-80 md:w-80 w-full max-w-md" } else { "w-15 px-2.5 md:max-h-15 md:w-15 w-full max-h-15" })}>
            <div class="flex justify-between items-center mb-5 border-b border-gray-200 dark:border-gray-600 pb-4">
                <Show when=move || is_expanded.get()>
                    <h3 class="text-xl font-semibold text-gray-900 dark:text-white m-0">"Tasks"</h3>
                </Show>
                <div class="flex gap-2 items-center">
                    <Show when=move || is_expanded.get()>
                        <button 
                            class="bg-gradient-to-br from-green-600 to-green-700 text-white border-none rounded-md w-7 h-7 flex items-center justify-center cursor-pointer text-lg font-semibold transition-all duration-200 hover:from-green-700 hover:to-green-800 hover:scale-105"
                            on:click=move |_| set_show_creation_form.update(|show| *show = !*show)
                            title="Add new task"
                        >
                            "+"
                        </button>
                    </Show>
                    <button 
                        class="bg-transparent border-none text-lg cursor-pointer p-1 rounded-md text-gray-600 dark:text-gray-400 transition-all duration-200 hover:bg-gray-100 dark:hover:bg-gray-700 hover:text-gray-700 dark:hover:text-gray-300"
                        on:click=move |_| set_is_expanded.update(|expanded| *expanded = !*expanded)
                    >
                        {move || if is_expanded.get() { "←" } else { "→" }}
                    </button>
                </div>
            </div>
            
            <Show when=move || is_expanded.get()>
                <div class="overflow-y-auto max-h-[calc(600px-80px)]">
                    <Show when=move || show_creation_form.get()>
                        {move || {
                            let task_resource = task_resource_signal.get_value();
                            view! {
                                <TaskCreationForm 
                                    task_resource=task_resource
                                    on_close=Callback::new(move |_| set_show_creation_form.set(false))
                                />
                            }
                        }}
                    </Show>
                    
                    <div class="flex flex-col gap-3">
                        {move || {
                            let task_resource = task_resource_signal.get_value();
                            let tasks = task_resource.tasks.get();
                            let active_task_id = task_resource.active_task.get().map(|t| t.id);
                            
                            if tasks.is_empty() && !show_creation_form.get() {
                                view! {
                                    <div class="text-center py-10 px-5 text-gray-600 dark:text-gray-400">
                                        <p class="text-sm leading-relaxed">"No tasks yet. Create your first task to get started!"</p>
                                    </div>
                                }.into_any()
                            } else {
                                tasks.into_iter().map(|task| {
                                    let task_id = task.id;
                                    let is_active = active_task_id == Some(task_id);
                                    let task_resource_for_click = task_resource_signal.get_value();
                                    
                                    let progress_percentage = if task.max_sessions > 0 {
                                        (task.current_sessions as f64 / task.max_sessions as f64) * 100.0
                                    } else {
                                        0.0
                                    };
                                    
                                    let task_classes = if is_active {
                                        "p-4 bg-green-100 dark:bg-green-900 rounded-xl border border-green-300 dark:border-green-700 cursor-pointer transition-all duration-200 shadow-[var(--shadow-active)] hover:bg-green-200 dark:hover:bg-green-800 hover:border-green-400 dark:hover:border-green-600 hover:-translate-y-0.5 hover:shadow-md"
                                    } else if task.status == TaskStatus::Completed {
                                        "p-4 opacity-70 bg-gray-100 dark:bg-gray-700 rounded-xl border border-gray-200 dark:border-gray-600 cursor-pointer transition-all duration-200 hover:bg-gray-200 dark:hover:bg-gray-600 hover:border-gray-300 dark:hover:border-gray-500 hover:-translate-y-0.5 hover:shadow-md"
                                    } else if task.status == TaskStatus::Paused {
                                        "p-4 bg-yellow-100 dark:bg-yellow-900 rounded-xl border border-yellow-300 dark:border-yellow-700 cursor-pointer transition-all duration-200 hover:bg-yellow-200 dark:hover:bg-yellow-800 hover:border-yellow-400 dark:hover:border-yellow-600 hover:-translate-y-0.5 hover:shadow-md"
                                    } else {
                                        "p-4 bg-gray-50 dark:bg-slate-800 rounded-xl border border-gray-200 dark:border-slate-600 cursor-pointer transition-all duration-200 hover:bg-gray-100 dark:hover:bg-slate-700 hover:border-gray-300 dark:hover:border-slate-500 hover:-translate-y-0.5 hover:shadow-md"
                                    };
                                    
                                    view! {
                                        <div 
                                            class=task_classes
                                            on:click=move |_| {
                                                let task_resource = task_resource_for_click.clone();
                                                spawn_local(async move {
                                                    if let Err(e) = task_resource.switch_task(task_id).await {
                                                        web_sys::console::error_1(&format!("Failed to switch task: {}", e).into());
                                                    }
                                                });
                                            }
                                        >
                                            <div class="flex justify-between items-center mb-2">
                                                <h4 class="font-semibold text-gray-900 dark:text-white text-base">{task.name.clone()}</h4>
                                                <span class="bg-gray-200 dark:bg-slate-600 text-gray-700 dark:text-slate-200 px-2 py-0.5 rounded-xl text-xs font-medium">
                                                    {task.current_sessions}"/"{task.max_sessions}
                                                </span>
                                            </div>
                                            
                                            <div class="flex gap-2 items-center mt-2">
                                                {if !task.is_completed() {
                                                    let task_id_for_complete = task_id;
                                                    let task_id_for_reset = task_id;
                                                    let task_resource_for_complete = task_resource_for_click.clone();
                                                    let task_resource_for_reset = task_resource_for_click.clone();
                                                    
                                                    view! {
                                                        <button 
                                                            class="bg-transparent border-none p-1 rounded-md cursor-pointer text-sm font-medium transition-all duration-200 flex items-center justify-center min-w-7 h-7 bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300 border border-green-200 dark:border-green-700 hover:bg-green-200 dark:hover:bg-green-800 hover:border-green-300 dark:hover:border-green-600 hover:-translate-y-0.5"
                                                            title="Complete Session"
                                                            on:click=move |e| {
                                                                e.stop_propagation();
                                                                let task_resource = task_resource_for_complete.clone();
                                                                spawn_local(async move {
                                                                    let args = to_value(&task_id_for_complete.to_string()).unwrap();
                                                                    let result = invoke(events::task::COMPLETE_SESSION, args).await;
                                                                    match from_value::<Task>(result) {
                                                                        Ok(_) => {
                                                                            task_resource.refetch();
                                                                            web_sys::console::log_1(&"Session completed".into());
                                                                        }
                                                                        Err(e) => {
                                                                            web_sys::console::error_1(&format!("Failed to complete session: {}", e).into());
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            "✓"
                                                        </button>
                                                        <button 
                                                            class="bg-transparent border-none p-1 rounded-md cursor-pointer text-sm font-medium transition-all duration-200 flex items-center justify-center min-w-7 h-7 bg-yellow-100 dark:bg-yellow-900 text-yellow-700 dark:text-yellow-300 border border-yellow-200 dark:border-yellow-700 hover:bg-yellow-200 dark:hover:bg-yellow-800 hover:border-yellow-300 dark:hover:border-yellow-600 hover:-translate-y-0.5"
                                                            title="Reset Sessions"
                                                            on:click=move |e| {
                                                                e.stop_propagation();
                                                                let task_resource = task_resource_for_reset.clone();
                                                                spawn_local(async move {
                                                                    let args = to_value(&task_id_for_reset.to_string()).unwrap();
                                                                    let result = invoke(events::task::RESET_SESSIONS, args).await;
                                                                    match from_value::<Task>(result) {
                                                                        Ok(_) => {
                                                                            task_resource.refetch();
                                                                            web_sys::console::log_1(&"Sessions reset".into());
                                                                        }
                                                                        Err(e) => {
                                                                            web_sys::console::error_1(&format!("Failed to reset sessions: {}", e).into());
                                                                        }
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            "↻"
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="text-xl ml-auto" title="Task completed">
                                                            "🎉"
                                                        </span>
                                                    }.into_any()
                                                }}
                                            </div>
                                            
                                            {task.description.clone().map(|desc| {
                                                if !desc.is_empty() {
                                                    view! {
                                                        <p class="text-sm text-gray-600 dark:text-gray-400 mb-3 leading-relaxed">{desc}</p>
                                                    }.into_any()
                                                } else {
                                                    view! {}.into_any()
                                                }
                                            })}
                                            
                                            <div class="bg-gray-200 dark:bg-slate-600 h-1 rounded-sm overflow-hidden mb-2">
                                                <div 
                                                    class={format!("h-full rounded-sm transition-all duration-300 {}", 
                                                        if task.status == TaskStatus::Completed { 
                                                            "bg-gradient-to-r from-gray-600 to-gray-500" 
                                                        } else { 
                                                            "bg-gradient-to-r from-green-500 to-green-600" 
                                                        })}
                                                    style=format!("width: {}%", progress_percentage)
                                                ></div>
                                            </div>
                                            
                                            {if !task.tags.is_empty() {
                                                view! {
                                                    <div class="flex gap-1.5 flex-wrap">
                                                        {task.tags.into_iter().map(|tag| {
                                                            view! {
                                                                <span class="bg-amber-100 dark:bg-amber-900 text-amber-800 dark:text-amber-200 px-2 py-0.5 rounded-2xl text-xs font-medium">{tag}</span>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {}.into_any()
                                            }}
                                        </div>
                                    }
                                }).collect::<Vec<_>>().into_any()
                            }
                        }}
                    </div>
                </div>
            </Show>
        </div>
    }
}