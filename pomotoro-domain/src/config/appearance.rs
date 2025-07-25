use serde::{Deserialize, Serialize};
use crate::Result;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub theme: Theme,
    pub show_seconds_in_display: bool,
    pub always_on_top: bool,
    pub compact_mode: bool,
    pub show_task_list_sidebar: bool,
    pub animate_progress: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: Theme::System,
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