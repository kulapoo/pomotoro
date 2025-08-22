use serde::{Deserialize, Serialize};
use crate::TimerConfiguration;

/// Unified timer state machine that combines status and phase into a single enum.
/// Each variant encapsulates the data specific to that state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", content = "data")]
pub enum TimerState {
    /// Timer is idle, not tracking any time
    Idle {
        /// Configuration for timing durations
        configuration: TimerConfiguration,
        /// Number of completed work sessions
        session_count: u32,
        /// Currently active entity (if any)
        active_entity: Option<String>,
    },
    
    /// Timer is actively counting down during a work session
    Working {
        /// Seconds remaining in the current work session
        remaining_seconds: u32,
        /// Configuration for timing durations
        configuration: TimerConfiguration,
        /// Number of completed work sessions
        session_count: u32,
        /// Currently active entity (must exist during work)
        active_entity: Option<String>,
        /// Number of sessions completed for the current entity
        entity_session_count: u32,
    },
    
    /// Timer is actively counting down during a short break
    ShortBreak {
        /// Seconds remaining in the break
        remaining_seconds: u32,
        /// Configuration for timing durations
        configuration: TimerConfiguration,
        /// Number of completed work sessions
        session_count: u32,
        /// Entity to resume after break
        active_entity: Option<String>,
        /// Number of sessions completed for the current entity
        entity_session_count: u32,
    },
    
    /// Timer is actively counting down during a long break
    LongBreak {
        /// Seconds remaining in the break
        remaining_seconds: u32,
        /// Configuration for timing durations
        configuration: TimerConfiguration,
        /// Number of completed work sessions
        session_count: u32,
        /// Entity to resume after break
        active_entity: Option<String>,
        /// Number of sessions completed for the current entity
        entity_session_count: u32,
    },
    
    /// Timer is paused, preserving the previous state
    Paused {
        /// The state that was paused
        paused_from: Box<TimerState>,
        /// Remaining seconds when paused
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
    /// Create a new idle timer with the given configuration
    pub fn new(configuration: TimerConfiguration) -> Self {
        Self::Idle {
            configuration,
            session_count: 0,
            active_entity: None,
        }
    }
    
    /// Get the current configuration from any state
    pub fn configuration(&self) -> &TimerConfiguration {
        match self {
            Self::Idle { configuration, .. } |
            Self::Working { configuration, .. } |
            Self::ShortBreak { configuration, .. } |
            Self::LongBreak { configuration, .. } => configuration,
            Self::Paused { paused_from, .. } => paused_from.configuration(),
        }
    }
    
    /// Get the current session count
    pub fn session_count(&self) -> u32 {
        match self {
            Self::Idle { session_count, .. } |
            Self::Working { session_count, .. } |
            Self::ShortBreak { session_count, .. } |
            Self::LongBreak { session_count, .. } => *session_count,
            Self::Paused { paused_from, .. } => paused_from.session_count(),
        }
    }
    
    /// Get the currently active entity
    pub fn active_entity(&self) -> Option<&str> {
        match self {
            Self::Idle { active_entity, .. } |
            Self::Working { active_entity, .. } |
            Self::ShortBreak { active_entity, .. } |
            Self::LongBreak { active_entity, .. } => active_entity.as_deref(),
            Self::Paused { paused_from, .. } => paused_from.active_entity(),
        }
    }
    
    /// Get the currently active entity ID (for compatibility)
    pub fn active_entity_id(&self) -> Option<String> {
        self.active_entity().map(|s| s.to_string())
    }
    
    /// Get remaining seconds (shows work duration if idle)
    pub fn remaining_seconds(&self) -> u32 {
        match self {
            Self::Idle { configuration, .. } => {
                // When idle, show the work phase duration as the "ready" time
                configuration.get_phase_duration_seconds(super::Phase::Work)
            },
            Self::Working { remaining_seconds, .. } |
            Self::ShortBreak { remaining_seconds, .. } |
            Self::LongBreak { remaining_seconds, .. } |
            Self::Paused { remaining_seconds, .. } => *remaining_seconds,
        }
    }
    
    /// Check if the timer is currently running (actively counting down)
    pub fn is_running(&self) -> bool {
        matches!(
            self,
            Self::Working { .. } | Self::ShortBreak { .. } | Self::LongBreak { .. }
        )
    }
    
    /// Check if the timer is paused
    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused { .. })
    }
    
    /// Check if the timer is idle
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle { .. })
    }
    
    /// Check if the timer is in a work phase (working or paused from working)
    pub fn is_work_phase(&self) -> bool {
        match self {
            Self::Working { .. } => true,
            Self::Paused { paused_from, .. } => paused_from.is_work_phase(),
            _ => false,
        }
    }
    
    /// Check if the timer is in a break phase
    pub fn is_break_phase(&self) -> bool {
        match self {
            Self::ShortBreak { .. } | Self::LongBreak { .. } => true,
            Self::Paused { paused_from, .. } => paused_from.is_break_phase(),
            _ => false,
        }
    }
    
    
    
    
    
    
    
    /// Get the timer status
    pub fn status(&self) -> super::Status {
        match self {
            Self::Idle { .. } => super::Status::Stopped,
            Self::Working { .. } | Self::ShortBreak { .. } | Self::LongBreak { .. } => super::Status::Running,
            Self::Paused { .. } => super::Status::Paused,
        }
    }
    
    
    /// Get the current phase as enum
    pub fn phase(&self) -> super::Phase {
        match self {
            Self::Idle { .. } => super::Phase::Work, // Default to Work for idle
            Self::Working { .. } => super::Phase::Work,
            Self::ShortBreak { .. } => super::Phase::ShortBreak,
            Self::LongBreak { .. } => super::Phase::LongBreak,
            Self::Paused { paused_from, .. } => paused_from.phase(),
        }
    }
    
    /// Get the entity session count (only available in Working state)
    pub fn entity_session_count(&self) -> u32 {
        match self {
            Self::Working { entity_session_count, .. } => *entity_session_count,
            Self::ShortBreak { entity_session_count, .. } => *entity_session_count,
            Self::LongBreak { entity_session_count, .. } => *entity_session_count,
            Self::Paused { paused_from, .. } => paused_from.entity_session_count(),
            _ => 0,
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
        // When idle, remaining_seconds shows the work phase duration as "ready" time
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
        // Removed test of deprecated presentation logic method
    }
    
    
}