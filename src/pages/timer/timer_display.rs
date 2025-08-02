use leptos::prelude::*;
use super::timer_state::TimerState;
use pomotoro_domain::TimerStateWithTask;
use crate::components::circular_progress::CircularProgress;

#[component]
pub fn TimerDisplay(
    timer_state: ReadSignal<TimerState>, 
    timer_with_task: ReadSignal<TimerStateWithTask>
) -> impl IntoView {
    view! {
        <div class="mb-8">
            <div class="mb-4 p-2 bg-blue-100 dark:bg-blue-900/20 rounded-xl border border-blue-200 dark:border-blue-700/40">
                <span class="text-sm text-gray-600 dark:text-gray-300 font-medium mr-2">"Current Task:"</span>
                <span class="text-base text-gray-900 dark:text-white font-semibold">{move || timer_with_task.get().get_active_task_name()}</span>
            </div>
            <h1 class="text-4xl md:text-4xl text-3xl font-bold text-gray-900 dark:text-white mb-2 text-shadow-sm">{move || timer_state.get().get_phase_name()}</h1>
            <div class="text-lg text-gray-600 dark:text-gray-300 font-medium">
                {move || timer_state.get().get_session_display()}
            </div>
        </div>

        <div class="relative flex items-center justify-center my-10">
            <CircularProgress
                progress=Signal::derive(move || timer_with_task.get().get_progress_percentage())
                phase=Signal::derive(move || timer_state.get().phase())
            />
            <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
                <span class="text-5xl md:text-5xl text-4xl font-bold text-gray-900 dark:text-white font-mono text-shadow-sm">{move || timer_with_task.get().format_time()}</span>
            </div>
        </div>
    }
}