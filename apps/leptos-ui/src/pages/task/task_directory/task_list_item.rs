use super::TaskDirectoryViewModel;
use domain::{Task, TaskStatus};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

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
    let navigate = use_navigate();

    // Create a single derived signal for button state
    let is_task_completed = task.status == TaskStatus::Completed;
    let button_disabled = is_active || is_task_completed;
    let button_enabled = !is_active && !is_task_completed;

    let progress_percentage = if task.max_sessions > 0 {
        (task.current_sessions as f64 / task.max_sessions as f64) * 100.0
    } else {
        0.0
    };

    let task_classes = match (&task.status, is_active) {
        (_, true) => "bg-indigo-600/5 border-l-4 border-indigo-600 shadow-md",
        (TaskStatus::Completed, _) => "bg-slate-50 opacity-75",
        (TaskStatus::Paused, _) => "bg-amber-500/5 border-l-4 border-amber-500",
        (TaskStatus::Queued, _) => "bg-white",
        _ => "bg-white",
    };

    web_sys::console::log_1(&format!("Task {:?}", task.status).into());

    view! {
        <div class={format!("rounded-lg shadow-sm border border-slate-200 transition-all duration-200 hover:shadow-md {}", task_classes)}>
            <div class="p-4 cursor-pointer" on:click=move |_| {
                vm.with_value(|v| v.switch_active_task(task_id, None));
            }>
                <div class="flex justify-between items-start mb-3">
                    <h3 class="text-lg font-semibold text-slate-800">{task.name.clone()}</h3>
                    <span class={format!("px-3 py-1 text-xs font-medium rounded-full {}",
                        match &task.status {
                            TaskStatus::Active => "bg-indigo-600 text-white",
                            TaskStatus::Completed => "bg-emerald-500 text-white",
                            TaskStatus::Paused => "bg-amber-500 text-white",
                            TaskStatus::Queued => "bg-slate-600 text-white",
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

                <p class="text-xs text-slate-400 font-mono mb-2">
                    {format!("ID: {}", task_id.to_string().chars().take(8).collect::<String>())}
                </p>

                {task.description.clone().map(|desc| {
                    if !desc.is_empty() {
                        view! {
                            <p class="text-slate-600 text-sm mb-3">{desc}</p>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                })}

                {if !task.tags.is_empty() {
                    view! {
                        <div class="flex flex-wrap gap-2 mb-3">
                            {task.tags.iter().map(|tag| {
                                view! {
                                    <span class="px-2 py-1 text-xs bg-slate-200 text-slate-700 rounded-md">{tag.clone()}</span>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }}

                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                    <div class="flex-1">
                        <span class="text-sm text-slate-600 mb-2 block">
                            {format!("{} of {} pomodoros completed", task.current_sessions, task.max_sessions)}
                        </span>
                        <div class="w-full bg-slate-200 rounded-full h-2 overflow-hidden">
                            <div
                                class="h-full bg-indigo-600 rounded-full transition-all duration-300"
                                style=format!("width: {}%", progress_percentage)
                            ></div>
                        </div>
                    </div>
                    <button
                        class={format!(
                            "px-4 py-2 font-medium rounded-md transition-all duration-200 {} {}",
                            if button_enabled { "bg-indigo-600 text-white hover:bg-indigo-700" } else { "bg-slate-300 text-slate-600" },
                            if button_disabled { "cursor-not-allowed" } else { "" }
                        )}
                        disabled=button_disabled
                        on:click=move |ev| {
                            ev.stop_propagation();
                            if button_enabled {
                                let navigate = navigate.clone();
                                vm.with_value(|v| v.switch_active_task(
                                    task_id,
                                    Some(Box::new(move || navigate("/timer", Default::default())))
                                ));
                            }
                        }
                    >
                        {if is_active {
                            "Currently Active"
                        } else if is_task_completed {
                            "Task Completed"
                        } else {
                            "Select Task"
                        }}
                    </button>
                </div>
            </div>

            <div class="flex gap-2 px-4 pb-4 border-t border-slate-200 pt-3">
                {if task.status == TaskStatus::Completed {
                    view! {
                        <button
                            class="p-2 text-2xl hover:bg-slate-100 rounded-md transition-all duration-200"
                            title="Reset to Queued"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                let reset_sessions = web_sys::window()
                                    .unwrap()
                                    .confirm_with_message("Reset session?")
                                    .unwrap_or(false);

                                if reset_sessions {
                                    vm.with_value(|v| v.reset_task_to_queued(task_id));
                                }
                            }
                        >
                            "🔄"
                        </button>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                <button
                    class="p-2 text-2xl hover:bg-slate-100 rounded-md transition-all duration-200"
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
                    class="p-2 text-2xl hover:bg-slate-100 rounded-md transition-all duration-200"
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
