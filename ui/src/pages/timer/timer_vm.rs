use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen;
use domain::{TimerState, TimerStatus, Phase, event_names};

use crate::utils::{invoke_command_no_args, setup_phase_complete_events, setup_timer_events, ViewModel};

pub struct TimerViewModel {
    timer_state: ReadSignal<TimerState>,
    set_timer_state: WriteSignal<TimerState>,
}

impl ViewModel for TimerViewModel {
    type State = TimerState;

    fn new() -> Self {
        let (timer_state, set_timer_state) = signal(TimerState::default());

        let vm = Self {
            timer_state,
            set_timer_state,
        };

        vm.initialize();
        vm
    }

    fn state(&self) -> ReadSignal<Self::State> {
        self.timer_state
    }

    fn set_state(&self) -> WriteSignal<Self::State> {
        self.set_timer_state
    }
}

impl TimerViewModel {
    fn initialize(&self) {
        let set_timer_state = self.set_timer_state;

        Effect::new(move |_| {
            spawn_local(async move {
                if let Ok(result) = invoke_command_no_args(event_names::timer::GET_STATE).await {
                    if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                        set_timer_state.set(state);
                    }
                }
            });
        });

        setup_timer_events(set_timer_state);
        setup_phase_complete_events();
    }

    pub fn start_pause_timer(&self) {
        let current_state = self.timer_state.get_untracked();
        let set_timer_state = self.set_timer_state;

        spawn_local(async move {
            let command = match current_state.status() {
                TimerStatus::Running => event_names::timer::PAUSE,
                _ => event_names::timer::START,
            };

            if let Ok(result) = invoke_command_no_args(command).await {
                if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                    set_timer_state.set(state);
                }
            }
        });
    }

    pub fn reset_timer(&self) {
        let set_timer_state = self.set_timer_state;

        spawn_local(async move {
            if let Ok(result) = invoke_command_no_args(event_names::timer::RESET).await {
                if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                    set_timer_state.set(state);
                }
            }
        });
    }

    pub fn skip_phase(&self) {
        let set_timer_state = self.set_timer_state;

        spawn_local(async move {
            if let Ok(result) = invoke_command_no_args(event_names::timer::SKIP_PHASE).await {
                if let Ok(state) = serde_wasm_bindgen::from_value::<TimerState>(result) {
                    set_timer_state.set(state);
                }
            }
        });
    }

    pub fn get_phase_name(&self) -> String {
        let state = self.timer_state.get();
        match &state {
            TimerState::Idle { .. } => "Idle".to_string(),
            TimerState::Working { .. } => "Focus Time".to_string(),
            TimerState::ShortBreak { .. } => "Short Break".to_string(),
            TimerState::LongBreak { .. } => "Long Break".to_string(),
            TimerState::Paused { paused_from, .. } => {
                match paused_from.as_ref() {
                    TimerState::Working { .. } => "Focus Time (Paused)".to_string(),
                    TimerState::ShortBreak { .. } => "Short Break (Paused)".to_string(),
                    TimerState::LongBreak { .. } => "Long Break (Paused)".to_string(),
                    _ => "Paused".to_string(),
                }
            }
        }
    }

    pub fn format_time(&self) -> String {
        let state = self.timer_state.get();
        let seconds = state.remaining_seconds();
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", minutes, secs)
    }

    pub fn get_progress_percentage(&self) -> f64 {
        let state = self.timer_state.get();
        let remaining = state.remaining_seconds();
        let total = match &state {
            TimerState::Working { configuration, .. } => {
                configuration.get_phase_duration_seconds(Phase::Work)
            }
            TimerState::ShortBreak { configuration, .. } => {
                configuration.get_phase_duration_seconds(Phase::ShortBreak)
            }
            TimerState::LongBreak { configuration, .. } => {
                configuration.get_phase_duration_seconds(Phase::LongBreak)
            }
            TimerState::Paused { paused_from, .. } => {
                let phase = Self::get_current_phase_static(paused_from);
                paused_from.configuration().get_phase_duration_seconds(phase)
            }
            TimerState::Idle { .. } => return 0.0,
        };

        if total == 0 {
            0.0
        } else {
            ((total - remaining) as f64 / total as f64) * 100.0
        }
    }

    pub fn get_session_display(&self) -> String {
        let state = self.timer_state.get();
        let session_count = state.session_count();
        let sessions_until_long_break = state.configuration().sessions_until_long_break as u32;
        format!("Session {}/{}", session_count % sessions_until_long_break + 1, sessions_until_long_break)
    }

    pub fn get_start_pause_button_text(&self) -> &'static str {
        match self.timer_state.get().status() {
            TimerStatus::Running => "Pause",
            _ => "Start"
        }
    }

    fn get_current_phase_static(state: &TimerState) -> Phase {
        match state {
            TimerState::Working { .. } => Phase::Work,
            TimerState::ShortBreak { .. } => Phase::ShortBreak,
            TimerState::LongBreak { .. } => Phase::LongBreak,
            TimerState::Idle { .. } => Phase::Work,
            TimerState::Paused { paused_from, .. } => Self::get_current_phase_static(paused_from),
        }
    }

    pub fn get_sessions_completed(&self) -> usize {
        let state = self.timer_state.get();
        (state.session_count() % state.configuration().sessions_until_long_break as u32) as usize
    }

    pub fn get_today_pomodoros(&self) -> u32 {
        // This would typically come from a stats service, for now return session count
        let state = self.timer_state.get();
        state.session_count()
    }

    pub fn get_task_pomodoros(&self) -> u32 {
        // This would typically track pomodoros for the active task
        // For now, return a default value
        0
    }
}