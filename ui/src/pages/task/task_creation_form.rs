use crate::pages::task::TasksViewModel;
use leptos::callback::Callback;
use leptos::prelude::*;

#[component]
pub fn TaskCreationForm(
    vm: StoredValue<TasksViewModel>,
    on_close: Callback<()>,
) -> impl IntoView {
    let (task_name, set_task_name) = signal(String::new());
    let (task_description, set_task_description) = signal(String::new());
    let (is_creating, set_is_creating) = signal(false);

    let create_task = move |_| {
        let name = task_name.get();
        let description = task_description.get();

        if name.trim().is_empty() || is_creating.get() {
            return;
        }

        set_is_creating.set(true);
        web_sys::console::log_1(&format!("Creating task: {}", name).into());
        
        vm.with_value(|v| {
            v.create_task(
                name.trim().to_string(),
                description.trim().to_string(),
            )
        });
        
        // Clear the form
        set_task_name.set(String::new());
        set_task_description.set(String::new());
        set_is_creating.set(false);
        on_close.run(());
    };

    view! {
        <div class="task-creation-form">
            <h4 class="form-title">"Create New Task"</h4>

            <div class="form-group">
                <label class="form-label">"Task Name"</label>
                <input
                    type="text"
                    class="form-input"
                    placeholder="Enter task name..."
                    prop:value=move || task_name.get()
                    on:input=move |ev| {
                        set_task_name.set(event_target_value(&ev));
                    }
                    prop:disabled=move || is_creating.get()
                />
            </div>

            <div class="form-group">
                <label class="form-label">"Description (Optional)"</label>
                <textarea
                    class="form-textarea"
                    placeholder="Enter task description..."
                    prop:value=move || task_description.get()
                    on:input=move |ev| {
                        set_task_description.set(event_target_value(&ev));
                    }
                    prop:disabled=move || is_creating.get()
                    rows="3"
                ></textarea>
            </div>

            <div class="form-actions">
                <button
                    class="btn btn-primary"
                    prop:disabled=move || task_name.get().trim().is_empty() || is_creating.get()
                    on:click=create_task
                >
                    {move || if is_creating.get() { "Creating..." } else { "Create Task" }}
                </button>

                <button
                    class="btn btn-secondary"
                    on:click=move |_| on_close.run(())
                    prop:disabled=move || is_creating.get()
                >
                    "Cancel"
                </button>
            </div>
        </div>
    }
}
