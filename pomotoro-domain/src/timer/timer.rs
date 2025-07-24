use serde::{Deserialize, Serialize};
use crate::{TimerStatus, Phase, Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub status: TimerStatus,
    pub phase: Phase,
    pub remaining_seconds: u32,
    pub session_count: u32,
    pub is_break_cycle: bool,
}

impl Default for Timer {
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

impl Timer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_phase_duration(mut self, duration_seconds: u32) -> Self {
        self.remaining_seconds = duration_seconds;
        self
    }

    pub fn next_phase(&mut self, sessions_until_long_break: u8) -> Result<(Phase, Phase)> {
        let old_phase = self.phase.clone();

        match self.phase {
            Phase::Work => {
                self.session_count += 1;

                if self.session_count % sessions_until_long_break as u32 == 0 {
                    self.phase = Phase::LongBreak;
                } else {
                    self.phase = Phase::ShortBreak;
                }
                self.is_break_cycle = true;
            }
            Phase::ShortBreak | Phase::LongBreak => {
                let was_long_break = matches!(self.phase, Phase::LongBreak);
                self.phase = Phase::Work;
                self.is_break_cycle = false;

                if was_long_break && self.session_count >= sessions_until_long_break as u32 {
                    self.session_count = 0;
                }
            }
        }

        Ok((old_phase, self.phase.clone()))
    }

    pub fn reset_current_phase(&mut self, duration_seconds: u32) {
        self.remaining_seconds = duration_seconds;
        self.status = TimerStatus::Stopped;
    }

    pub fn set_status(&mut self, new_status: TimerStatus) -> Result<()> {
        if !self.status.can_transition_to(&new_status) {
            return Err(Error::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: format!("{:?}", new_status),
            });
        }

        self.status = new_status;
        Ok(())
    }

    pub fn format_time(&self) -> String {
        let minutes = self.remaining_seconds / 60;
        let seconds = self.remaining_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn get_phase_name(&self) -> &'static str {
        self.phase.get_name()
    }

    pub fn get_progress_percentage(&self, total_duration: u32) -> f64 {
        let total_duration = total_duration as f64;
        let elapsed = total_duration - self.remaining_seconds as f64;
        (elapsed / total_duration * 100.0).max(0.0).min(100.0)
    }

    pub fn get_session_display(&self) -> String {
        let current_session = self.session_count % 4 + if self.session_count == 0 { 0 } else { 1 };
        format!("Session {}/4", current_session)
    }

    pub fn tick(&mut self) -> bool {
        if self.status == TimerStatus::Running && self.remaining_seconds > 0 {
            self.remaining_seconds -= 1;
            self.remaining_seconds == 0
        } else {
            false
        }
    }
}