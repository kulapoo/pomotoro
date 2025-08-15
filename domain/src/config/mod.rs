pub mod appearance;
pub mod audio;
mod config;
pub mod general;
pub mod notification;
pub mod config_repo;
pub mod task_defaults;
#[cfg(any(test, feature = "test-utils"))]
pub mod test_repository;

pub use appearance::{Theme, AppearanceConfig};
pub use audio::AudioConfig;
pub use self::config::Config;
pub use general::{TaskCyclingBehavior, GeneralConfig};
pub use notification::{NotificationPosition, NotificationConfig};
pub use config_repo::ConfigRepository;
pub use task_defaults::TaskDefaults;