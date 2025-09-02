use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", content = "data")]
pub enum TimerState {
    Idle,

    Working {
        remaining_seconds: u32,
    },

    ShortBreak {
        remaining_seconds: u32,
    },

    LongBreak {
        remaining_seconds: u32,
    },

    Paused {
        paused_from: Box<TimerState>,
        remaining_seconds: u32,
    },
}

impl Default for TimerState {
    fn default() -> Self {
        Self::Idle
    }
}

impl TimerState {
    pub fn new() -> Self {
        Self::Idle
    }


    pub fn remaining_seconds(&self) -> u32 {
        match self {
            Self::Idle => 0,
            Self::Working {
                remaining_seconds, ..
            }
            | Self::ShortBreak {
                remaining_seconds, ..
            }
            | Self::LongBreak {
                remaining_seconds, ..
            }
            | Self::Paused {
                remaining_seconds, ..
            } => *remaining_seconds,
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(
            self,
            Self::Working { .. }
                | Self::ShortBreak { .. }
                | Self::LongBreak { .. }
        )
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused { .. })
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    pub fn is_work_phase(&self) -> bool {
        match self {
            Self::Working { .. } => true,
            Self::Paused { paused_from, .. } => paused_from.is_work_phase(),
            _ => false,
        }
    }

    pub fn is_break_phase(&self) -> bool {
        match self {
            Self::ShortBreak { .. } | Self::LongBreak { .. } => true,
            Self::Paused { paused_from, .. } => paused_from.is_break_phase(),
            _ => false,
        }
    }

    pub fn status(&self) -> super::Status {
        match self {
            Self::Idle => super::Status::Stopped,
            Self::Working { .. }
            | Self::ShortBreak { .. }
            | Self::LongBreak { .. } => super::Status::Running,
            Self::Paused { .. } => super::Status::Paused,
        }
    }

    pub fn phase(&self) -> super::Phase {
        match self {
            Self::Idle => super::Phase::Work,
            Self::Working { .. } => super::Phase::Work,
            Self::ShortBreak { .. } => super::Phase::ShortBreak,
            Self::LongBreak { .. } => super::Phase::LongBreak,
            Self::Paused { paused_from, .. } => paused_from.phase(),
        }
    }


    pub fn with_remaining_seconds(&self, seconds: u32) -> Self {
        match self {
            Self::Working { .. } => {
                Self::Working {
                    remaining_seconds: seconds,
                }
            }
            Self::ShortBreak { .. } => {
                Self::ShortBreak {
                    remaining_seconds: seconds,
                }
            }
            Self::LongBreak { .. } => {
                Self::LongBreak {
                    remaining_seconds: seconds,
                }
            }
            Self::Paused { paused_from, .. } => {
                Self::Paused {
                    paused_from: paused_from.clone(),
                    remaining_seconds: seconds,
                }
            }
            _ => self.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_default_idle_state() {
        let state = TimerState::default();
        assert!(state.is_idle());
        assert_eq!(state.remaining_seconds(), 0);
    }

    #[test]
    fn should_identify_work_phase() {
        let state = TimerState::Working {
            remaining_seconds: 100,
        };
        assert!(state.is_work_phase());
        assert!(!state.is_break_phase());
        assert!(state.is_running());
    }

    #[test]
    fn should_identify_break_phase() {
        let state = TimerState::ShortBreak {
            remaining_seconds: 60,
        };
        assert!(state.is_break_phase());
        assert!(!state.is_work_phase());
        assert!(state.is_running());
    }

    #[test]
    fn should_handle_paused_state() {
        let working = TimerState::Working {
            remaining_seconds: 100,
        };

        let paused = TimerState::Paused {
            paused_from: Box::new(working),
            remaining_seconds: 100,
        };

        assert!(paused.is_paused());
        assert!(paused.is_work_phase());
    }

    #[test]
    fn should_update_remaining_seconds_for_working_state() {
        let state = TimerState::Working {
            remaining_seconds: 100,
        };

        let updated = state.with_remaining_seconds(50);
        
        match updated {
            TimerState::Working { 
                remaining_seconds,
                .. 
            } => {
                assert_eq!(remaining_seconds, 50);
            }
            _ => panic!("Expected Working state"),
        }
    }

    #[test]
    fn should_update_remaining_seconds_for_break_states() {
        let short_break = TimerState::ShortBreak {
            remaining_seconds: 300,
        };

        let updated = short_break.with_remaining_seconds(150);
        
        match updated {
            TimerState::ShortBreak { 
                remaining_seconds,
                .. 
            } => {
                assert_eq!(remaining_seconds, 150);
            }
            _ => panic!("Expected ShortBreak state"),
        }
    }

    #[test]
    fn should_update_remaining_seconds_for_paused_state() {
        let working = TimerState::Working {
            remaining_seconds: 100,
        };

        let paused = TimerState::Paused {
            paused_from: Box::new(working),
            remaining_seconds: 100,
        };

        let updated = paused.with_remaining_seconds(75);
        
        match updated {
            TimerState::Paused { 
                remaining_seconds,
                .. 
            } => {
                assert_eq!(remaining_seconds, 75);
            }
            _ => panic!("Expected Paused state"),
        }
    }

    #[test]
    fn should_return_idle_state_unchanged() {
        let idle = TimerState::Idle;

        let updated = idle.with_remaining_seconds(999);
        
        match updated {
            TimerState::Idle => {
                assert_eq!(idle, updated);
            }
            _ => panic!("Expected Idle state"),
        }
    }
}
