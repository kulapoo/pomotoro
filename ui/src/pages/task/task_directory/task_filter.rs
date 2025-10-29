use super::TaskDirectoryViewModel;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn TaskFilter(vm: StoredValue<TaskDirectoryViewModel>) -> impl IntoView {
    let handle_search = move |ev: leptos::ev::Event| {
        if let Some(target) = ev.target() {
            let input_elem = target.unchecked_into::<web_sys::HtmlInputElement>();
            let query = input_elem.value();
            vm.with_value(|v| v.search_tasks(query));
        }
    };

    let handle_sort_change = move |ev: leptos::ev::Event| {
        if let Some(target) = ev.target() {
            let select = target.unchecked_into::<web_sys::HtmlSelectElement>();
            let sort_by = select.value();
            vm.with_value(|v| v.set_sort(sort_by));
        }
    };

    let handle_status_change = move |ev: leptos::ev::Event| {
        if let Some(target) = ev.target() {
            let select = target.unchecked_into::<web_sys::HtmlSelectElement>();
            let status = select.value();
            vm.with_value(|v| v.set_status_filter(status));
        }
    };

    view! {
        <div class="mb-6">
            <div class="bg-white rounded-lg shadow-sm p-4 mb-4">
                <div class="relative">
                    <input
                        type="text"
                        placeholder="Search tasks by name, description, or tags..."
                        class="w-full px-4 py-3 pr-10 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:border-transparent transition-all"
                        on:input=handle_search
                    />
                    <svg class="absolute right-3 top-1/2 transform -translate-y-1/2 text-slate-400" width="20" height="20" viewBox="0 0 20 20" fill="none">
                        <path
                            d="M9 17A8 8 0 109 1a8 8 0 000 16zM19 19l-4.35-4.35"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                </div>
            </div>

            <div class="flex flex-col sm:flex-row gap-4 mb-4">
                <div class="flex-1">
                    <label for="sort-select" class="block text-sm font-medium text-slate-700 mb-2">Sort by:</label>
                    <select
                        id="sort-select"
                        class="w-full px-4 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:border-transparent transition-all bg-white"
                        on:change=handle_sort_change
                        prop:value=move || vm.with_value(|v| v.get_sort_by())
                    >
                        <option value="name">Name</option>
                        <option value="created_at" selected>Created Date</option>
                        <option value="sessions_completed">Sessions Completed</option>
                        <option value="status">Status</option>
                    </select>
                </div>

                <div class="flex-1">
                    <label for="status-select" class="block text-sm font-medium text-slate-700 mb-2">Status:</label>
                    <select
                        id="status-select"
                        class="w-full px-4 py-2 border border-slate-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:border-transparent transition-all bg-white"
                        on:change=handle_status_change
                        prop:value=move || vm.with_value(|v| v.get_status_filter())
                    >
                        <option value="all" selected>All Tasks</option>
                        <option value="active">Active</option>
                        <option value="queued">Queued</option>
                        <option value="paused">Paused</option>
                        <option value="completed">Completed</option>
                    </select>
                </div>
            </div>

            <div class="text-sm text-slate-600">
                {move || {
                    let total = vm.with_value(|v| v.get_tasks().len());
                    let query = vm.with_value(|v| v.get_search_query());
                    let status = vm.with_value(|v| v.get_status_filter());

                    if !query.is_empty() || status != "all" {
                        view! {
                            <span>
                                {format!("Found {} task{}", total, if total == 1 { "" } else { "s" })}
                            </span>
                        }.into_any()
                    } else {
                        view! {
                            <span>
                                {format!("{} total task{}", total, if total == 1 { "" } else { "s" })}
                            </span>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}