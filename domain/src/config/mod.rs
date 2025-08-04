pub mod appearance;
pub mod audio;
pub mod config;
pub mod general;
pub mod notification;
pub mod repo;
pub mod task_defaults;
#[cfg(any(test, feature = "test-utils"))]
pub mod test_repository;

pub use appearance::{Theme, AppearanceConfig};
pub use audio::AudioConfig;
pub use config::Config;
pub use general::{TaskCyclingBehavior, GeneralConfig};
pub use notification::{NotificationPosition, NotificationConfig};
pub use repo::ConfigRepository;
pub use task_defaults::TaskDefaults;