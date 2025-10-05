mod accessors;
mod initialization;
mod commands;
mod import_export;
mod storage;

use crate::components::error_toast::ErrorInfo;
use domain::*;
use leptos::prelude::*;

use crate::utils::ViewModel;

pub struct SettingsViewModel {
    pub(super) config: ReadSignal<Option<Config>>,
    pub(super) set_config: WriteSignal<Option<Config>>,
    pub(super) is_saving: ReadSignal<bool>,
    pub(super) set_is_saving: WriteSignal<bool>,
    pub(super) error_state: ReadSignal<Option<ErrorInfo>>,
    pub(super) set_error_state: WriteSignal<Option<ErrorInfo>>,
}

impl ViewModel for SettingsViewModel {
    type State = Option<Config>;

    fn new() -> Self {
        let (config, set_config) = signal(None::<Config>);
        let (is_saving, set_is_saving) = signal(false);
        let (error_state, set_error_state) = signal(None::<ErrorInfo>);

        let vm = Self {
            config,
            set_config,
            is_saving,
            set_is_saving,
            error_state,
            set_error_state,
        };

        vm.initialize();
        vm
    }

    fn state(&self) -> ReadSignal<Self::State> {
        self.config
    }

    fn set_state(&self) -> WriteSignal<Self::State> {
        self.set_config
    }
}
