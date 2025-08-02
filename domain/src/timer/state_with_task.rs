use serde::{Deserialize, Serialize};

use crate::{Task, TimerState};

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

    pub fn with_task(mut self, task: Task) -> Self {
        self.active_task = Some(task);
        self
    }

    pub fn without_task(mut self) -> Self {
        self.active_task = None;
        self
    }

    // Delegate timer state methods
    pub fn get_active_task_name(&self) -> String {
        self.active_task
            .as_ref()
            .map(|task| task.name.clone())
            .unwrap_or_else(|| "No Task".to_string())
    }

    pub fn get_progress_percentage(&self) -> f64 {
        self.timer_state.get_progress_percentage()
    }

    pub fn format_time(&self) -> String {
        self.timer_state.format_time()
    }
}

impl Default for TimerStateWithTask {
    fn default() -> Self {
        Self::new(TimerState::default(), None)
    }
}