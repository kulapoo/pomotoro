use leptos::prelude::*;
use domain::TimerState;
use domain::TimerStateWithTask;
use crate::components::circular_progress::CircularProgress;

#[component]
pub fn TimerDisplay(
    timer_state: ReadSignal<TimerState>, 
    timer_with_task: ReadSignal<TimerStateWithTask>
) -> impl IntoView {
    view! {
        <div class="mb-8">
            // Session label above timer
            <h2 class="timer-label">
                {move || timer_state.get().get_phase_name()}
            </h2>
        </div>

        // Large timer display with circular progress
        <div class="relative flex items-center justify-center my-12">
            <CircularProgress
                progress=Signal::derive(move || timer_with_task.get().get_progress_percentage())
                phase=Signal::derive(move || timer_state.get().phase())
            />
            <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
                <div class="text-center">
                    // Large digital display - 72px font size
                    <div class="timer-display">
                        {move || timer_with_task.get().format_time()}
                    </div>
                </div>
            </div>
        </div>
    }
}