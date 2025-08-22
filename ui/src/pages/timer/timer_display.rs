use leptos::prelude::*;
use domain::TimerState;
use crate::components::circular_progress::CircularProgress;
use crate::pages::timer::timer_view_model::TimerViewModel;

#[component]
#[allow(dead_code)]
pub fn TimerDisplay(
    timer_state: ReadSignal<TimerState>
) -> impl IntoView {
    view! {
        <div class="mb-8">
            // Session label above timer
            <h2 class="timer-label">
                {move || timer_state.get().phase().name()}
            </h2>
        </div>

        // Large timer display with circular progress
        <div class="relative flex items-center justify-center my-12">
            <CircularProgress
                progress=Signal::derive(move || TimerViewModel::get_progress_percentage(&timer_state.get()))
                phase=Signal::derive(move || timer_state.get().phase())
            />
            <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
                <div class="text-center">
                    // Large digital display - 72px font size
                    <div class="timer-display">
                        {move || TimerViewModel::format_time(&timer_state.get())}
                    </div>
                </div>
            </div>
        </div>
    }
}