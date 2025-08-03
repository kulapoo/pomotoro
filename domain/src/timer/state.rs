use serde::{Deserialize, Serialize};

use crate::{Phase, Result, TaskId, Timer, TimerConfiguration, TimerStatus};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimerState {
    pub timer: Timer,
    pub active_task_id: Option<TaskId>,
    pub configuration: TimerConfiguration,
    pub task_session_count: u32,
}

impl TimerState {
    pub fn get_phase_duration(&self) -> u32 {
        self.configuration
            .get_phase_duration_seconds(self.timer.phase)
    }

    pub fn next_phase(&mut self) -> Result<(Phase, Phase)> {
        if self.timer.phase == Phase::Work {
            self.task_session_count += 1;
        }

        let (old_phase, new_phase) = self
            .timer
            .next_phase(self.configuration.sessions_until_long_break)?;
        let duration = self.get_phase_duration();
        self.timer.remaining_seconds = duration;

        Ok((old_phase, new_phase))
    }

    pub fn reset_current_phase(&mut self) {
        let duration = self.get_phase_duration();
        self.timer.reset_current_phase(duration);
    }

    pub fn switch_task(&mut self, new_task_id: TaskId) -> Result<()> {
        // Allow task switching during running sessions (preserves timer state)
        self.active_task_id = Some(new_task_id);
        self.task_session_count = 0;

        // Only reset timer if not running or in break cycle
        if self.timer.status != TimerStatus::Running && !self.timer.is_break_cycle {
            let duration = self.get_phase_duration();
            self.timer.remaining_seconds = duration;
        }

        Ok(())
    }

    pub fn set_status(&mut self, new_status: TimerStatus) -> Result<()> {
        self.timer.set_status(new_status)
    }

    pub fn format_time(&self) -> String {
        self.timer.format_time()
    }

    pub fn get_phase_name(&self) -> &'static str {
        self.timer.get_phase_name()
    }

    pub fn get_progress_percentage(&self) -> f64 {
        let total_duration = self.get_phase_duration();
        self.timer.get_progress_percentage(total_duration)
    }

    pub fn get_session_display(&self) -> String {
        self.timer.get_session_display()
    }

    pub fn tick(&mut self) -> bool {
        self.timer.tick()
    }

    // Convenience methods to access timer_core properties
    pub fn status(&self) -> TimerStatus {
        self.timer.status.clone()
    }

    pub fn phase(&self) -> Phase {
        self.timer.phase
    }

    pub fn remaining_seconds(&self) -> u32 {
        self.timer.remaining_seconds
    }

    pub fn session_count(&self) -> u32 {
        self.timer.session_count
    }

    pub fn is_break_cycle(&self) -> bool {
        self.timer.is_break_cycle
    }

    /// Set the timer configuration, adjusting current phase duration if needed.
    pub fn set_configuration(&mut self, configuration: TimerConfiguration) {
        let old_duration = self.get_phase_duration();
        self.configuration = configuration;
        let new_duration = self.get_phase_duration();

        // Only update remaining time if timer is stopped and duration changed
        if self.timer.status == TimerStatus::Stopped && old_duration != new_duration {
            self.timer.remaining_seconds = new_duration;
        }
    }

    /// Switch task with new configuration.
    pub fn switch_task_with_config(
        &mut self,
        new_task_id: TaskId,
        configuration: TimerConfiguration,
    ) -> Result<()> {
        // Allow task switching during running sessions (preserves timer state)
        self.active_task_id = Some(new_task_id);
        self.task_session_count = 0;
        self.configuration = configuration;
        // Only reset timer if not running or in break cycle
        if self.timer.status != TimerStatus::Running && !self.timer.is_break_cycle {
            let duration = self.get_phase_duration();
            self.timer.remaining_seconds = duration;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_and_deserialize_timer_state() {
        let state = TimerState::default();
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&state).unwrap();
        println!("Serialized TimerState:\n{}", json);
        
        // Deserialize back
        let deserialized: TimerState = serde_json::from_str(&json).unwrap();
        
        // Verify structure matches
        assert_eq!(state.timer.status, deserialized.timer.status);
        assert_eq!(state.timer.phase, deserialized.timer.phase);
        assert_eq!(state.timer.remaining_seconds, deserialized.timer.remaining_seconds);
        assert_eq!(state.timer.session_count, deserialized.timer.session_count);
        assert_eq!(state.timer.is_break_cycle, deserialized.timer.is_break_cycle);
        assert_eq!(state.active_task_id, deserialized.active_task_id);
        assert_eq!(state.task_session_count, deserialized.task_session_count);
        assert_eq!(state.configuration.work_duration, deserialized.configuration.work_duration);
    }
}
