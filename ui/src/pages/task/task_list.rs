use crate::pages::task::{TaskCreationForm, TasksViewModel};
use leptos::callback::Callback;
use leptos::prelude::*;

#[component]
pub fn TaskList(vm: StoredValue<TasksViewModel>) -> impl IntoView {
    view! {
        <div class="task-list-container">
            <div class="task-actions">
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
                        class="btn btn-add-task"
                        on:click=move |_| vm.with_value(|v| v.set_creating_task(true))
                    >
                        <span class="btn-icon">"+ "</span>
                        "Add New Task"
                    </button>
                </Show>
            </div>

            <div class="task-list" id="taskList">
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
                                <div class="empty-state">
                                    <p class="empty-state-text">"No tasks yet. Create your first task to get started!"</p>
                                </div>
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

                                    let task_classes = match (&task.status, is_active) {
                                        (_, true) => "task-item active-task",
                                        (domain::TaskStatus::Completed, _) => "task-item completed-task",
                                        (domain::TaskStatus::Paused, _) => "task-item paused-task",
                                        (domain::TaskStatus::Queued, _) => "task-item queued-task",
                                        _ => "task-item"
                                    };

                                    view! {
                                        <div
                                            class=task_classes
                                            on:click=move |_| {
                                                vm.with_value(|v| v.switch_active_task(task_id));
                                            }
                                        >
                                                            <div class="task-header">
                                                <h3>{task.name.clone()}</h3>
                                                <span class={format!("task-status status-{}", 
                                                    match &task.status {
                                                        domain::TaskStatus::Active => "active",
                                                        domain::TaskStatus::Completed => "completed",
                                                        domain::TaskStatus::Paused => "paused",
                                                        domain::TaskStatus::Queued => "queued",
                                                    }
                                                )}>
                                                    {match &task.status {
                                                        domain::TaskStatus::Active => "Active",
                                                        domain::TaskStatus::Completed => "Completed",
                                                        domain::TaskStatus::Paused => "Paused",
                                                        domain::TaskStatus::Queued => "Queued",
                                                    }}
                                                </span>
                                            </div>

                                                            {task.description.clone().map(|desc| {
                                                if !desc.is_empty() {
                                                    view! {
                                                        <p class="task-description">{desc}</p>
                                                    }.into_any()
                                                } else {
                                                    ().into_any()
                                                }
                                            })}
                                            
                                                            {if !task.tags.is_empty() {
                                                view! {
                                                    <div class="task-tags">
                                                        {task.tags.iter().map(|tag| {
                                                            view! {
                                                                <span class="task-tag">{tag.clone()}</span>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any()
                                            } else {
                                                ().into_any()
                                            }}

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
                                                <button class="btn-select" disabled=move || task.status == domain::TaskStatus::Completed>
                                                    {if is_active { 
                                                        "Currently Active" 
                                                    } else if task.status == domain::TaskStatus::Completed {
                                                        "Task Completed"
                                                    } else { 
                                                        "Select Task" 
                                                    }}
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        </>
                    }
                }}
            </div>
        </div>
    }
}
