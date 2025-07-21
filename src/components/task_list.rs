use leptos::prelude::*;
use leptos::callback::Callback;
use leptos::task::spawn_local;
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;
use pomotoro_domain::{Task, TaskId, TaskStatus, TimerStateWithTask, events};
use crate::components::TaskCreationForm;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Clone)]
pub struct TaskResource {
    pub tasks: ReadSignal<Vec<Task>>,
    pub active_task: ReadSignal<Option<Task>>,
    set_tasks: WriteSignal<Vec<Task>>,
    set_active_task: WriteSignal<Option<Task>>,
}

impl TaskResource {
    pub fn new() -> Self {
        let (tasks, set_tasks) = signal(Vec::<Task>::new());
        let (active_task, set_active_task) = signal(None::<Task>);

        let set_tasks_clone = set_tasks;
        let set_active_task_clone = set_active_task;
        
        spawn_local(async move {
            // Load all tasks
            web_sys::console::log_1(&"Loading all tasks...".into());
            let result = invoke(events::task::GET_ALL, JsValue::NULL).await;
            match from_value::<Vec<Task>>(result) {
                Ok(task_list) => {
                    web_sys::console::log_1(&format!("Loaded {} tasks", task_list.len()).into());
                    set_tasks_clone.set(task_list);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load tasks: {}", e).into());
                }
            }
            
            // Load timer state with active task
            web_sys::console::log_1(&"Loading timer state with task...".into());
            let result = invoke(events::timer::GET_STATE_WITH_TASK, JsValue::NULL).await;
            match from_value::<Option<Task>>(result) {
                Ok(task) => {
                    web_sys::console::log_1(&format!("Active task: {:?}", task.as_ref().map(|t| &t.name)).into());
                    set_active_task_clone.set(task);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load active task: {}", e).into());
                }
            }
        });

        Self { tasks, active_task, set_tasks, set_active_task }
    }

    pub async fn switch_task(&self, task_id: TaskId) -> Result<(), String> {
        let args = to_value(&task_id).map_err(|e| e.to_string())?;
        let result = invoke(events::timer::SWITCH_TASK, args).await;
        
        match from_value::<TimerStateWithTask>(result) {
            Ok(timer_state_with_task) => {
                self.set_active_task.set(timer_state_with_task.active_task.clone());
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn refetch(&self) {
        let set_tasks = self.set_tasks;
        let set_active_task = self.set_active_task;
        
        spawn_local(async move {
            web_sys::console::log_1(&"Refetching tasks...".into());
            let result = invoke(events::task::GET_ALL, JsValue::NULL).await;
            match from_value::<Vec<Task>>(result) {
                Ok(task_list) => {
                    web_sys::console::log_1(&format!("Refetched {} tasks", task_list.len()).into());
                    set_tasks.set(task_list);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to refetch tasks: {}", e).into());
                }
            }
            
            let result = invoke(events::timer::GET_STATE_WITH_TASK, JsValue::NULL).await;
            match from_value::<TimerStateWithTask>(result) {
                Ok(timer_state_with_task) => {
                    web_sys::console::log_1(&"Refetched active task".into());
                    set_active_task.set(timer_state_with_task.active_task);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to refetch active task: {}", e).into());
                }
            }
        });
    }
}

#[component]
pub fn TaskList(task_resource: TaskResource) -> impl IntoView {
    let (is_expanded, set_is_expanded) = signal(true);
    let (show_creation_form, set_show_creation_form) = signal(false);
    let task_resource_signal = StoredValue::new(task_resource);
    
    view! {
        <div class="task-sidebar" class:collapsed=move || !is_expanded.get()>
            <div class="task-sidebar-header">
                <Show when=move || is_expanded.get()>
                    <h3>"Tasks"</h3>
                </Show>
                <div class="header-actions">
                    <Show when=move || is_expanded.get()>
                        <button 
                            class="add-task-btn"
                            on:click=move |_| set_show_creation_form.update(|show| *show = !*show)
                            title="Add new task"
                        >
                            "+"
                        </button>
                    </Show>
                    <button 
                        class="collapse-btn"
                        on:click=move |_| set_is_expanded.update(|expanded| *expanded = !*expanded)
                    >
                        {move || if is_expanded.get() { "←" } else { "→" }}
                    </button>
                </div>
            </div>
            
            <Show when=move || is_expanded.get()>
                <div class="task-sidebar-content">
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
                    
                    <div class="task-list">
                        {move || {
                            let task_resource = task_resource_signal.get_value();
                            let tasks = task_resource.tasks.get();
                            let active_task_id = task_resource.active_task.get().map(|t| t.id);
                            
                            if tasks.is_empty() && !show_creation_form.get() {
                                view! {
                                    <div class="empty-state">
                                        <p>"No tasks yet. Create your first task to get started!"</p>
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
                                    
                                    let mut task_class = vec!["task-item"];
                                    if is_active {
                                        task_class.push("active");
                                    }
                                    if task.status == TaskStatus::Completed {
                                        task_class.push("task-completed");
                                    } else if task.status == TaskStatus::Paused {
                                        task_class.push("task-paused");
                                    }
                                    let class_str = task_class.join(" ");
                                    
                                    view! {
                                        <div 
                                            class=class_str
                                            on:click=move |_| {
                                                let task_resource = task_resource_for_click.clone();
                                                spawn_local(async move {
                                                    if let Err(e) = task_resource.switch_task(task_id).await {
                                                        web_sys::console::error_1(&format!("Failed to switch task: {}", e).into());
                                                    }
                                                });
                                            }
                                        >
                                            <div class="task-header">
                                                <h4 class="task-name">{task.name.clone()}</h4>
                                                <span class="task-sessions">
                                                    {task.current_sessions}"/"{task.max_sessions}
                                                </span>
                                            </div>
                                            
                                            {task.description.clone().map(|desc| {
                                                if !desc.is_empty() {
                                                    view! {
                                                        <p class="task-description">{desc}</p>
                                                    }.into_any()
                                                } else {
                                                    view! {}.into_any()
                                                }
                                            })}
                                            
                                            <div class="task-progress">
                                                <div 
                                                    class="task-progress-bar"
                                                    style=format!("width: {}%", progress_percentage)
                                                ></div>
                                            </div>
                                            
                                            {if !task.tags.is_empty() {
                                                view! {
                                                    <div class="task-tags">
                                                        {task.tags.into_iter().map(|tag| {
                                                            view! {
                                                                <span class="task-tag">{tag}</span>
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