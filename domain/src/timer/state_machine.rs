use crate::TimerConfiguration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", content = "data")]
pub enum TimerState {
    Idle {
        configuration: TimerConfiguration,
        session_count: u32,
        active_entity: Option<String>,
    },

    Working {
        remaining_seconds: u32,
        configuration: TimerConfiguration,
        session_count: u32,
        active_entity: Option<String>,
        entity_session_count: u32,
    },

    ShortBreak {
        remaining_seconds: u32,
        configuration: TimerConfiguration,
        session_count: u32,
        active_entity: Option<String>,
        entity_session_count: u32,
    },

    LongBreak {
        remaining_seconds: u32,
        configuration: TimerConfiguration,
        session_count: u32,
        active_entity: Option<String>,
        entity_session_count: u32,
    },

    Paused {
        paused_from: Box<TimerState>,
        remaining_seconds: u32,
    },
}

impl Default for TimerState {
    fn default() -> Self {
        Self::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: None,
        }
    }
}

impl TimerState {
    pub fn new(configuration: TimerConfiguration) -> Self {
        Self::Idle {
            configuration,
            session_count: 0,
            active_entity: None,
        }
    }

    pub fn configuration(&self) -> &TimerConfiguration {
        match self {
            Self::Idle { configuration, .. }
            | Self::Working { configuration, .. }
            | Self::ShortBreak { configuration, .. }
            | Self::LongBreak { configuration, .. } => configuration,
            Self::Paused { paused_from, .. } => paused_from.configuration(),
        }
    }

    pub fn session_count(&self) -> u32 {
        match self {
            Self::Idle { session_count, .. }
            | Self::Working { session_count, .. }
            | Self::ShortBreak { session_count, .. }
            | Self::LongBreak { session_count, .. } => *session_count,
            Self::Paused { paused_from, .. } => paused_from.session_count(),
        }
    }

    pub fn active_entity(&self) -> Option<&str> {
        match self {
            Self::Idle { active_entity, .. }
            | Self::Working { active_entity, .. }
            | Self::ShortBreak { active_entity, .. }
            | Self::LongBreak { active_entity, .. } => active_entity.as_deref(),
            Self::Paused { paused_from, .. } => paused_from.active_entity(),
        }
    }

    pub fn active_entity_id(&self) -> Option<String> {
        self.active_entity().map(|s| s.to_string())
    }

    pub fn remaining_seconds(&self) -> u32 {
        match self {
            Self::Idle { configuration, .. } => {
                configuration.get_phase_duration_seconds(super::Phase::Work)
            }
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
        matches!(self, Self::Idle { .. })
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
            Self::Idle { .. } => super::Status::Stopped,
            Self::Working { .. }
            | Self::ShortBreak { .. }
            | Self::LongBreak { .. } => super::Status::Running,
            Self::Paused { .. } => super::Status::Paused,
        }
    }

    pub fn phase(&self) -> super::Phase {
        match self {
            Self::Idle { .. } => super::Phase::Work,
            Self::Working { .. } => super::Phase::Work,
            Self::ShortBreak { .. } => super::Phase::ShortBreak,
            Self::LongBreak { .. } => super::Phase::LongBreak,
            Self::Paused { paused_from, .. } => paused_from.phase(),
        }
    }

    pub fn entity_session_count(&self) -> u32 {
        match self {
            Self::Working {
                entity_session_count,
                ..
            } => *entity_session_count,
            Self::ShortBreak {
                entity_session_count,
                ..
            } => *entity_session_count,
            Self::LongBreak {
                entity_session_count,
                ..
            } => *entity_session_count,
            Self::Paused { paused_from, .. } => {
                paused_from.entity_session_count()
            }
            _ => 0,
        }
    }

    pub fn with_remaining_seconds(&self, seconds: u32) -> Self {
        match self {
            Self::Working { 
                configuration, 
                session_count, 
                active_entity, 
                entity_session_count, 
                .. 
            } => {
                Self::Working {
                    remaining_seconds: seconds,
                    configuration: configuration.clone(),
                    session_count: *session_count,
                    active_entity: active_entity.clone(),
                    entity_session_count: *entity_session_count,
                }
            }
            Self::ShortBreak { 
                configuration, 
                session_count, 
                active_entity, 
                entity_session_count, 
                .. 
            } => {
                Self::ShortBreak {
                    remaining_seconds: seconds,
                    configuration: configuration.clone(),
                    session_count: *session_count,
                    active_entity: active_entity.clone(),
                    entity_session_count: *entity_session_count,
                }
            }
            Self::LongBreak { 
                configuration, 
                session_count, 
                active_entity, 
                entity_session_count, 
                .. 
            } => {
                Self::LongBreak {
                    remaining_seconds: seconds,
                    configuration: configuration.clone(),
                    session_count: *session_count,
                    active_entity: active_entity.clone(),
                    entity_session_count: *entity_session_count,
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
        assert_eq!(state.session_count(), 0);
        assert_eq!(state.remaining_seconds(), 1500);
        assert!(state.active_entity().is_none());
    }

    #[test]
    fn should_identify_work_phase() {
        let state = TimerState::Working {
            remaining_seconds: 100,
            configuration: TimerConfiguration::default(),
            session_count: 1,
            active_entity: None,
            entity_session_count: 0,
        };
        assert!(state.is_work_phase());
        assert!(!state.is_break_phase());
        assert!(state.is_running());
    }

    #[test]
    fn should_identify_break_phase() {
        let state = TimerState::ShortBreak {
            remaining_seconds: 60,
            configuration: TimerConfiguration::default(),
            session_count: 1,
            active_entity: None,
            entity_session_count: 0,
        };
        assert!(state.is_break_phase());
        assert!(!state.is_work_phase());
        assert!(state.is_running());
    }

    #[test]
    fn should_handle_paused_state() {
        let working = TimerState::Working {
            remaining_seconds: 100,
            configuration: TimerConfiguration::default(),
            session_count: 1,
            active_entity: None,
            entity_session_count: 0,
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
            configuration: TimerConfiguration::default(),
            session_count: 5,
            active_entity: Some("task-123".to_string()),
            entity_session_count: 3,
        };

        let updated = state.with_remaining_seconds(50);
        
        match updated {
            TimerState::Working { 
                remaining_seconds, 
                session_count,
                active_entity,
                entity_session_count,
                .. 
            } => {
                assert_eq!(remaining_seconds, 50);
                assert_eq!(session_count, 5);
                assert_eq!(active_entity, Some("task-123".to_string()));
                assert_eq!(entity_session_count, 3);
            }
            _ => panic!("Expected Working state"),
        }
    }

    #[test]
    fn should_update_remaining_seconds_for_break_states() {
        let short_break = TimerState::ShortBreak {
            remaining_seconds: 300,
            configuration: TimerConfiguration::default(),
            session_count: 2,
            active_entity: Some("task-456".to_string()),
            entity_session_count: 1,
        };

        let updated = short_break.with_remaining_seconds(150);
        
        match updated {
            TimerState::ShortBreak { 
                remaining_seconds,
                session_count,
                active_entity,
                entity_session_count,
                .. 
            } => {
                assert_eq!(remaining_seconds, 150);
                assert_eq!(session_count, 2);
                assert_eq!(active_entity, Some("task-456".to_string()));
                assert_eq!(entity_session_count, 1);
            }
            _ => panic!("Expected ShortBreak state"),
        }
    }

    #[test]
    fn should_update_remaining_seconds_for_paused_state() {
        let working = TimerState::Working {
            remaining_seconds: 100,
            configuration: TimerConfiguration::default(),
            session_count: 1,
            active_entity: None,
            entity_session_count: 0,
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
        let idle = TimerState::Idle {
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_entity: None,
        };

        let updated = idle.with_remaining_seconds(999);
        
        match updated {
            TimerState::Idle { .. } => {
                // Idle state should remain unchanged
                assert_eq!(idle, updated);
            }
            _ => panic!("Expected Idle state"),
        }
    }
}
