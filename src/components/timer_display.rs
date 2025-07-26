use leptos::prelude::*;
use crate::store::TimerState;
use pomotoro_domain::TimerStateWithTask;
use super::circular_progress::CircularProgress;

#[component]
pub fn TimerDisplay(
    timer_state: ReadSignal<TimerState>, 
    timer_with_task: ReadSignal<TimerStateWithTask>
) -> impl IntoView {
    view! {
        <div class="timer-header">
            <div class="active-task">
                <span class="task-label">"Current Task:"</span>
                <span class="task-name">{move || timer_with_task.get().get_active_task_name()}</span>
            </div>
            <h1 class="phase-title">{move || timer_state.get().get_phase_name()}</h1>
            <div class="session-counter">
                {move || timer_state.get().get_session_display()}
            </div>
        </div>

        <div class="timer-display">
            <CircularProgress
                progress=Signal::derive(move || timer_with_task.get().get_progress_percentage())
                phase=Signal::derive(move || timer_state.get().phase())
            />
            <div class="time-overlay">
                <span class="time-text">{move || timer_with_task.get().format_time()}</span>
            </div>
        </div>
    }
}