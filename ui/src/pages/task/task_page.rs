use leptos::prelude::*;
use crate::pages::task::{TaskList, TasksViewModel};
use crate::utils::ViewModel;

#[component]
pub fn TaskPage() -> impl IntoView {
    let vm = StoredValue::new(TasksViewModel::new());

    view! {
        <div class="tasks-section">
            <h1 class="section-title">"Tasks Directory"</h1>
            <TaskList vm=vm />
        </div>
    }
}