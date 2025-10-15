use super::TaskDirectoryViewModel;
use domain::{Task, TaskStatus};
use leptos::prelude::*;

#[component]
pub fn TaskListItem<F>(
    task: Task,
    is_active: bool,
    vm: StoredValue<TaskDirectoryViewModel>,
    on_edit: F,
) -> impl IntoView
where
    F: Fn(Task) + 'static + Clone,
{
    let task_id = task.id;
    let task_for_edit = task.clone();
    let task_status_for_button = task.status.clone();
    let task_status_for_disabled = task.status.clone();
    let task_status_for_onclick = task.status.clone();

    let progress_percentage = if task.max_sessions > 0 {
        (task.current_sessions as f64 / task.max_sessions as f64) * 100.0
    } else {
        0.0
    };

    let task_classes = match (&task.status, is_active) {
        (_, true) => "task-item active-task",
        (TaskStatus::Completed, _) => "task-item completed-task",
        (TaskStatus::Paused, _) => "task-item paused-task",
        (TaskStatus::Queued, _) => "task-item queued-task",
        _ => "task-item"
    };

    web_sys::console::log_1(&format!("Task {:?}", task.status).into());

    view! {
        <div class=task_classes>
            <div class="task-content" on:click=move |_| {
                vm.with_value(|v| v.switch_active_task(task_id));
            }>
                <div class="task-header">
                    <h3>{task.name.clone()}</h3>
                    <span class={format!("task-status status-{}",
                        match &task.status {
                            TaskStatus::Active => "active",
                            TaskStatus::Completed => "completed",
                            TaskStatus::Paused => "paused",
                            TaskStatus::Queued => "queued",
                        }
                    )}>
                        {match &task.status {
                            TaskStatus::Active => "Active",
                            TaskStatus::Completed => "Completed",
                            TaskStatus::Paused => "Paused",
                            TaskStatus::Queued => "Queued",
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
                    <button
                        class="btn-select"
                        disabled=move || is_active || task_status_for_disabled == TaskStatus::Completed
                        on:click=move |ev| {
                            ev.stop_propagation();
                            if !is_active && task_status_for_onclick != TaskStatus::Completed {
                                vm.with_value(|v| v.switch_active_task(task_id));
                            }
                        }
                    >
                        {if is_active {
                            "Currently Active"
                        } else if task_status_for_button == TaskStatus::Completed {
                            "Task Completed"
                        } else {
                            "Select Task"
                        }}
                    </button>
                </div>
            </div>

            <div class="task-actions">
                {if task.status == TaskStatus::Completed {
                    view! {
                        <button
                            class="btn-icon btn-reset"
                            title="Reset to Queued"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                let reset_sessions = web_sys::window()
                                    .unwrap()
                                    .confirm_with_message("Reset session count to 0?")
                                    .unwrap_or(false);
                                vm.with_value(|v| v.reset_task_to_queued(task_id, reset_sessions));
                            }
                        >
                            "🔄"
                        </button>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                <button
                    class="btn-icon btn-edit"
                    title="Edit Task"
                    on:click={
                        let on_edit = on_edit.clone();
                        move |ev| {
                            ev.stop_propagation();
                            on_edit(task_for_edit.clone());
                        }
                    }
                >
                    "✏️"
                </button>
                <button
                    class="btn-icon btn-delete"
                    title="Delete Task"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        vm.with_value(|v| v.delete_task(task_id));
                    }
                >
                    "🗑️"
                </button>
            </div>
        </div>
    }
}