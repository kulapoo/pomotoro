use crate::pages::task::{TaskList, TasksViewModel};
use crate::utils::ViewModel;
use leptos::prelude::*;

#[component]
pub fn TaskPage() -> impl IntoView {
    let vm = StoredValue::new(TasksViewModel::new());

    view! {
        <div class="tasks-container">
            <div class="tasks-header">
                <h2 class="tasks-title">"My Tasks"</h2>
            </div>
            <TaskList vm=vm />
        </div>
    }
}
