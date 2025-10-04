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

    web_sys::console::log_1(&format!("Timer state: {:?}", timer_state.get_untracked()).into());

    view! {
        <ErrorToast error_signal=error_state set_error=set_error_state />
        <div class="timer-container">
            <div class="current-task-display" id="currentTaskDisplay">
                "Working on: "{move || vm.with_value(|v| v.get_active_task_name())}
            </div>

            <div class="timer-label" id="timerLabel">{move || vm.with_value(|v| v.get_phase_name())}</div>
            <div class="timer-display" id="timerDisplay">{move || vm.with_value(|v| v.format_time())}</div>

            <div class="timer-controls">
                <TimerControls vm=vm />
            </div>

            <div class="session-dots" id="sessionDots">
                {move || {
                    let sessions_completed = vm.with_value(|v| v.get_sessions_completed());
                    (0..4).map(|i| {
                        let class = if i < sessions_completed { "dot completed" } else { "dot" };
                        view! { <div class=class></div> }
                    }).collect_view()
                }}
            </div>

            <div class="pomodoro-stats">
                <div class="stat-item">
                    <div class="stat-value" id="taskPomodoros">{move || vm.with_value(|v| v.get_task_pomodoros())}</div>
                    <div class="stat-label">"Task Pomodoros"</div>
                </div>
            </div>
        </div>
    }
}
