use crate::pages::task::{TaskCreationForm, TaskUpdateForm, TasksViewModel};
use domain::Task;
use leptos::prelude::*;

#[component]
pub fn TaskList(vm: StoredValue<TasksViewModel>) -> impl IntoView {
    let (editing_task, set_editing_task) = signal(None::<Task>);
    view! {
        <div class="task-list-container">
            <div class="task-actions">
                <Show when=move || vm.with_value(|v| v.is_creating_task())>
                    {move || {
                        view! {
                            <TaskCreationForm
                                vm=vm
                                on_close={move || vm.with_value(|v| v.set_creating_task(false))}
                            />
                        }
                    }}
                </Show>

                <Show when=move || editing_task.get().is_some()>
                    {move || {
                        let task = editing_task.get().expect("editing_task should be Some");
                        view! {
                            <TaskUpdateForm
                                vm=vm
                                task=task
                                on_close={move || set_editing_task.set(None)}
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
                                    let task_name_for_delete = task.name.clone();
                                    let task_for_edit = task.clone();
                                    let task_status_for_button = task.status.clone();
                                    let task_status_for_disabled = task.status.clone();

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
                                        <div class=task_classes>
                                            <div class="task-content" on:click=move |_| {
                                                vm.with_value(|v| v.switch_active_task(task_id));
                                            }>
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
                                                <button class="btn-select" disabled=move || task_status_for_disabled == domain::TaskStatus::Completed>
                                                    {if is_active { 
                                                        "Currently Active" 
                                                    } else if task_status_for_button == domain::TaskStatus::Completed {
                                                        "Task Completed"
                                                    } else { 
                                                        "Select Task" 
                                                    }}
                                                </button>
                                            </div>
                                            </div>

                                            <div class="task-actions">
                                                <button
                                                    class="btn-icon btn-edit"
                                                    title="Edit Task"
                                                    on:click=move |ev| {
                                                        ev.stop_propagation();
                                                        set_editing_task.set(Some(task_for_edit.clone()));
                                                    }
                                                >
                                                    "✏️"
                                                </button>
                                                <button
                                                    class="btn-icon btn-delete"
                                                    title="Delete Task"
                                                    on:click=move |ev| {
                                                        ev.stop_propagation();
                                                        if web_sys::window()
                                                            .unwrap()
                                                            .confirm_with_message(&format!("Are you sure you want to delete '{}'?", task_name_for_delete))
                                                            .unwrap_or(false)
                                                        {
                                                            vm.with_value(|v| v.delete_task(task_id));
                                                        }
                                                    }
                                                >
                                                    "🗑️"
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
