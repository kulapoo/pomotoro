use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn get_name(&self) -> &'static str {
        match self {
            Phase::Work => "Focus Time",
            Phase::ShortBreak => "Short Break",
            Phase::LongBreak => "Long Break",
        }
    }

    pub fn default_duration_seconds(&self) -> u32 {
        match self {
            Phase::Work => 25 * 60,
            Phase::ShortBreak => 5 * 60,
            Phase::LongBreak => 15 * 60,
        }
    }
}