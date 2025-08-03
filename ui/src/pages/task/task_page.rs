use leptos::prelude::*;
use crate::pages::task::TaskList;
use super::TaskResource;

#[component]
pub fn TaskPage() -> impl IntoView {
    let task_resource = expect_context::<TaskResource>();
    
    view! {
        <div class="tasks-section">
            <h1 class="section-title">"Tasks Directory"</h1>
            <TaskList task_resource=task_resource />
        </div>
    }
}