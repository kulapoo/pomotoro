use domain::{AudioLibrary, AudioAsset, AudioCategory, Result, Error};
use usecases::audio::manage_library::AudioLibraryService;
use std::sync::RwLock;

pub struct InMemoryAudioLibraryService {
    library: RwLock<AudioLibrary>,
}

impl InMemoryAudioLibraryService {
    pub fn new() -> Self {
        let mut library = AudioLibrary::new();
        
        // Add default notification sounds
        library.add_asset(AudioAsset {
            id: "session-complete-bell".to_string(),
            name: "Session Complete Bell".to_string(),
            file_path: "assets/audio/bell.mp3".into(),
            category: AudioCategory::NotificationSound,
            duration_ms: Some(2000),
        });
        
        library.add_asset(AudioAsset {
            id: "break-complete-chime".to_string(),
            name: "Break Complete Chime".to_string(),
            file_path: "assets/audio/chime.mp3".into(),
            category: AudioCategory::NotificationSound,
            duration_ms: Some(1500),
        });
        
        library.add_asset(AudioAsset {
            id: "task-complete-success".to_string(),
            name: "Task Complete Success".to_string(),
            file_path: "assets/audio/success.mp3".into(),
            category: AudioCategory::NotificationSound,
            duration_ms: Some(2500),
        });
        
        library.add_asset(AudioAsset {
            id: "phase-transition-soft".to_string(),
            name: "Phase Transition".to_string(),
            file_path: "assets/audio/transition.mp3".into(),
            category: AudioCategory::NotificationSound,
            duration_ms: Some(1000),
        });
        
        Self {
            library: RwLock::new(library),
        }
    }
}

impl Default for InMemoryAudioLibraryService {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioLibraryService for InMemoryAudioLibraryService {
    fn get_library(&self) -> Result<AudioLibrary> {
        Ok(self.library.read()
            .map_err(|e| Error::ConfigurationError {
                message: format!("Failed to read library: {e}"),
            })?
            .clone())
    }

    fn add_asset(&mut self, asset: AudioAsset) -> Result<()> {
        self.library.write()
            .map_err(|e| Error::ConfigurationError {
                message: format!("Failed to write library: {e}"),
            })?
            .add_asset(asset);
        Ok(())
    }

    fn remove_asset(&mut self, asset_id: &str) -> Result<bool> {
        Ok(self.library.write()
            .map_err(|e| Error::ConfigurationError {
                message: format!("Failed to write library: {e}"),
            })?
            .remove_asset(asset_id)
            .is_some())
    }

    fn get_asset(&self, asset_id: &str) -> Result<Option<AudioAsset>> {
        Ok(self.library.read()
            .map_err(|e| Error::ConfigurationError {
                message: format!("Failed to read library: {e}"),
            })?
            .get_asset(asset_id)
            .cloned())
    }

    fn get_assets_by_category(&self, category: AudioCategory) -> Result<Vec<AudioAsset>> {
        Ok(self.library.read()
            .map_err(|e| Error::ConfigurationError {
                message: format!("Failed to read library: {e}"),
            })?
            .assets
            .values()
            .filter(|asset| asset.category == category)
            .cloned()
            .collect())
    }

    fn save_library(&self, _library: &AudioLibrary) -> Result<()> {
        // In-memory implementation doesn't persist
        Ok(())
    }
}