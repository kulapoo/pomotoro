use leptos::prelude::*;
use crate::store::TimerState;
use super::circular_progress::CircularProgress;

#[component]
pub fn TimerDisplay(timer_state: ReadSignal<TimerState>) -> impl IntoView {
    view! {
        <div class="timer-header">
            <h1 class="phase-title">{move || timer_state.get().get_phase_name()}</h1>
            <div class="session-counter">
                {move || timer_state.get().get_session_display()}
            </div>
        </div>

        <div class="timer-display">
            <CircularProgress
                progress=Signal::derive(move || timer_state.get().get_progress_percentage())
                phase=Signal::derive(move || timer_state.get().phase.clone())
            />
            <div class="time-overlay">
                <span class="time-text">{move || timer_state.get().format_time()}</span>
            </div>
        </div>
    }
}