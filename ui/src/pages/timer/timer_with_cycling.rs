use crate::components::{ErrorToast, TaskCycleControls, TaskCompletionIndicator};
use crate::pages::timer::{TimerControls, TimerViewModel};
use crate::pages::task::TasksViewModel;
use crate::utils::ViewModel;
use leptos::prelude::*;
use web_sys;

#[component]
pub fn TimerPageWithCycling() -> impl IntoView {
    let timer_vm = StoredValue::new(TimerViewModel::new());
    let tasks_vm = StoredValue::new(TasksViewModel::new());
    
    let timer_state = timer_vm.with_value(|v| v.state());
    let error_state = timer_vm.with_value(|v| v.error_state());
    let set_error_state = timer_vm.with_value(|v| v.set_error_state());
    
    let active_task = tasks_vm.with_value(|v| v.get_active_task());
    let cycle_position = tasks_vm.with_value(|v| v.get_cycle_position());
    
    let is_timer_active = Signal::derive(move || {
        let state = timer_state.get();
        !matches!(state, domain::TimerState::Idle { .. })
    });
    
    let handle_next_task = move || {
        tasks_vm.with_value(|v| v.cycle_to_next_incomplete_task());
    };
    
    let handle_previous_task = move || {
        tasks_vm.with_value(|v| v.cycle_to_previous_incomplete_task());
    };

    view! {
        <ErrorToast error_signal=error_state set_error=set_error_state />
        <div class="timer-container">
            <div class="current-task-section">
                <TaskCycleControls 
                    on_next=handle_next_task
                    on_previous=handle_previous_task
                    position=cycle_position
                    is_active=is_timer_active
                />
                <TaskCompletionIndicator task=active_task />
                <div class="current-task-display" id="currentTaskDisplay">
                    "Working on: "{move || {
                        active_task.get()
                            .map(|t| t.name)
                            .unwrap_or_else(|| "No task selected".to_string())
                    }}
                </div>
            </div>

            <div class="timer-label" id="timerLabel">
                {move || timer_vm.with_value(|v| v.get_phase_name())}
            </div>
            <div class="timer-display" id="timerDisplay">
                {move || timer_vm.with_value(|v| v.format_time())}
            </div>

            <div class="timer-controls">
                <TimerControls vm=timer_vm />
            </div>

            <div class="session-dots" id="sessionDots">
                {move || {
                    let sessions_completed = timer_vm.with_value(|v| v.get_sessions_completed());
                    (0..4).map(|i| {
                        let class = if i < sessions_completed { 
                            "dot completed" 
                        } else { 
                            "dot" 
                        };
                        view! { <div class=class></div> }
                    }).collect_view()
                }}
            </div>

            <div class="pomodoro-stats">
                <div class="stat-item">
                    <div class="stat-value" id="todayPomodoros">
                        {move || timer_vm.with_value(|v| v.get_today_pomodoros())}
                    </div>
                    <div class="stat-label">"Today's Pomodoros"</div>
                </div>
                <div class="stat-item">
                    <div class="stat-value" id="taskPomodoros">
                        {move || {
                            active_task.get()
                                .map(|t| t.current_sessions)
                                .unwrap_or(0)
                        }}
                    </div>
                    <div class="stat-label">"Task Sessions"</div>
                </div>
            </div>
            
            <div class="keyboard-shortcuts-hint">
                <span>"Ctrl+Tab: Next Task"</span>
                <span>" | "</span>
                <span>"Ctrl+Shift+Tab: Previous Task"</span>
            </div>
        </div>
    }
}