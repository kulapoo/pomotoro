use domain::{AudioAsset, AudioCategory, AudioLibrary, Error, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct AddAudioAssetCmd {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub category: AudioCategory,
    pub description: Option<String>,
    pub default_volume: f32,
}

#[derive(Debug, Clone)]
pub struct RemoveAudioAssetCmd {
    pub asset_id: String,
}

#[derive(Debug, Clone)]
pub struct GetAudioLibraryQuery {
    pub category_filter: Option<AudioCategory>,
}

/// Audio library service trait for dependency injection
pub trait AudioLibraryService: Send + Sync {
    fn get_library(&self) -> Result<AudioLibrary>;
    fn add_asset(&mut self, asset: AudioAsset) -> Result<()>;
    fn remove_asset(&mut self, asset_id: &str) -> Result<bool>;
    fn get_asset(&self, asset_id: &str) -> Result<Option<AudioAsset>>;
    fn get_assets_by_category(
        &self,
        category: AudioCategory,
    ) -> Result<Vec<AudioAsset>>;
    fn save_library(&self, library: &AudioLibrary) -> Result<()>;
}

pub async fn get_audio_library(
    library_service: &Arc<Mutex<dyn AudioLibraryService>>,
    query: GetAudioLibraryQuery,
) -> Result<AudioLibrary> {
    let service = library_service.lock().await;

    let mut library = service.get_library()?;

    if let Some(category) = query.category_filter {
        let filtered_assets = service.get_assets_by_category(category)?;
        library.assets.clear();
        for asset in filtered_assets {
            library.assets.insert(asset.id.clone(), asset);
        }
    }

    Ok(library)
}

pub async fn add_audio_asset(
    library_service: &Arc<Mutex<dyn AudioLibraryService>>,
    cmd: AddAudioAssetCmd,
) -> Result<AudioAsset> {
    if !(0.0..=1.0).contains(&cmd.default_volume) {
        return Err(Error::ConfigurationError {
            message: format!(
                "Default volume must be between 0.0 and 1.0, got {}",
                cmd.default_volume
            ),
        });
    }

    if cmd.id.trim().is_empty() {
        return Err(Error::ConfigurationError {
            message: "Asset ID cannot be empty".to_string(),
        });
    }

    if cmd.file_path.trim().is_empty() {
        return Err(Error::ConfigurationError {
            message: "File path cannot be empty".to_string(),
        });
    }

    if !std::path::Path::new(&cmd.file_path).exists() {
        return Err(Error::ConfigurationError {
            message: format!("Audio file does not exist: {}", cmd.file_path),
        });
    }

    let asset = AudioAsset {
        id: cmd.id,
        name: cmd.name,
        file_path: cmd.file_path.into(),
        category: cmd.category,
        duration_ms: None,
    };

    let mut service = library_service.lock().await;

    service.add_asset(asset.clone())?;
    Ok(asset)
}

pub async fn remove_audio_asset(
    library_service: &Arc<Mutex<dyn AudioLibraryService>>,
    cmd: RemoveAudioAssetCmd,
) -> Result<bool> {
    let mut service = library_service.lock().await;

    service.remove_asset(&cmd.asset_id)
}

pub async fn get_audio_asset(
    library_service: &Arc<Mutex<dyn AudioLibraryService>>,
    asset_id: String,
) -> Result<Option<AudioAsset>> {
    let service = library_service.lock().await;

    service.get_asset(&asset_id)
}

pub async fn get_assets_by_category(
    library_service: &Arc<Mutex<dyn AudioLibraryService>>,
    category: AudioCategory,
) -> Result<Vec<AudioAsset>> {
    let service = library_service.lock().await;

    service.get_assets_by_category(category)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    struct MockLibraryService {
        library: AudioLibrary,
    }

    impl MockLibraryService {
        fn new() -> Self {
            Self {
                library: AudioLibrary::new(),
            }
        }
    }

    impl AudioLibraryService for MockLibraryService {
        fn get_library(&self) -> Result<AudioLibrary> {
            Ok(self.library.clone())
        }

        fn add_asset(&mut self, asset: AudioAsset) -> Result<()> {
            self.library.add_asset(asset);
            Ok(())
        }

        fn remove_asset(&mut self, asset_id: &str) -> Result<bool> {
            Ok(self.library.remove_asset(asset_id).is_some())
        }

        fn get_asset(&self, asset_id: &str) -> Result<Option<AudioAsset>> {
            Ok(self.library.get_asset(asset_id).cloned())
        }

        fn get_assets_by_category(
            &self,
            category: AudioCategory,
        ) -> Result<Vec<AudioAsset>> {
            Ok(self
                .library
                .assets
                .values()
                .filter(|asset| asset.category == category)
                .cloned()
                .collect())
        }

        fn save_library(&self, _library: &AudioLibrary) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_add_audio_asset_successfully() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.mp3");
        File::create(&file_path)
            .unwrap()
            .write_all(b"fake audio data")
            .unwrap();

        let service: Arc<Mutex<dyn AudioLibraryService>> =
            Arc::new(Mutex::new(MockLibraryService::new()));

        let cmd = AddAudioAssetCmd {
            id: "test-asset".to_string(),
            name: "Test Asset".to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            category: AudioCategory::BackgroundAmbient,
            description: Some("Test description".to_string()),
            default_volume: 0.7,
        };

        let asset = add_audio_asset(&service, cmd).await.unwrap();

        assert_eq!(asset.id, "test-asset");
        assert_eq!(asset.name, "Test Asset");
        assert_eq!(asset.category, AudioCategory::BackgroundAmbient);
    }

    #[tokio::test]
    async fn should_fail_with_invalid_volume() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.mp3");
        File::create(&file_path).unwrap();

        let service: Arc<Mutex<dyn AudioLibraryService>> =
            Arc::new(Mutex::new(MockLibraryService::new()));

        let cmd = AddAudioAssetCmd {
            id: "test-asset".to_string(),
            name: "Test Asset".to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            category: AudioCategory::BackgroundAmbient,
            description: None,
            default_volume: 1.5,
        };

        let result = add_audio_asset(&service, cmd).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_fail_with_nonexistent_file() {
        let service: Arc<Mutex<dyn AudioLibraryService>> =
            Arc::new(Mutex::new(MockLibraryService::new()));

        let cmd = AddAudioAssetCmd {
            id: "test-asset".to_string(),
            name: "Test Asset".to_string(),
            file_path: "/nonexistent/file.mp3".to_string(),
            category: AudioCategory::BackgroundAmbient,
            description: None,
            default_volume: 0.5,
        };

        let result = add_audio_asset(&service, cmd).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_get_library_successfully() {
        let service: Arc<Mutex<dyn AudioLibraryService>> =
            Arc::new(Mutex::new(MockLibraryService::new()));

        let query = GetAudioLibraryQuery {
            category_filter: None,
        };

        let library = get_audio_library(&service, query).await.unwrap();
        assert!(library.assets.is_empty());
    }

    #[tokio::test]
    async fn should_remove_asset_successfully() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.mp3");
        File::create(&file_path).unwrap();

        let service: Arc<Mutex<dyn AudioLibraryService>> =
            Arc::new(Mutex::new(MockLibraryService::new()));

        let add_cmd = AddAudioAssetCmd {
            id: "test-asset".to_string(),
            name: "Test Asset".to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            category: AudioCategory::BackgroundAmbient,
            description: None,
            default_volume: 0.5,
        };
        add_audio_asset(&service, add_cmd).await.unwrap();

        let remove_cmd = RemoveAudioAssetCmd {
            asset_id: "test-asset".to_string(),
        };
        let removed = remove_audio_asset(&service, remove_cmd).await.unwrap();
        assert!(removed);
    }
}
