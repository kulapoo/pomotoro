use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimerStatus {
    Stopped,
    Running,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub status: TimerStatus,
    pub phase: Phase,
    pub remaining_seconds: u32,
    pub session_count: u32,
    pub is_break_cycle: bool,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            status: TimerStatus::Stopped,
            phase: Phase::Work,
            remaining_seconds: 25 * 60,
            session_count: 0,
            is_break_cycle: false,
        }
    }
}

impl TimerState {
    pub fn format_time(&self) -> String {
        let minutes = self.remaining_seconds / 60;
        let seconds = self.remaining_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
    
    pub fn get_phase_name(&self) -> &'static str {
        match self.phase {
            Phase::Work => "Focus Time",
            Phase::ShortBreak => "Short Break",
            Phase::LongBreak => "Long Break",
        }
    }
    
    pub fn get_progress_percentage(&self) -> f64 {
        let total_duration = match self.phase {
            Phase::Work => 25.0 * 60.0,
            Phase::ShortBreak => 5.0 * 60.0,
            Phase::LongBreak => 15.0 * 60.0,
        };
        let elapsed = total_duration - self.remaining_seconds as f64;
        (elapsed / total_duration * 100.0).max(0.0).min(100.0)
    }
    
    pub fn get_session_display(&self) -> String {
        let current_session = self.session_count % 4 + if self.session_count == 0 { 0 } else { 1 };
        format!("Session {}/4", current_session)
    }
}