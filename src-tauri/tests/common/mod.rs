pub mod app_builder;
pub mod mock_audio;
pub mod test_config;
pub mod time_utils;

use std::sync::Arc;
use pomotoro_lib::task::{InMemoryTaskRepository, TaskRepository};
use pomotoro_lib::config::{InMemoryConfigRepo, ConfigRepository};
use pomotoro_lib::timer::TimerService;
use pomotoro_lib::audio::AudioService;
use std::sync::Mutex;
use tempfile::TempDir;

pub struct TestContext {
    pub task_repo: TaskRepository,
    pub config_repo: ConfigRepository,
    pub timer_manager: Arc<TimerService>,
    pub audio_manager: Arc<Mutex<AudioService>>,
    pub _temp_dir: TempDir,
}

impl TestContext {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        let task_repo: TaskRepository = Arc::new(InMemoryTaskRepository::with_default_task());
        let config_repo: ConfigRepository = Arc::new(InMemoryConfigRepo::new());
        let timer_manager = Arc::new(TimerService::new());
        let audio_manager = Arc::new(Mutex::new(
            AudioService::new().expect("Failed to create audio manager")
        ));

        Self {
            task_repo,
            config_repo,
            timer_manager,
            audio_manager,
            _temp_dir: temp_dir,
        }
    }

    pub fn new_with_custom_task_repo(task_repo: TaskRepository) -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        let config_repo: ConfigRepository = Arc::new(InMemoryConfigRepo::new());
        let timer_manager = Arc::new(TimerService::new());
        let audio_manager = Arc::new(Mutex::new(
            AudioService::new().expect("Failed to create audio manager")
        ));

        Self {
            task_repo,
            config_repo,
            timer_manager,
            audio_manager,
            _temp_dir: temp_dir,
        }
    }
}

#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(val) => panic!("Expected Err, got Ok: {:?}", val),
            Err(e) => e,
        }
    };
}