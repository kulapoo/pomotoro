use leptos::prelude::*;
use leptos::callback::Callback;
use crate::pages::task::{TaskCreationForm, TasksViewModel};

#[component]
pub fn TaskList(vm: StoredValue<TasksViewModel>) -> impl IntoView {
    
    view! {
        <>
            <div class="add-task-form">
                <Show when=move || vm.with_value(|v| v.is_creating_task())>
                    {move || {
                        view! {
                            <TaskCreationForm 
                                vm=vm
                                on_close=Callback::new(move |_| vm.with_value(|v| v.set_creating_task(false)))
                            />
                        }
                    }}
                </Show>
                <Show when=move || !vm.with_value(|v| v.is_creating_task())>
                    <button 
                        class="btn btn-primary"
                        on:click=move |_| vm.with_value(|v| v.set_creating_task(true))
                    >
                        "ADD"
                    </button>
                </Show>
            </div>
            
            <ul class="task-list" id="taskList">
                {move || {
                    let (tasks, active_task_id) = vm.with_value(|v| {
                        let tasks = v.get_tasks();
                        let active_task = v.get_active_task();
                        let active_task_id = active_task.as_ref().map(|t| t.id);
                        (tasks, active_task_id)
                    });
                    let tasks_clone = tasks.clone();
                    
                    view! {
                        <>
                            <Show when=move || tasks.is_empty() && !vm.with_value(|v| v.is_creating_task())>
                                <p style="text-align: center; opacity: 0.7; padding: 20px;">"No tasks yet. Create your first task to get started!"</p>
                            </Show>

                            <For
                                each=move || tasks_clone.clone()
                                key=|task| task.id
                                children=move |task| {
                                    let task_id = task.id;
                                    let is_active = active_task_id == Some(task_id);
                                    
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
                                        <li 
                                            class=task_classes
                                            on:click=move |_| {
                                                vm.with_value(|v| v.switch_active_task(task_id));
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
                                        </li>
                                    }
                                }
                            />
                        </>
                    }
                }}
            </ul>
        </>
    }
}