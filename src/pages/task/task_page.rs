use leptos::prelude::*;
use crate::pages::task::TaskList;
use super::TaskResource;
use crate::components::PageHeader;

#[component]
pub fn TaskPage() -> impl IntoView {
    let task_resource = expect_context::<TaskResource>();
    
    view! {
        <div class="w-full">
            <PageHeader 
                title="Tasks".to_string()
            />
            <div class="bg-white rounded-2xl p-6 shadow-lg">
                <TaskList task_resource=task_resource />
            </div>
        </div>
    }
}