use serde::{Deserialize, Serialize};
use crate::task::types::{TaskId, Task};

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
            match self.phase {
                Phase::Work => 25 * 60,
                Phase::ShortBreak => 5 * 60,
                Phase::LongBreak => 15 * 60,
            }
        }
    }

    pub fn next_phase(&mut self, task: Option<&Task>) {
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
    }

    pub fn reset_current_phase(&mut self, task: Option<&Task>) {
        self.remaining_seconds = self.get_phase_duration(task);
        self.status = TimerStatus::Stopped;
    }


    pub fn switch_task(&mut self, new_task_id: TaskId, task: Option<&Task>) {
        self.active_task_id = Some(new_task_id);
        self.task_session_count = 0;
        
        if self.status == TimerStatus::Running {
            self.status = TimerStatus::Stopped;
        }
        
        if !self.is_break_cycle {
            self.remaining_seconds = self.get_phase_duration(task);
        }
    }

}