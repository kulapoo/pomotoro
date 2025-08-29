pub mod appearance;
pub mod audio;
mod config;
pub mod general;
pub mod notification;
pub mod repo;
pub mod task_defaults;

pub use self::config::Config;
pub use appearance::{AppearanceConfig, Theme};
pub use audio::AudioConfig;
pub use general::{GeneralConfig, TaskCyclingBehavior};
pub use notification::{NotificationConfig, NotificationPosition};
pub use repo::ConfigRepository;
pub use task_defaults::TaskDefaults;
