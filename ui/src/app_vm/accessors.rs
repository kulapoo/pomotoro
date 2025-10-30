use domain::{Phase, TimerState};
use leptos::prelude::*;
use crate::components::ErrorInfo;

use super::AppViewModel;

impl AppViewModel {
    pub fn timer_state(&self) -> ReadSignal<TimerState> {
        self.timer_state
    }

    pub fn error_state(&self) -> ReadSignal<Option<ErrorInfo>> {
        self.error_state
    }

    pub fn format_time(&self) -> String {
        let state = self.timer_state.get();
        let seconds = state.remaining_seconds();
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{minutes:02}:{secs:02}")
    }

    pub fn is_running(&self) -> bool {
        self.timer_state.get().is_running()
    }

    pub fn is_paused(&self) -> bool {
        self.timer_state.get().is_paused()
    }

    pub fn is_idle(&self) -> bool {
        self.timer_state.get().is_idle()
    }

    pub fn current_phase(&self) -> Option<Phase> {
        let state = self.timer_state.get();
        match state {
            TimerState::Working { .. } => Some(Phase::Work),
            TimerState::ShortBreak { .. } => Some(Phase::ShortBreak),
            TimerState::LongBreak { .. } => Some(Phase::LongBreak),
            TimerState::Paused { paused_from, .. } => {
                // Get phase from the paused state
                match paused_from.as_ref() {
                    TimerState::Working { .. } => Some(Phase::Work),
                    TimerState::ShortBreak { .. } => Some(Phase::ShortBreak),
                    TimerState::LongBreak { .. } => Some(Phase::LongBreak),
                    _ => None,
                }
            }
            TimerState::Idle => None,
        }
    }

    pub fn phase_name(&self) -> String {
        self.current_phase()
            .map(|phase| phase.name().to_string())
            .unwrap_or_else(|| "Idle".to_string())
    }

    // System tray helper methods
    pub fn tray_display(&self) -> String {
        if self.is_idle() {
            "Pomotoro - Ready".to_string()
        } else {
            let time = self.format_time();
            let phase = self.phase_name();
            let status = if self.is_paused() { " (Paused)" } else { "" };
            format!("{phase}: {time}{status}")
        }
    }

    pub fn tray_tooltip(&self) -> String {
        if self.is_idle() {
            "Click to start a work session".to_string()
        } else if self.is_paused() {
            format!("Timer paused - {} remaining", self.format_time())
        } else {
            format!("{} - {} remaining", self.phase_name(), self.format_time())
        }
    }
}