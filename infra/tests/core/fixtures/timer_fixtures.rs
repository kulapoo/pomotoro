use domain::{TimerConfiguration, timer::TimerState};
use std::time::Duration;

/// Timer-related fixtures for testing
pub struct TimerFixtures;

impl TimerFixtures {
    /// Create a default timer configuration
    pub fn default_config() -> TimerConfiguration {
        TimerConfiguration::new(
            Duration::from_secs(25 * 60), // 25 minutes work
            Duration::from_secs(5 * 60),  // 5 minutes short break
            Duration::from_secs(15 * 60), // 15 minutes long break
            4,                            // Sessions until long break
        )
        .expect("Failed to create timer configuration")
    }

    /// Create a fast timer config for testing (minimum allowed values)
    pub fn fast_config() -> TimerConfiguration {
        TimerConfiguration::new(
            Duration::from_secs(60), // 1 minute work (minimum allowed)
            Duration::from_secs(60), // 1 minute short break (minimum allowed)
            Duration::from_secs(60), // 1 minute long break (minimum allowed)
            2,                       // 2 sessions until long break
        )
        .expect("Failed to create fast timer configuration")
    }

    /// Create a custom timer configuration
    pub fn custom_config(
        work_secs: u64,
        short_break_secs: u64,
        long_break_secs: u64,
        sessions: u8,
    ) -> TimerConfiguration {
        TimerConfiguration::new(
            Duration::from_secs(work_secs),
            Duration::from_secs(short_break_secs),
            Duration::from_secs(long_break_secs),
            sessions,
        )
        .expect("Failed to create custom timer configuration")
    }

    /// Create an initial timer state
    pub fn initial_state() -> TimerState {
        TimerState::Idle
    }

    /// Create a timer state in work phase
    pub fn work_state(remaining_secs: u32) -> TimerState {
        TimerState::Working {
            remaining_seconds: remaining_secs,
        }
    }

    /// Create a timer state in break phase
    pub fn break_state(remaining_secs: u32, is_long_break: bool) -> TimerState {
        if is_long_break {
            TimerState::LongBreak {
                remaining_seconds: remaining_secs,
            }
        } else {
            TimerState::ShortBreak {
                remaining_seconds: remaining_secs,
            }
        }
    }

    /// Create a paused timer state
    pub fn paused_state() -> TimerState {
        let working_state = TimerState::Working {
            remaining_seconds: 10 * 60,
        };
        TimerState::Paused {
            paused_from: Box::new(working_state),
            remaining_seconds: 10 * 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_default_timer_config() {
        let config = TimerFixtures::default_config();
        assert_eq!(config.work_duration, Duration::from_secs(25 * 60));
        assert_eq!(config.sessions_until_long_break, 4);
    }

    #[test]
    fn creates_fast_timer_config() {
        let config = TimerFixtures::fast_config();

        assert_eq!(config.work_duration, Duration::from_secs(60));
        assert_eq!(config.short_break_duration, Duration::from_secs(60));
        assert_eq!(config.long_break_duration, Duration::from_secs(60));
        assert_eq!(config.sessions_until_long_break, 2);
    }

    #[test]
    fn creates_work_state() {
        let state = TimerFixtures::work_state(600);
        assert!(matches!(state, TimerState::Working { .. }));
        assert_eq!(state.remaining_seconds(), 600);
        assert!(state.is_running());
    }

    #[test]
    fn creates_break_states() {
        let short_break = TimerFixtures::break_state(300, false);
        assert!(matches!(short_break, TimerState::ShortBreak { .. }));
        // Session count is now tracked in Task, not TimerState

        let long_break = TimerFixtures::break_state(900, true);
        assert!(matches!(long_break, TimerState::LongBreak { .. }));
        // Session count is now tracked in Task, not TimerState
    }
}
