use leptos::prelude::*;
use leptos::callback::Callback;
use crate::pages::task::TasksViewModel;

#[component]
pub fn TaskCreationForm(
    vm: StoredValue<TasksViewModel>,
    on_close: Callback<()>,
) -> impl IntoView {
    let (task_name, set_task_name) = signal(String::new());
    let (task_description, set_task_description) = signal(String::new());
    
    let create_task = move |_| {
        let name = task_name.get();
        let description = task_description.get();
        
        if name.trim().is_empty() {
            return;
        }
        
        vm.with_value(|v| v.create_task(name.trim().to_string(), description.trim().to_string()));
        on_close.run(());
    };

    view! {
        <div class="bg-gray-50 dark:bg-slate-800 border border-gray-200 dark:border-slate-600 rounded-xl p-4 mb-4">
            <h4 class="m-0 mb-4 text-base font-semibold text-gray-900 dark:text-white">"Create New Task"</h4>
            
            <div class="mb-3">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Task Name"</label>
                <input
                    type="text"
                    class="w-full px-3 py-2 border border-gray-300 dark:border-slate-600 rounded-md text-sm bg-white dark:bg-slate-700 text-gray-900 dark:text-white transition-colors focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-200 dark:focus:ring-blue-300"
                    placeholder="Enter task name..."
                    prop:value=move || task_name.get()
                    on:input=move |ev| {
                        set_task_name.set(event_target_value(&ev));
                    }
                />
            </div>
            
            <div class="mb-3">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description (Optional)"</label>
                <textarea
                    class="w-full px-3 py-2 border border-gray-300 dark:border-slate-600 rounded-md text-sm bg-white dark:bg-slate-700 text-gray-900 dark:text-white transition-colors resize-y min-h-15 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-200 dark:focus:ring-blue-300"
                    placeholder="Enter task description..."
                    prop:value=move || task_description.get()
                    on:input=move |ev| {
                        set_task_description.set(event_target_value(&ev));
                    }
                ></textarea>
            </div>
            
            <div class="flex gap-2 justify-end mt-4">
                <button
                    class="bg-gradient-to-br from-blue-500 to-blue-600 text-white border-none rounded-lg px-4 py-2 text-sm font-medium cursor-pointer transition-all duration-200 hover:from-blue-600 hover:to-blue-700 hover:-translate-y-0.5 disabled:opacity-60 disabled:cursor-not-allowed disabled:transform-none"
                    prop:disabled=move || task_name.get().trim().is_empty()
                    on:click=create_task
                >
                    "Create Task"
                </button>
                
                <button
                    class="bg-gray-100 dark:bg-slate-600 text-gray-700 dark:text-slate-200 border border-gray-200 dark:border-slate-500 rounded-lg px-4 py-2 text-sm font-medium cursor-pointer transition-all duration-200 hover:bg-gray-200 dark:hover:bg-slate-500"
                    on:click=move |_| on_close.run(())
                >
                    "Cancel"
                </button>
            </div>
        </div>
    }
}