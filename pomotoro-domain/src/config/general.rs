use serde::{Deserialize, Serialize};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskCyclingBehavior {
    Manual,
    AutoAdvance,
    RoundRobin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub task_cycling_behavior: TaskCyclingBehavior,
    pub max_sessions_default: u8,
    pub auto_start_breaks: bool,
    pub auto_start_work_after_break: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            task_cycling_behavior: TaskCyclingBehavior::Manual,
            max_sessions_default: 4,
            auto_start_breaks: true,
            auto_start_work_after_break: false,
            minimize_to_tray: true,
            start_minimized: false,
        }
    }
}

impl GeneralConfig {
    pub fn validate(&self) -> Result<()> {
        if self.max_sessions_default == 0 || self.max_sessions_default > 20 {
            return Err(Error::InvalidSessionCount {
                count: self.max_sessions_default,
            });
        }
        Ok(())
    }
}