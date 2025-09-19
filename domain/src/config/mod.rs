pub mod appearance;
pub mod audio;
mod config;
pub mod general;
pub mod notification;
pub mod repo;
pub mod events;

pub use self::config::Config;
pub use appearance::{AppearanceConfig, Theme};
pub use audio::AudioConfig;
pub use general::{GeneralConfig, TaskCyclingBehavior};
pub use notification::{NotificationConfig, NotificationPosition};
pub use repo::ConfigRepository;
pub use events::{ConfigUpdated, ConfigReset};
