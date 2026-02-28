use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceConfig {
    pub theme: Theme,
    pub show_seconds_in_display: bool,
    pub always_on_top: bool,
    #[serde(default)]
    pub compact_mode: bool,
    #[serde(default = "default_true")]
    pub show_task_list_sidebar: bool,
    #[serde(default = "default_true")]
    pub animate_progress: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Light, // Changed from System to Light
            show_seconds_in_display: true,
            always_on_top: false,
            compact_mode: false,
            show_task_list_sidebar: true,
            animate_progress: true,
        }
    }
}

impl AppearanceConfig {
    pub fn validate(&self) -> Result<()> {
        // All appearance settings are valid by enum/boolean constraints
        Ok(())
    }
}
