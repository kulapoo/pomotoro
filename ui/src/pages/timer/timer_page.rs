use leptos::prelude::*;
use crate::pages::timer::{TimerControls, TimerViewModel};
use crate::shared::ViewModel;
use crate::components::CircularProgress;

#[component]
pub fn TimerPage() -> impl IntoView {
    let vm = StoredValue::new(TimerViewModel::new());
    let timer_state = vm.with_value(|v| v.state());
    
    view! {
        <div class="timer-section">
            <div class="current-task">
                <div class="task-label">"Current Task"</div>
                <div class="task-title">{move || {
                    let state = timer_state.get();
                    state.active_entity_id().map(|_| "Active Task".to_string()).unwrap_or_else(|| "No active task".to_string())
                }}</div>
                <div class="task-progress">{move || vm.with_value(|v| v.get_session_display())}</div>
            </div>
            
            <div class="timer-label">{move || vm.with_value(|v| v.get_phase_name())}</div>
            
            <CircularProgress
                progress=Signal::derive(move || vm.with_value(|v| v.get_progress_percentage()))
                phase=Signal::derive(move || timer_state.get().phase())
            />
            
            <div class="timer-display">{move || vm.with_value(|v| v.format_time())}</div>
            
            <div class="timer-controls">
                <TimerControls vm=vm />
            </div>
        </div>
    }
}