use crate::components::ErrorToast;
use crate::pages::timer::{TimerControls, TimerViewModel};
use crate::utils::ViewModel;
use leptos::prelude::*;
use web_sys;

#[component]
pub fn TimerPage() -> impl IntoView {
    let vm = StoredValue::new(TimerViewModel::new());
    let timer_state = vm.with_value(|v| v.state());
    let error_state = vm.with_value(|v| v.error_state());
    let set_error_state = vm.with_value(|v| v.set_error_state());

    web_sys::console::log_1(
        &format!("Timer state: {:?}", timer_state.get_untracked()).into(),
    );

    view! {
        <ErrorToast error_signal=error_state set_error=set_error_state />
        <div class="max-w-2xl mx-auto px-4 py-8">
            <div class="bg-white rounded-lg shadow-md p-6 mb-6 text-center" id="currentTaskDisplay">
                <p class="text-lg text-slate-700">
                    <span class="font-semibold text-indigo-600">"Working on: "</span>
                    {move || vm.with_value(|v| v.get_active_task_name())}
                </p>
                {move || vm.with_value(|v| v.get_active_entity_id()).map(|id| {
                    let short_id = id.chars().take(8).collect::<String>();
                    view! {
                        <p class="text-xs text-slate-400 font-mono mt-1">
                            {format!("ID: {}", short_id)}
                        </p>
                    }
                })}
            </div>

            <div class="text-center mb-4">
                <div class="text-2xl font-semibold text-slate-800 mb-2" id="timerLabel">
                    {move || vm.with_value(|v| v.get_phase_name())}
                </div>
                <div class="text-6xl font-bold text-indigo-600 mb-8" id="timerDisplay">
                    {move || vm.with_value(|v| v.format_time())}
                </div>
            </div>

            <div class="flex flex-wrap gap-3 justify-center mb-8">
                <TimerControls vm=vm />
            </div>

            <div class="flex justify-center gap-2 mb-8" id="sessionDots">
                {move || {
                    let sessions_completed = vm.with_value(|v| v.get_sessions_completed());
                    let max_sessions = vm.with_value(|v| v.get_active_task())
                        .map(|task| task.max_sessions as usize)
                        .unwrap_or(4);
                    (0..max_sessions).map(|i| {
                        let class = if i < sessions_completed {
                            "w-3 h-3 rounded-full bg-indigo-600 shadow-sm"
                        } else {
                            "w-3 h-3 rounded-full bg-slate-300"
                        };
                        view! { <div class=class></div> }
                    }).collect_view()
                }}
            </div>

            <div class="bg-white rounded-lg shadow-md p-6">
                <div class="text-center">
                    <div class="text-4xl font-bold text-indigo-600 mb-2" id="taskPomodoros">
                        {move || vm.with_value(|v| v.get_task_pomodoros())}
                    </div>
                    <div class="text-sm text-slate-600 uppercase tracking-wide">"Phase(s)"</div>
                </div>
            </div>
        </div>
    }
}
