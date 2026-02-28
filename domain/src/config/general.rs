use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskCyclingBehavior {
    Manual,
    AutoAdvance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralConfig {
    pub task_cycling_behavior: TaskCyclingBehavior,
    pub auto_start_breaks: bool,
    pub auto_start_work_after_break: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    #[serde(default)]
    pub enable_screen_blocking: bool,
    #[serde(default = "default_persistence_interval_seconds")]
    pub persistence_interval_seconds: u32,
}

fn default_persistence_interval_seconds() -> u32 {
    10 // Save every 10 seconds by default
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            task_cycling_behavior: TaskCyclingBehavior::Manual,
            auto_start_breaks: true,
            auto_start_work_after_break: true,
            minimize_to_tray: true,
            start_minimized: false,
            enable_screen_blocking: false,
            persistence_interval_seconds: default_persistence_interval_seconds(
            ),
        }
    }
}

impl GeneralConfig {
    pub fn validate(&self) -> Result<()> {
        if self.persistence_interval_seconds == 0 {
            return Err(Error::InvalidDuration {
                duration: self.persistence_interval_seconds,
            });
        }

        if self.persistence_interval_seconds > 3600 {
            return Err(Error::InvalidDuration {
                duration: self.persistence_interval_seconds,
            });
        }

        Ok(())
    }
}
