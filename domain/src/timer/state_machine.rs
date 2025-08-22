use serde::{Deserialize, Serialize};
use crate::{Result, Error, TimerConfiguration, TaskId};

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
        /// Currently active task (if any)
        active_task: Option<TaskId>,
    },
    
    /// Timer is actively counting down during a work session
    Working {
        /// Seconds remaining in the current work session
        remaining_seconds: u32,
        /// Configuration for timing durations
        configuration: TimerConfiguration,
        /// Number of completed work sessions
        session_count: u32,
        /// Currently active task (must exist during work)
        active_task: Option<TaskId>,
        /// Number of sessions completed for the current task
        task_session_count: u32,
    },
    
    /// Timer is actively counting down during a short break
    ShortBreak {
        /// Seconds remaining in the break
        remaining_seconds: u32,
        /// Configuration for timing durations
        configuration: TimerConfiguration,
        /// Number of completed work sessions
        session_count: u32,
        /// Task to resume after break
        active_task: Option<TaskId>,
        /// Number of sessions completed for the current task
        task_session_count: u32,
    },
    
    /// Timer is actively counting down during a long break
    LongBreak {
        /// Seconds remaining in the break
        remaining_seconds: u32,
        /// Configuration for timing durations
        configuration: TimerConfiguration,
        /// Number of completed work sessions
        session_count: u32,
        /// Task to resume after break
        active_task: Option<TaskId>,
        /// Number of sessions completed for the current task
        task_session_count: u32,
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
            active_task: None,
        }
    }
}

impl TimerState {
    /// Create a new idle timer with the given configuration
    pub fn new(configuration: TimerConfiguration) -> Self {
        Self::Idle {
            configuration,
            session_count: 0,
            active_task: None,
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
    
    /// Get the currently active task
    pub fn active_task(&self) -> Option<TaskId> {
        match self {
            Self::Idle { active_task, .. } |
            Self::Working { active_task, .. } |
            Self::ShortBreak { active_task, .. } |
            Self::LongBreak { active_task, .. } => *active_task,
            Self::Paused { paused_from, .. } => paused_from.active_task(),
        }
    }
    
    /// Get the currently active task ID (alias for active_task for compatibility)
    pub fn active_task_id(&self) -> Option<TaskId> {
        self.active_task()
    }
    
    /// Get remaining seconds (shows work duration if idle)
    pub fn remaining_seconds(&self) -> u32 {
        match self {
            Self::Idle { configuration, .. } => {
                // When idle, show the work phase duration as the "ready" time
                configuration.get_phase_duration_seconds(crate::Phase::Work)
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
    
    /// Get a human-readable name for the current phase
    /// @deprecated - Use TimerViewModel in application layer for presentation logic
    #[deprecated(note = "Use TimerViewModel in application layer for presentation logic")]
    pub fn phase_name(&self) -> &'static str {
        match self {
            Self::Idle { .. } => "Stopped",
            Self::Working { .. } => "Focus Time",
            Self::ShortBreak { .. } => "Short Break",
            Self::LongBreak { .. } => "Long Break",
            Self::Paused { paused_from, .. } => {
                match paused_from.as_ref() {
                    Self::Working { .. } => "Focus Time (Paused)",
                    Self::ShortBreak { .. } => "Short Break (Paused)",
                    Self::LongBreak { .. } => "Long Break (Paused)",
                    _ => "Paused",
                }
            }
        }
    }
    
    /// Format remaining time as MM:SS
    /// @deprecated - Use TimerViewModel in application layer for presentation logic
    #[deprecated(note = "Use TimerViewModel in application layer for presentation logic")]
    pub fn format_time(&self) -> String {
        let seconds = self.remaining_seconds();
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}", minutes, secs)
    }
    
    /// Get progress percentage for the current phase
    /// @deprecated - Use TimerViewModel in application layer for presentation logic
    #[deprecated(note = "Use TimerViewModel in application layer for presentation logic")]
    pub fn progress_percentage(&self) -> f64 {
        match self {
            Self::Idle { .. } => 0.0,
            Self::Working { remaining_seconds, configuration, .. } => {
                let total = configuration.get_phase_duration_seconds(crate::Phase::Work) as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            Self::ShortBreak { remaining_seconds, configuration, .. } => {
                let total = configuration.get_phase_duration_seconds(crate::Phase::ShortBreak) as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            Self::LongBreak { remaining_seconds, configuration, .. } => {
                let total = configuration.get_phase_duration_seconds(crate::Phase::LongBreak) as f64;
                let elapsed = total - *remaining_seconds as f64;
                (elapsed / total * 100.0).clamp(0.0, 100.0)
            }
            Self::Paused { paused_from, remaining_seconds } => {
                // Calculate based on what was paused
                match paused_from.as_ref() {
                    Self::Working { configuration, .. } => {
                        let total = configuration.get_phase_duration_seconds(crate::Phase::Work) as f64;
                        let elapsed = total - *remaining_seconds as f64;
                        (elapsed / total * 100.0).clamp(0.0, 100.0)
                    }
                    Self::ShortBreak { configuration, .. } => {
                        let total = configuration.get_phase_duration_seconds(crate::Phase::ShortBreak) as f64;
                        let elapsed = total - *remaining_seconds as f64;
                        (elapsed / total * 100.0).clamp(0.0, 100.0)
                    }
                    Self::LongBreak { configuration, .. } => {
                        let total = configuration.get_phase_duration_seconds(crate::Phase::LongBreak) as f64;
                        let elapsed = total - *remaining_seconds as f64;
                        (elapsed / total * 100.0).clamp(0.0, 100.0)
                    }
                    _ => 0.0,
                }
            }
        }
    }
    
    /// Get session display string (e.g., "Session 2/4")
    /// @deprecated - Use TimerViewModel in application layer for presentation logic
    #[deprecated(note = "Use TimerViewModel in application layer for presentation logic")]
    pub fn session_display(&self) -> String {
        let count = self.session_count();
        let config = self.configuration();
        let sessions_until_long = config.sessions_until_long_break as u32;
        
        let current_in_cycle = if count == 0 {
            0
        } else {
            ((count - 1) % sessions_until_long) + 1
        };
        
        format!("Session {}/{}", current_in_cycle, sessions_until_long)
    }
    
    /// Update configuration in any state
    /// @deprecated - State mutations should only happen through Timer aggregate
    #[deprecated(note = "State mutations should only happen through Timer aggregate")]
    pub fn update_configuration(&mut self, new_config: TimerConfiguration) -> Result<()> {
        match self {
            Self::Idle { configuration, .. } => {
                *configuration = new_config;
            }
            Self::Working { configuration: _, .. } |
            Self::ShortBreak { configuration: _, .. } |
            Self::LongBreak { configuration: _, .. } => {
                // Only update if not actively running to avoid confusion
                return Err(Error::InvalidStateTransition {
                    from: "Running".to_string(),
                    to: "ConfigUpdate".to_string(),
                });
            }
            Self::Paused { .. } => {
                return Err(Error::InvalidStateTransition {
                    from: "Paused".to_string(),
                    to: "ConfigUpdate".to_string(),
                });
            }
        }
        Ok(())
    }
    
    /// Update the active task
    /// @deprecated - State mutations should only happen through Timer aggregate
    #[deprecated(note = "State mutations should only happen through Timer aggregate")]
    pub fn set_active_task(&mut self, task_id: Option<TaskId>) -> Result<()> {
        match self {
            Self::Idle { active_task, .. } => {
                *active_task = task_id;
                Ok(())
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{:?}", self),
                to: "TaskSwitch".to_string(),
            }),
        }
    }
    
    /// Get the timer status
    pub fn status(&self) -> crate::TimerStatus {
        match self {
            Self::Idle { .. } => crate::TimerStatus::Stopped,
            Self::Working { .. } | Self::ShortBreak { .. } | Self::LongBreak { .. } => crate::TimerStatus::Running,
            Self::Paused { .. } => crate::TimerStatus::Paused,
        }
    }
    
    /// Switch task with configuration
    /// @deprecated - State mutations should only happen through Timer aggregate
    #[deprecated(note = "State mutations should only happen through Timer aggregate")]
    pub fn switch_task_with_config(&mut self, task_id: TaskId, config: crate::TimerConfiguration) -> Result<()> {
        match self {
            Self::Idle { configuration, active_task, .. } => {
                *configuration = config;
                *active_task = Some(task_id);
                Ok(())
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{:?}", self),
                to: "TaskSwitch".to_string(),
            }),
        }
    }
    
    /// Get the current phase as enum
    pub fn phase(&self) -> crate::Phase {
        match self {
            Self::Idle { .. } => crate::Phase::Work, // Default to Work for idle
            Self::Working { .. } => crate::Phase::Work,
            Self::ShortBreak { .. } => crate::Phase::ShortBreak,
            Self::LongBreak { .. } => crate::Phase::LongBreak,
            Self::Paused { paused_from, .. } => paused_from.phase(),
        }
    }
    
    /// Get the task session count (only available in Working state)
    pub fn task_session_count(&self) -> u32 {
        match self {
            Self::Working { task_session_count, .. } => *task_session_count,
            Self::ShortBreak { task_session_count, .. } => *task_session_count,
            Self::LongBreak { task_session_count, .. } => *task_session_count,
            Self::Paused { paused_from, .. } => paused_from.task_session_count(),
            _ => 0,
        }
    }
    
    /// Set the timer status
    /// @deprecated - State mutations should only happen through Timer aggregate
    #[deprecated(note = "State mutations should only happen through Timer aggregate")]
    pub fn set_status(&mut self, status: crate::TimerStatus) -> Result<()> {
        match (self.status(), status) {
            (current, new) if current == new => Ok(()), // No change needed
            (crate::TimerStatus::Stopped, crate::TimerStatus::Running) => {
                // For backwards compatibility with tests, allow this transition
                // In real code, should use Timer::start() instead
                if let Self::Idle { configuration, session_count, active_task } = self.clone() {
                    if active_task.is_some() {
                        let remaining_seconds = configuration.get_phase_duration_seconds(crate::Phase::Work);
                        *self = Self::Working {
                            remaining_seconds,
                            configuration,
                            session_count,
                            active_task,
                            task_session_count: 0,
                        };
                        return Ok(());
                    }
                }
                Err(Error::InvalidStateTransition {
                    from: "Stopped".to_string(),
                    to: "Running".to_string(),
                })
            }
            (crate::TimerStatus::Running, crate::TimerStatus::Stopped) => {
                // For backwards compatibility with tests, allow stopping from running
                // In real code, should use Timer::reset() instead
                let configuration = self.configuration().clone();
                let active_task = self.active_task();
                *self = Self::Idle {
                    configuration,
                    session_count: 0,
                    active_task,
                };
                Ok(())
            }
            (crate::TimerStatus::Paused, crate::TimerStatus::Stopped) => {
                // Allow stopping from paused
                let configuration = self.configuration().clone();
                let active_task = self.active_task();
                *self = Self::Idle {
                    configuration,
                    session_count: 0,
                    active_task,
                };
                Ok(())
            }
            _ => Err(Error::InvalidStateTransition {
                from: format!("{:?}", self.status()),
                to: format!("{:?}", status),
            })
        }
    }
    
    /// Get session display (alias for session_display)
    /// @deprecated - Use TimerViewModel in application layer for presentation logic
    #[deprecated(note = "Use TimerViewModel in application layer for presentation logic")]
    pub fn get_session_display(&self) -> String {
        #[allow(deprecated)]
        self.session_display()
    }
    
    /// Get phase name (alias for phase_name)
    /// @deprecated - Use TimerViewModel in application layer for presentation logic
    #[deprecated(note = "Use TimerViewModel in application layer for presentation logic")]
    pub fn get_phase_name(&self) -> &'static str {
        #[allow(deprecated)]
        self.phase_name()
    }
    
    /// Get progress percentage (alias for progress_percentage)
    /// @deprecated - Use TimerViewModel in application layer for presentation logic
    #[deprecated(note = "Use TimerViewModel in application layer for presentation logic")]
    pub fn get_progress_percentage(&self) -> f64 {
        #[allow(deprecated)]
        self.progress_percentage()
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
        assert!(state.active_task().is_none());
    }
    
    #[test]
    fn should_identify_work_phase() {
        let state = TimerState::Working {
            remaining_seconds: 100,
            configuration: TimerConfiguration::default(),
            session_count: 1,
            active_task: None,
            task_session_count: 0,
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
            active_task: None,
            task_session_count: 0,
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
            active_task: None,
            task_session_count: 0,
        };
        
        let paused = TimerState::Paused {
            paused_from: Box::new(working),
            remaining_seconds: 100,
        };
        
        assert!(paused.is_paused());
        assert!(paused.is_work_phase());
        assert_eq!(paused.phase_name(), "Focus Time (Paused)");
    }
    
    #[test]
    fn should_format_time_correctly() {
        let state = TimerState::Working {
            remaining_seconds: 1525, // 25:25
            configuration: TimerConfiguration::default(),
            session_count: 0,
            active_task: None,
            task_session_count: 0,
        };
        assert_eq!(state.format_time(), "25:25");
        
        let state2 = TimerState::ShortBreak {
            remaining_seconds: 65, // 01:05
            configuration: TimerConfiguration::default(),
            session_count: 1,
            active_task: None,
            task_session_count: 0,
        };
        assert_eq!(state2.format_time(), "01:05");
    }
    
    #[test]
    fn should_calculate_progress_percentage() {
        let config = TimerConfiguration::default();
        let total_work = config.get_phase_duration_seconds(crate::Phase::Work);
        
        let state = TimerState::Working {
            remaining_seconds: total_work / 2, // 50% complete
            configuration: config,
            session_count: 0,
            active_task: None,
            task_session_count: 0,
        };
        
        let progress = state.progress_percentage();
        assert!((progress - 50.0).abs() < 0.1);
    }
}