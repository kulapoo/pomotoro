use domain::{Phase, TimerState, TimerStatus};
use leptos::prelude::*;

use super::TimerViewModel;

// Display & Formatting
impl TimerViewModel {
    pub fn get_phase_name(&self) -> String {
        let state = self.timer_state.get();
        match &state {
            TimerState::Idle { .. } => "Idle".to_string(),
            TimerState::Working { .. } => "Focus Time".to_string(),
            TimerState::ShortBreak { .. } => "Short Break".to_string(),
            TimerState::LongBreak { .. } => "Long Break".to_string(),
            TimerState::Paused { paused_from, .. } => {
                match paused_from.as_ref() {
                    TimerState::Working { .. } => {
                        "Focus Time (Paused)".to_string()
                    }
                    TimerState::ShortBreak { .. } => {
                        "Short Break (Paused)".to_string()
                    }
                    TimerState::LongBreak { .. } => {
                        "Long Break (Paused)".to_string()
                    }
                    _ => "Paused".to_string(),
                }
            }
        }
    }

    pub fn format_time(&self) -> String {
        let state = self.timer_state.get();
        let active_task = self.active_task.get();
        let mut seconds = state.remaining_seconds();
        let mut minutes = seconds / 60;
        let mut secs = seconds % 60;

        if state == TimerState::Idle {
            seconds = active_task.map(|t| t.config.timer.work_duration.as_secs() as u32).unwrap_or_default();
            minutes = seconds / 60;
            secs = seconds % 60;
        }

        format!("{minutes:02}:{secs:02}")
    }

    pub fn get_progress_percentage(&self) -> f64 {
        let state = self.timer_state.get();
        let config = self.timer_config.get();
        let remaining = state.remaining_seconds();
        let total = match &state {
            TimerState::Working { .. } => {
                config.get_phase_duration_seconds(Phase::Work)
            }
            TimerState::ShortBreak { .. } => {
                config.get_phase_duration_seconds(Phase::ShortBreak)
            }
            TimerState::LongBreak { .. } => {
                config.get_phase_duration_seconds(Phase::LongBreak)
            }
            TimerState::Paused { paused_from, .. } => {
                let phase = Self::get_current_phase_static(paused_from);
                config.get_phase_duration_seconds(phase)
            }
            TimerState::Idle => return 0.0,
        };

        if total == 0 {
            0.0
        } else {
            ((total - remaining) as f64 / total as f64) * 100.0
        }
    }

    pub fn get_session_display(&self) -> String {
        if let Some(task) = self.active_task.get() {
            let config = self.timer_config.get();
            let sessions_until_long_break =
                config.sessions_until_long_break as u32;
            format!(
                "Session {}/{}",
                (task.current_sessions % sessions_until_long_break as u8) + 1,
                sessions_until_long_break
            )
        } else {
            "No active task".to_string()
        }
    }

    pub fn get_start_pause_button_text(&self) -> &'static str {
        match self.timer_state.get().status() {
            TimerStatus::Running => "Pause",
            _ => "Start",
        }
    }

    pub fn get_sessions_completed(&self) -> usize {
        if let Some(task) = self.active_task.get() {
            let config = self.timer_config.get();
            (task.current_sessions % config.sessions_until_long_break) as usize
        } else {
            0
        }
    }

    pub fn get_today_pomodoros(&self) -> u32 {
        // This would typically come from a stats service, for now return task sessions
        if let Some(task) = self.active_task.get() {
            task.current_sessions as u32
        } else {
            0
        }
    }

    pub fn get_task_pomodoros(&self) -> u32 {
        if let Some(task) = self.active_task.get() {
            task.current_sessions as u32
        } else {
            0
        }
    }
}
