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
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Light, // Changed from System to Light
        }
    }
}

impl AppearanceConfig {
    pub fn validate(&self) -> Result<()> {
        // All appearance settings are valid by enum/boolean constraints
        Ok(())
    }
}
