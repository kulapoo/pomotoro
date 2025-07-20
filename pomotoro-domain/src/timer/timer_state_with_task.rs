use serde::{Deserialize, Serialize};

use crate::{TimerState, Task};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStateWithTask {
    pub timer_state: TimerState,
    pub active_task: Option<Task>,
}

impl TimerStateWithTask {
    pub fn new(timer_state: TimerState, active_task: Option<Task>) -> Self {
        Self {
            timer_state,
            active_task,
        }
    }

    pub fn format_time(&self) -> String {
        self.timer_state.format_time()
    }

    pub fn get_progress_percentage(&self) -> f64 {
        self.timer_state.get_progress_percentage(self.active_task.as_ref())
    }

    pub fn get_phase_name(&self) -> &'static str {
        self.timer_state.get_phase_name()
    }

    pub fn get_active_task_name(&self) -> String {
        self.active_task
            .as_ref()
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "Focus Session".to_string())
    }
}