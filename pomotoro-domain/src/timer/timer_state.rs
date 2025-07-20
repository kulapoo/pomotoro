use serde::{Deserialize, Serialize};

use crate::{TaskId, TimerStatus, Phase, Task, Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub status: TimerStatus,
    pub phase: Phase,
    pub remaining_seconds: u32,
    pub session_count: u32,
    pub is_break_cycle: bool,
    pub active_task_id: Option<TaskId>,
    pub task_session_count: u32,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            status: TimerStatus::Stopped,
            phase: Phase::Work,
            remaining_seconds: 25 * 60,
            session_count: 0,
            is_break_cycle: false,
            active_task_id: None,
            task_session_count: 0,
        }
    }
}

impl TimerState {
    pub fn get_phase_duration(&self, task: Option<&Task>) -> u32 {
        if let Some(task) = task {
            match self.phase {
                Phase::Work => task.config.work_duration.as_secs() as u32,
                Phase::ShortBreak => task.config.short_break_duration.as_secs() as u32,
                Phase::LongBreak => task.config.long_break_duration.as_secs() as u32,
            }
        } else {
            self.phase.default_duration_seconds()
        }
    }

    pub fn next_phase(&mut self, task: Option<&Task>) -> Result<(Phase, Phase)> {
        let old_phase = self.phase.clone();
        let sessions_until_long_break = task
            .map(|t| t.config.sessions_until_long_break)
            .unwrap_or(4);

        match self.phase {
            Phase::Work => {
                self.session_count += 1;
                self.task_session_count += 1;
                
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
        
        self.remaining_seconds = self.get_phase_duration(task);
        Ok((old_phase, self.phase.clone()))
    }

    pub fn reset_current_phase(&mut self, task: Option<&Task>) {
        self.remaining_seconds = self.get_phase_duration(task);
        self.status = TimerStatus::Stopped;
    }

    pub fn switch_task(&mut self, new_task_id: TaskId, task: Option<&Task>) -> Result<()> {
        if self.status == TimerStatus::Running {
            return Err(Error::InvalidStateTransition {
                from: "Running".to_string(),
                to: "Task Switch".to_string(),
            });
        }

        self.active_task_id = Some(new_task_id);
        self.task_session_count = 0;
        
        if self.status == TimerStatus::Running {
            self.status = TimerStatus::Stopped;
        }
        
        if !self.is_break_cycle {
            self.remaining_seconds = self.get_phase_duration(task);
        }
        
        Ok(())
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
    
    pub fn get_progress_percentage(&self, task: Option<&Task>) -> f64 {
        let total_duration = self.get_phase_duration(task) as f64;
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