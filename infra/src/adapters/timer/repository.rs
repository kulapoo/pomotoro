use domain::{TimerState, Result as DomainResult, Error};
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

    /// Save timer state to persistent storage
    pub async fn save_state(&self, state: &TimerState) -> DomainResult<()> {
        let state_path = self.get_state_file_path().await
            .map_err(|e| Error::RepositoryError { message: e })?;

        let json = serde_json::to_string_pretty(state)
            .map_err(|e| Error::RepositoryError { 
                message: format!("Failed to serialize state: {e}") 
            })?;

        tokio::fs::write(state_path, json)
            .await
            .map_err(|e| Error::RepositoryError { 
                message: format!("Failed to write state file: {e}") 
            })?;

        Ok(())
    }

    /// Load timer state from persistent storage
    pub async fn load_state(&self) -> DomainResult<Option<TimerState>> {
        let state_path = self.get_state_file_path().await
            .map_err(|e| Error::RepositoryError { message: e })?;

        if !state_path.exists() {
            return Ok(None);
        }

        let json = tokio::fs::read_to_string(state_path)
            .await
            .map_err(|e| Error::RepositoryError { 
                message: format!("Failed to read state file: {e}") 
            })?;

        let saved_state: TimerState = serde_json::from_str(&json)
            .map_err(|e| Error::RepositoryError { 
                message: format!("Failed to deserialize state: {e}") 
            })?;

        Ok(Some(saved_state))
    }
}