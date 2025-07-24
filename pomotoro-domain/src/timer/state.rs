use serde::{Deserialize, Serialize};

use crate::{TaskId, Timer, Error, Result, Phase, TimerStatus, TimerConfiguration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub timer: Timer,
    pub active_task_id: Option<TaskId>,
    pub configuration: TimerConfiguration,
    pub task_session_count: u32,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            timer: Timer::default(),
            active_task_id: None,
            task_session_count: 0,
            configuration: TimerConfiguration::default(),
        }
    }
}

impl TimerState {
    pub fn get_phase_duration(&self) -> u32 {
        self.configuration.get_phase_duration_seconds(self.timer.phase)
    }

    pub fn next_phase(&mut self) -> Result<(Phase, Phase)> {
        if self.timer.phase == Phase::Work {
            self.task_session_count += 1;
        }

        let (old_phase, new_phase) = self.timer.next_phase(self.configuration.sessions_until_long_break)?;
        let duration = self.get_phase_duration();
        self.timer.remaining_seconds = duration;

        Ok((old_phase, new_phase))
    }

    pub fn reset_current_phase(&mut self) {
        let duration = self.get_phase_duration();
        self.timer.reset_current_phase(duration);
    }

    pub fn switch_task(&mut self, new_task_id: TaskId) -> Result<()> {
        if self.timer.status == TimerStatus::Running {
            return Err(Error::InvalidStateTransition {
                from: "Running".to_string(),
                to: "Task Switch".to_string(),
            });
        }

        self.active_task_id = Some(new_task_id);
        self.task_session_count = 0;

        if !self.timer.is_break_cycle {
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
        self.timer.phase.clone()
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
    pub fn switch_task_with_config(&mut self, new_task_id: TaskId, configuration: TimerConfiguration) -> Result<()> {
        if self.timer.status == TimerStatus::Running {
            return Err(Error::InvalidStateTransition {
                from: "Running".to_string(),
                to: "Task Switch".to_string(),
            });
        }

        self.active_task_id = Some(new_task_id);
        self.task_session_count = 0;
        self.configuration = configuration;

        if !self.timer.is_break_cycle {
            let duration = self.get_phase_duration();
            self.timer.remaining_seconds = duration;
        }

        Ok(())
    }
}