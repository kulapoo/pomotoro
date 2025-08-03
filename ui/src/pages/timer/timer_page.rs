use leptos::prelude::*;
use crate::pages::timer::TimerControls;
use crate::pages::timer::timer_state::TimerPageState;

#[component]
pub fn TimerPage() -> impl IntoView {
    let page_state = TimerPageState::new();
    
    view! {
        <div class="timer-section">
            // Current Task Context Box
            <div class="current-task">
                <div class="task-label">"Current Task"</div>
                <div class="task-title">{move || page_state.timer_with_task.get().get_active_task_name()}</div>
                <div class="task-progress">{move || page_state.timer_state.get().get_session_display()}</div>
            </div>
            
            // Timer label
            <div class="timer-label">{move || page_state.timer_state.get().get_phase_name()}</div>
            
            // Progress ring
            <div class="progress-ring">
                <div class="progress-circle"></div>
            </div>
            
            // Timer display
            <div class="timer-display">{move || page_state.timer_with_task.get().format_time()}</div>
            
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