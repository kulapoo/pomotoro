use leptos::prelude::*;
use crate::pages::timer::TimerControls;
use crate::pages::timer::timer_state::TimerPageState;
use crate::pages::timer::timer_view_model::TimerViewModel;
use crate::components::CircularProgress;

#[component]
pub fn TimerPage() -> impl IntoView {
    let page_state = TimerPageState::new();
    
    view! {
        <div class="timer-section">
            // Current Task Context Box
            <div class="current-task">
                <div class="task-label">"Current Task"</div>
                <div class="task-title">{move || {
                    let state = page_state.timer_state.get();
                    state.active_task().map(|_| "Active Task".to_string()).unwrap_or_else(|| "No active task".to_string())
                }}</div>
                <div class="task-progress">{move || TimerViewModel::get_session_display(&page_state.timer_state.get())}</div>
            </div>
            
            // Timer label
            <div class="timer-label">{move || TimerViewModel::get_phase_name(&page_state.timer_state.get())}</div>
            
            // Progress ring
            <CircularProgress
                progress=Signal::derive(move || TimerViewModel::get_progress_percentage(&page_state.timer_state.get()))
                phase=Signal::derive(move || page_state.timer_state.get().phase())
            />
            
            // Timer display
            <div class="timer-display">{move || TimerViewModel::format_time(&page_state.timer_state.get())}</div>
            
            // Timer controls
            <div class="timer-controls">
                <TimerControls
                    timer_state=page_state.timer_state
                    set_timer_state=page_state.set_timer_state
                />
            </div>
        </div>
    }
}