use domain::*;
use crate::components::error_toast::ErrorInfo;

use super::SettingsViewModel;

impl SettingsViewModel {
    pub fn get_config(&self) -> Option<Config> {
        self.config.get()
    }

    pub fn is_saving(&self) -> bool {
        self.is_saving.get()
    }

    pub fn get_error_state(&self) -> Option<ErrorInfo> {
        self.error_state.get()
    }

    pub fn clear_error(&self) {
        self.set_error_state.set(None);
    }
}
