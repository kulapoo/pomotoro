use domain::{TimerState, Phase};

/// View model for timer presentation logic
pub struct TimerViewModel;

impl TimerViewModel {
    /// Get human-readable phase name
    pub fn get_phase_name(state: &TimerState) -> &'static str {
        match state {
            TimerState::Idle { .. } => "Idle",
            TimerState::Working { .. } => "Focus Time",
            TimerState::ShortBreak { .. } => "Short Break",
            TimerState::LongBreak { .. } => "Long Break",
            TimerState::Paused { paused_from, .. } => {
                match paused_from.as_ref() {
                    TimerState::Working { .. } => "Focus Time (Paused)",
                    TimerState::ShortBreak { .. } => "Short Break (Paused)",
                    TimerState::LongBreak { .. } => "Long Break (Paused)",
                    _ => "Paused",
                }
            }
        }
    }
    
    /// Format remaining time as MM:SS
    pub fn format_time(state: &TimerState) -> String {
        let seconds = state.remaining_seconds();
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", minutes, secs)
    }
    
    /// Get progress percentage (0.0 to 100.0)
    pub fn get_progress_percentage(state: &TimerState) -> f64 {
        let remaining = state.remaining_seconds();
        let total = match state {
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
                let phase = Self::get_current_phase(paused_from);
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
    
    /// Get session display string
    pub fn get_session_display(state: &TimerState) -> String {
        let session_count = state.session_count();
        let sessions_until_long_break = state.configuration().sessions_until_long_break as u32;
        format!("Session {}/{}", session_count % sessions_until_long_break + 1, sessions_until_long_break)
    }
    
    /// Helper to get current phase from state
    fn get_current_phase(state: &TimerState) -> Phase {
        match state {
            TimerState::Working { .. } => Phase::Work,
            TimerState::ShortBreak { .. } => Phase::ShortBreak,
            TimerState::LongBreak { .. } => Phase::LongBreak,
            TimerState::Idle { .. } => Phase::Work, // Default to work
            TimerState::Paused { paused_from, .. } => Self::get_current_phase(paused_from),
        }
    }
}