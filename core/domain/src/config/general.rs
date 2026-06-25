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
    #[serde(default = "default_persistence_interval_seconds")]
    pub persistence_interval_seconds: u32,
    #[serde(default)]
    pub block_screen_after_work: bool,
    #[serde(default = "default_block_screen_after_work_message")]
    pub block_screen_after_work_message: String,
    #[serde(default)]
    pub block_screen_after_break: bool,
    #[serde(default = "default_block_screen_after_break_message")]
    pub block_screen_after_break_message: String,
}

fn default_persistence_interval_seconds() -> u32 {
    10 // Save every 10 seconds by default
}

fn default_block_screen_after_work_message() -> String {
    "Work session complete. Time for a break.".to_string()
}

fn default_block_screen_after_break_message() -> String {
    "Break over. Back to work.".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            task_cycling_behavior: TaskCyclingBehavior::Manual,
            auto_start_breaks: true,
            auto_start_work_after_break: true,
            minimize_to_tray: true,
            start_minimized: false,
            persistence_interval_seconds: default_persistence_interval_seconds(
            ),
            block_screen_after_work: false,
            block_screen_after_work_message:
                default_block_screen_after_work_message(),
            block_screen_after_break: false,
            block_screen_after_break_message:
                default_block_screen_after_break_message(),
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
