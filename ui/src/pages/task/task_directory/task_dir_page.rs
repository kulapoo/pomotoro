use super::{TaskDirectoryViewModel, TaskList, TaskFilter};
use crate::utils::ViewModel;
use leptos::prelude::*;

#[component]
pub fn TaskDirectoryPage() -> impl IntoView {
    let vm = StoredValue::new(TaskDirectoryViewModel::new());

    view! {
        <div class="max-w-5xl mx-auto">
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center mb-6 gap-4">
                <h2 class="text-3xl font-bold text-slate-800">"My Tasks"</h2>
                <div class="flex gap-3">
                    {move || {
                        let tasks = vm.with_value(|v| v.get_tasks());
                        let total = tasks.len();
                        let completed = tasks.iter().filter(|t| t.status == domain::TaskStatus::Completed).count();
                        let active = tasks.iter().filter(|t| t.status == domain::TaskStatus::Active).count();
                        view! {
                            <div class="flex flex-col items-center px-4 py-2 bg-white rounded-lg shadow-sm border border-slate-200">
                                <span class="text-2xl font-bold text-slate-800">{total}</span>
                                <span class="text-xs text-slate-600 uppercase tracking-wide">"Total"</span>
                            </div>
                            <div class="flex flex-col items-center px-4 py-2 bg-indigo-600/10 rounded-lg shadow-sm border border-indigo-600/20">
                                <span class="text-2xl font-bold text-indigo-600">{active}</span>
                                <span class="text-xs text-indigo-600 uppercase tracking-wide">"Active"</span>
                            </div>
                            <div class="flex flex-col items-center px-4 py-2 bg-emerald-500/10 rounded-lg shadow-sm border border-emerald-500/20">
                                <span class="text-2xl font-bold text-emerald-500">{completed}</span>
                                <span class="text-xs text-emerald-500 uppercase tracking-wide">"Completed"</span>
                            </div>
                        }
                    }}
                </div>
            </div>
            <TaskFilter vm=vm />
            <TaskList vm=vm />
        </div>
    }
}