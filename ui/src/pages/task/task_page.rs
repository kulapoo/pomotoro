use crate::pages::task::{TaskList, TaskSearch, TasksViewModel};
use crate::utils::ViewModel;
use leptos::prelude::*;

#[component]
pub fn TaskPage() -> impl IntoView {
    let vm = StoredValue::new(TasksViewModel::new());

    view! {
        <div class="tasks-container">
            <div class="tasks-header">
                <h2 class="tasks-title">"My Tasks"</h2>
                <div class="task-stats">
                    {move || {
                        let tasks = vm.with_value(|v| v.get_tasks());
                        let total = tasks.len();
                        let completed = tasks.iter().filter(|t| t.status == domain::TaskStatus::Completed).count();
                        let active = tasks.iter().filter(|t| t.status == domain::TaskStatus::Active).count();
                        view! {
                            <div class="stat-badge">
                                <span class="stat-value">{total}</span>
                                <span class="stat-label">"Total"</span>
                            </div>
                            <div class="stat-badge active">
                                <span class="stat-value">{active}</span>
                                <span class="stat-label">"Active"</span>
                            </div>
                            <div class="stat-badge completed">
                                <span class="stat-value">{completed}</span>
                                <span class="stat-label">"Completed"</span>
                            </div>
                        }
                    }}
                </div>
            </div>
            <TaskSearch vm=vm />
            <TaskList vm=vm />
        </div>
    }
}
