use domain::{Phase, Task, TimerState};
use leptos::prelude::*;
use crate::components::ErrorInfo;

use super::AppViewModel;

impl AppViewModel {
    pub fn timer_state(&self) -> ReadSignal<TimerState> {
        self.timer_state
    }

    pub fn active_task(&self) -> ReadSignal<Option<Task>> {
        self.active_task
    }

    pub fn set_active_task(&self) -> WriteSignal<Option<Task>> {
        self.set_active_task
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

    // Active task helper methods
    pub fn get_active_task(&self) -> Option<Task> {
        self.active_task.get()
    }

    pub fn get_active_task_name(&self) -> String {
        self.active_task
            .get()
            .map(|task| task.name.clone())
            .unwrap_or_else(|| "No active task".to_string())
    }

    pub fn get_active_entity_id(&self) -> Option<String> {
        self.active_task.get().map(|task| task.id.to_string())
    }

    pub fn is_active_task_completed(&self) -> bool {
        self.active_task.get().map(|t| t.is_completed()).unwrap_or(false)
    }
}