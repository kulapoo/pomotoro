use leptos::prelude::*;
use leptos::task::spawn_local;

use domain::{Task, event_names::commands};
use crate::utils::invoke;

/// Checks if the current active task has reached its maximum sessions
/// and cycles to the next task if needed
pub async fn check_task_cycle(set_active_task: WriteSignal<Option<Task>>) {
    // Check if current task has reached max sessions and needs to cycle
    invoke::<Vec<Task>, ()>(commands::task::GET_ACTIVE, None).await
        .ok()
        .and_then(|tasks| tasks.first().cloned())
        .map(|task| {
            // Check if task completed its max sessions
            if task.current_sessions >= task.max_sessions {
                // Cycle to next incomplete task
                spawn_local(async move {
                    cycle_to_next_task(set_active_task).await;
                });
            } else {
                set_active_task.set(Some(task));
            }
        })
        .unwrap_or_else(|| {
            web_sys::console::error_1(&"Failed to check task cycle".into());
        });
}

/// Cycles to the next incomplete task in the queue
pub async fn cycle_to_next_task(set_active_task: WriteSignal<Option<Task>>) {
    // Try to get Task
    let task = invoke::<Task, ()>(commands::task::CYCLE_INCOMPLETE_TASK, None).await
        .ok();

    task.as_ref()
        .map(|t| web_sys::console::log_1(&format!("Cycled to next task: {}", t.name).into()))
        .unwrap_or_else(|| web_sys::console::error_1(&"Failed to cycle task".into()));

    set_active_task.set(task);
}

/// Fetches the currently active task from the backend
pub async fn fetch_active_task(set_active_task: WriteSignal<Option<Task>>) {
    let active_task = invoke::<Vec<Task>, ()>(commands::task::GET_ACTIVE, None).await
        .ok()
        .and_then(|tasks| tasks.first().cloned());

    set_active_task.set(active_task);
}

/// Fetches a specific task by its ID
pub async fn fetch_task_by_id(task_id: &str, set_active_task: WriteSignal<Option<Task>>) {
    use serde::Serialize;

    #[derive(Serialize)]
    struct GetTaskArgs {
        id: String,
    }

    let args = GetTaskArgs {
        id: task_id.to_string(),
    };

    let task = invoke::<Option<Task>, _>(commands::task::GET, Some(args)).await
        .ok()
        .flatten()
        .map(|task| {
            web_sys::console::log_1(&format!("Timer page: Loaded active task: {}", task.name).into());
            task
        });

    if task.is_none() {
        web_sys::console::log_1(&"Timer page: Task not found or failed to parse".into());
    }

    set_active_task.set(task);
}