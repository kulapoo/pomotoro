//! Timer state machine representation.
//!
//! This module defines the pure state representation for the Pomodoro timer,
//! providing immutable state queries without side effects or business logic.

use serde::{Deserialize, Serialize};

/// Represents all possible states of a Pomodoro timer.
///
/// Each state variant contains the data relevant to that state.
/// The state machine ensures type-safe transitions and makes invalid states unrepresentable.
///
/// # Serialization
///
/// `TimerState` serializes as an internally-tagged enum keyed on `"state"`:
///
/// ```json
/// { "state": "Idle" }
/// { "state": "Working", "remaining_seconds": 1500 }
/// { "state": "Paused", "paused_from": { "state": "ShortBreak", ... }, "remaining_seconds": 750 }
/// ```
///
/// Deserialization additionally accepts the legacy externally-tagged shape
/// `{"state": "...", "data": { ... }}`, so previously-serialized data still
/// loads. New output is always the flat form above.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
#[serde(tag = "state")]
pub enum TimerState {
    /// Timer is stopped and ready to start.
    #[default]
    Idle,

    /// Active work session.
    Working { remaining_seconds: u32 },

    /// Short break between work sessions.
    ShortBreak { remaining_seconds: u32 },

    /// Long break after multiple work sessions.
    LongBreak { remaining_seconds: u32 },

    /// Timer is paused, preserving the previous state.
    Paused {
        paused_from: Box<TimerState>,
        remaining_seconds: u32,
    },
}

/// Accepts both the current flat (internally-tagged) wire form and the
/// legacy `{ "state", "data" }` form. Flat is preferred; legacy is the
/// fallback so older serialized data still deserializes cleanly.
impl<'de> Deserialize<'de> for TimerState {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Canonical flat representation: variant fields merged with the tag.
        #[derive(Deserialize)]
        #[serde(tag = "state")]
        enum FlatRepr {
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

        // Legacy representation: variant data nested under a "data" key.
        #[derive(Deserialize)]
        #[serde(tag = "state", content = "data")]
        enum LegacyRepr {
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

        fn from_flat(r: FlatRepr) -> TimerState {
            match r {
                FlatRepr::Idle => TimerState::Idle,
                FlatRepr::Working { remaining_seconds } => {
                    TimerState::Working { remaining_seconds }
                }
                FlatRepr::ShortBreak { remaining_seconds } => {
                    TimerState::ShortBreak { remaining_seconds }
                }
                FlatRepr::LongBreak { remaining_seconds } => {
                    TimerState::LongBreak { remaining_seconds }
                }
                FlatRepr::Paused {
                    paused_from,
                    remaining_seconds,
                } => TimerState::Paused {
                    paused_from,
                    remaining_seconds,
                },
            }
        }

        fn from_legacy(r: LegacyRepr) -> TimerState {
            match r {
                LegacyRepr::Idle => TimerState::Idle,
                LegacyRepr::Working { remaining_seconds } => {
                    TimerState::Working { remaining_seconds }
                }
                LegacyRepr::ShortBreak { remaining_seconds } => {
                    TimerState::ShortBreak { remaining_seconds }
                }
                LegacyRepr::LongBreak { remaining_seconds } => {
                    TimerState::LongBreak { remaining_seconds }
                }
                LegacyRepr::Paused {
                    paused_from,
                    remaining_seconds,
                } => TimerState::Paused {
                    paused_from,
                    remaining_seconds,
                },
            }
        }

        let value = serde_json::Value::deserialize(deserializer)?;

        match serde_json::from_value::<FlatRepr>(value.clone()) {
            Ok(flat) => Ok(from_flat(flat)),
            Err(flat_err) => {
                match serde_json::from_value::<LegacyRepr>(value) {
                    Ok(legacy) => Ok(from_legacy(legacy)),
                    Err(legacy_err) => Err(serde::de::Error::custom(format!(
                        "TimerState: failed to parse as flat or legacy form \
                     (flat: {flat_err}; legacy: {legacy_err})"
                    ))),
                }
            }
        }
    }
}

impl TimerState {
    /// Creates a new timer in idle state.
    pub fn new() -> Self {
        Self::Idle
    }

    /// Returns the remaining seconds for the current state.
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

    /// Checks if the timer is actively counting down.
    pub fn is_running(&self) -> bool {
        matches!(
            self,
            Self::Working { .. }
                | Self::ShortBreak { .. }
                | Self::LongBreak { .. }
        )
    }

    /// Checks if the timer is paused.
    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused { .. })
    }

    /// Checks if the timer is idle (stopped).
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Checks if currently in a work phase (including paused work).
    pub fn is_work_phase(&self) -> bool {
        match self {
            Self::Working { .. } => true,
            Self::Paused { paused_from, .. } => paused_from.is_work_phase(),
            _ => false,
        }
    }

    /// Checks if currently in a break phase (including paused break).
    pub fn is_break_phase(&self) -> bool {
        match self {
            Self::ShortBreak { .. } | Self::LongBreak { .. } => true,
            Self::Paused { paused_from, .. } => paused_from.is_break_phase(),
            _ => false,
        }
    }

    /// Returns the timer's operational status.
    pub fn status(&self) -> super::Status {
        match self {
            Self::Idle => super::Status::Stopped,
            Self::Working { .. }
            | Self::ShortBreak { .. }
            | Self::LongBreak { .. } => super::Status::Running,
            Self::Paused { .. } => super::Status::Paused,
        }
    }

    /// Returns the current phase (Work, ShortBreak, or LongBreak).
    pub fn phase(&self) -> super::Phase {
        match self {
            Self::Idle => super::Phase::Work,
            Self::Working { .. } => super::Phase::Work,
            Self::ShortBreak { .. } => super::Phase::ShortBreak,
            Self::LongBreak { .. } => super::Phase::LongBreak,
            Self::Paused { paused_from, .. } => paused_from.phase(),
        }
    }

    /// Creates a new state with updated remaining seconds.
    /// Returns unchanged state for Idle.
    pub fn with_remaining_seconds(&self, seconds: u32) -> Self {
        match self {
            Self::Working { .. } => Self::Working {
                remaining_seconds: seconds,
            },
            Self::ShortBreak { .. } => Self::ShortBreak {
                remaining_seconds: seconds,
            },
            Self::LongBreak { .. } => Self::LongBreak {
                remaining_seconds: seconds,
            },
            Self::Paused { paused_from, .. } => Self::Paused {
                paused_from: paused_from.clone(),
                remaining_seconds: seconds,
            },
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
                remaining_seconds, ..
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
                remaining_seconds, ..
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
                remaining_seconds, ..
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

    #[test]
    fn should_serialize_as_flat_internally_tagged() {
        // No "data" wrapper; variant fields sit beside the "state" tag.
        let working = TimerState::Working {
            remaining_seconds: 1500,
        };
        let json = serde_json::to_value(&working).unwrap();
        assert_eq!(json["state"], "Working");
        assert_eq!(json["remaining_seconds"], 1500);
        assert!(
            json.get("data").is_none(),
            "flat form must not contain a \"data\" key"
        );

        let idle_json = serde_json::to_value(&TimerState::Idle).unwrap();
        assert_eq!(idle_json["state"], "Idle");

        let paused = TimerState::Paused {
            paused_from: Box::new(TimerState::ShortBreak {
                remaining_seconds: 300,
            }),
            remaining_seconds: 120,
        };
        let paused_json = serde_json::to_value(&paused).unwrap();
        assert_eq!(paused_json["state"], "Paused");
        assert_eq!(paused_json["remaining_seconds"], 120);
        assert_eq!(paused_json["paused_from"]["state"], "ShortBreak");
        assert_eq!(paused_json["paused_from"]["remaining_seconds"], 300);
    }

    #[test]
    fn should_round_trip_flat_form() {
        let original = TimerState::Paused {
            paused_from: Box::new(TimerState::Working {
                remaining_seconds: 600,
            }),
            remaining_seconds: 542,
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TimerState = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn should_deserialize_legacy_data_wrapped_form() {
        // Old externally-tagged shape with variant data under "data".
        let json = r#"{"state":"Working","data":{"remaining_seconds":42}}"#;
        let state: TimerState = serde_json::from_str(json).unwrap();
        assert_eq!(
            state,
            TimerState::Working {
                remaining_seconds: 42
            }
        );

        let paused_json = r#"{"state":"Paused","data":{"paused_from":{"state":"LongBreak","data":{"remaining_seconds":60}},"remaining_seconds":30}}"#;
        let paused: TimerState = serde_json::from_str(paused_json).unwrap();
        assert_eq!(
            paused,
            TimerState::Paused {
                paused_from: Box::new(TimerState::LongBreak {
                    remaining_seconds: 60
                }),
                remaining_seconds: 30,
            }
        );
    }

    #[test]
    fn should_prefer_flat_form_when_ambiguous() {
        // Flat "data" field would be ignored by the flat parser; ensure the
        // flat path wins and reads the root-level remaining_seconds.
        let json = r#"{"state":"Working","remaining_seconds":7}"#;
        let state: TimerState = serde_json::from_str(json).unwrap();
        assert_eq!(
            state,
            TimerState::Working {
                remaining_seconds: 7
            }
        );
    }
}
