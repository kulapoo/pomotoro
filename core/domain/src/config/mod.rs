pub mod appearance;
pub mod audio;
#[allow(clippy::module_inception)]
mod config;
pub mod events;
pub mod general;
pub mod notification;
pub mod repo;

pub use self::config::Config;
pub use appearance::{AppearanceConfig, Theme};
pub use audio::AudioConfig;
pub use events::{ConfigReset, ConfigUpdated};
pub use general::{GeneralConfig, TaskCyclingBehavior};
pub use notification::{NotificationConfig, NotificationPosition};
pub use repo::ConfigRepository;
