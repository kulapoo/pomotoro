use domain::event_names::commands::storage;
use leptos::task::spawn_local;

use crate::utils::invoke;

use super::SettingsViewModel;

impl SettingsViewModel {
    pub fn get_storage_path(&self) -> String {
        String::from("/home/user/.config/pomotoro")
    }

    pub fn browse_for_directory(&self) -> Option<String> {
        None
    }

    pub fn validate_storage_path(
        &self,
        path: &str,
    ) -> std::result::Result<(), String> {
        if path.is_empty() {
            return Err("Path cannot be empty".to_string());
        }
        if !path.starts_with('/') && !path.starts_with("C:\\") {
            return Err("Path must be absolute".to_string());
        }
        Ok(())
    }

    pub fn update_storage_path(&self, _path: String) {}

    pub fn open_data_directory(&self) {
        spawn_local(async move {
            let _ = invoke::<(), ()>(storage::OPEN_DATA_DIR, None).await;
        });
    }

    pub fn clear_all_data(&self) {
        spawn_local(async move {
            let _ = invoke::<(), ()>(storage::CLEAR_ALL_DATA, None).await;
        });
    }
}
