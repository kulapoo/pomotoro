use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskCyclingBehavior {
    Manual,
    AutoAdvance,
    RoundRobin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralConfig {
    pub task_cycling_behavior: TaskCyclingBehavior,
    pub auto_start_breaks: bool,
    pub auto_start_work_after_break: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub enable_screen_blocking: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            task_cycling_behavior: TaskCyclingBehavior::Manual,
            auto_start_breaks: true,
            auto_start_work_after_break: false,
            minimize_to_tray: true,
            start_minimized: false,
            enable_screen_blocking: false,
        }
    }
}

impl GeneralConfig {
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}
