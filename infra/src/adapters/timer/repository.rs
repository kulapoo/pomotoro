use async_trait::async_trait;
use domain::{Timer, TimerRepository};
use domain::timer::{Error, Result as DomainResult};
use std::path::PathBuf;

/// Repository for persisting timer state to file system
///
/// This adapter handles the persistence concern that was previously
/// mixed into the TimerService. It provides clean separation between
/// timer logic and file I/O operations.
pub struct FileTimerStateRepository;

impl Default for FileTimerStateRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl FileTimerStateRepository {
    pub fn new() -> Self {
        Self
    }

    /// Get the path to the timer state file
    async fn get_state_file_path(&self) -> Result<PathBuf, String> {
        let app_data_dir = dirs::data_dir()
            .ok_or_else(|| "Failed to get app data dir".to_string())?
            .join("pomotoro");

        tokio::fs::create_dir_all(&app_data_dir)
            .await
            .map_err(|e| format!("Failed to create app data dir: {e}"))?;

        Ok(app_data_dir.join("timer_state.json"))
    }
}

#[async_trait]
impl TimerRepository for FileTimerStateRepository {
    async fn get(&self) -> DomainResult<Timer> {
        let state_path = self
            .get_state_file_path()
            .await
            .map_err(|e| Error::RepositoryError { message: e })?;

        if !state_path.exists() {
            // Return default timer if file doesn't exist
            let timer = Timer::default_timer();
            self.save(&timer).await?;
            return Ok(timer);
        }

        let json =
            tokio::fs::read_to_string(state_path).await.map_err(|e| {
                Error::RepositoryError {
                    message: format!("Failed to read timer file: {e}"),
                }
            })?;

        let timer: Timer =
            serde_json::from_str(&json).map_err(|e| {
                Error::RepositoryError {
                    message: format!("Failed to deserialize timer: {e}"),
                }
            })?;

        Ok(timer)
    }

    async fn save(&self, timer: &Timer) -> DomainResult<()> {
        let state_path = self
            .get_state_file_path()
            .await
            .map_err(|e| Error::RepositoryError { message: e })?;

        let json = serde_json::to_string_pretty(timer).map_err(|e| {
            Error::RepositoryError {
                message: format!("Failed to serialize timer: {e}"),
            }
        })?;

        tokio::fs::write(state_path, json).await.map_err(|e| {
            Error::RepositoryError {
                message: format!("Failed to write timer file: {e}"),
            }
        })?;

        Ok(())
    }
}
