use super::{TaskDirectoryViewModel, TaskListItem};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn TaskList(vm: StoredValue<TaskDirectoryViewModel>) -> impl IntoView {
    let navigate = use_navigate();

    view! {
        <div class="task-list-container">
            <div class="task-actions">
                <button
                    class="btn btn-add-task"
                    on:click={
                        let navigate = navigate.clone();
                        move |_| navigate("/tasks/new", Default::default())
                    }
                >
                    <span class="btn-icon">"➕"</span>
                    "Add New Task"
                </button>
            </div>

            <div class="task-list" id="taskList">
                {
                    let navigate = navigate.clone();
                    move || {
                        let tasks = vm.with_value(|v| v.get_tasks());
                        let active_task_id = vm.with_value(|v| {
                            v.get_active_task().as_ref().map(|t| t.id)
                        });

                        view! {
                            <>
                                <Show when=move || tasks.is_empty()>
                                    <div class="empty-state">
                                        <p class="empty-state-text">"No tasks yet. Create your first task to get started!"</p>
                                    </div>
                                </Show>

                                <For
                                    each=move || vm.with_value(|v| v.get_tasks())
                                    key=|task| task.id
                                    children={
                                        let navigate = navigate.clone();
                                        move |task| {
                                            let is_active = active_task_id == Some(task.id);
                                            let task_id = task.id;
                                            let edit_url = format!("/tasks/{}/edit", task_id);
                                            let navigate = navigate.clone();
                                            view! {
                                                <TaskListItem
                                                    task=task
                                                    is_active=is_active
                                                    vm=vm
                                                    on_edit={move |_task| {
                                                        let navigate = navigate.clone();
                                                        navigate(&edit_url, Default::default())
                                                    }}
                                                />
                                            }
                                        }
                                    }
                                />
                            </>
                        }
                    }
                }
            </div>
        </div>
    }
}