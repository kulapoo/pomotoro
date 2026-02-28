use domain::Task;
use leptos::prelude::*;

#[component]
pub fn TaskCompletionIndicator(
    task: ReadSignal<Option<Task>>,
) -> impl IntoView {
    let completion_class = move || {
        task.get()
            .map(|t| {
                if t.is_completed() {
                    "task-complete"
                } else {
                    "task-incomplete"
                }
            })
            .unwrap_or("task-no-selection")
    };

    let completion_text = move || {
        task.get()
            .map(|t| {
                if t.is_completed() {
                    format!(
                        "✓ Complete ({}/{})",
                        t.current_sessions, t.max_sessions
                    )
                } else {
                    format!(
                        "{}/{} sessions",
                        t.current_sessions, t.max_sessions
                    )
                }
            })
            .unwrap_or_else(|| "No task selected".to_string())
    };

    let progress_percentage = move || {
        task.get()
            .map(|t| t.get_progress_ratio() * 100.0)
            .unwrap_or(0.0)
    };

    view! {
        <div class=move || format!("task-completion-indicator {}", completion_class())>
            <div class="completion-progress">
                <div
                    class="progress-bar"
                    style=move || format!("width: {}%", progress_percentage())
                />
            </div>
            <div class="completion-text">
                {completion_text}
            </div>
        </div>
    }
}
