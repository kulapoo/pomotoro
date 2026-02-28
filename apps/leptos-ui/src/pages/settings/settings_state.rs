use crate::utils::invoke;
use domain::event_names::commands;
use domain::*;
use leptos::prelude::*;
use leptos::task::spawn_local;

pub async fn get_global_config() -> std::result::Result<Config, String> {
    invoke::<Config, ()>(commands::config::GET_GLOBAL, None).await
}

#[allow(dead_code)]
pub async fn save_global_config(
    config: Config,
) -> std::result::Result<(), String> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct SaveGlobalConfigArgs {
        config: Config,
    }

    let args = SaveGlobalConfigArgs { config };
    invoke::<(), _>(commands::config::SAVE_GLOBAL, Some(args)).await
}

#[allow(dead_code)]
pub async fn update_general(
    general: GeneralConfig,
) -> std::result::Result<Config, String> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct UpdateGeneralArgs {
        preferences: GeneralConfig,
    }

    let args = UpdateGeneralArgs {
        preferences: general,
    };
    invoke::<Config, _>(commands::config::UPDATE_GENERAL, Some(args)).await
}

#[allow(dead_code)]
pub async fn update_notification_preferences(
    preferences: NotificationConfig,
) -> std::result::Result<Config, String> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct UpdateNotificationArgs {
        preferences: NotificationConfig,
    }

    let args = UpdateNotificationArgs { preferences };
    invoke::<Config, _>(commands::config::UPDATE_NOTIFICATIONS, Some(args))
        .await
}

#[allow(dead_code)]
pub async fn update_appearance(
    appearance: AppearanceConfig,
) -> std::result::Result<Config, String> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct UpdateAppearanceArgs {
        preferences: AppearanceConfig,
    }

    let args = UpdateAppearanceArgs {
        preferences: appearance,
    };
    invoke::<Config, _>(commands::config::UPDATE_APPEARANCE, Some(args)).await
}

#[allow(dead_code)]
pub async fn update_audio_config(
    task_id: TaskId,
    audio_config: AudioConfig,
) -> std::result::Result<Config, String> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct UpdateAudioArgs {
        task_id: TaskId,
        audio_config: AudioConfig,
    }

    let args = UpdateAudioArgs {
        task_id,
        audio_config,
    };
    invoke::<Config, _>(commands::config::UPDATE_AUDIO, Some(args)).await
}

#[allow(dead_code)]
pub async fn update_default_timings(
    timer: TimerConfiguration,
) -> std::result::Result<Config, String> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct UpdateTimingsArgs {
        timer: TimerConfiguration,
    }

    let args = UpdateTimingsArgs { timer };
    invoke::<Config, _>(commands::config::UPDATE_TIMINGS, Some(args)).await
}

#[allow(dead_code)]
pub async fn reset_global_config_to_defaults()
-> std::result::Result<Config, String> {
    invoke::<Config, ()>(commands::config::RESET_TO_DEFAULTS, None).await
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct ConfigResource {
    pub config: ReadSignal<Option<Config>>,
    set_config: WriteSignal<Option<Config>>,
}

impl ConfigResource {
    pub fn new() -> Self {
        let (config, set_config) = signal(None::<Config>);

        let set_config_clone = set_config;
        spawn_local(async move {
            if let Ok(initial_config) = get_global_config().await {
                set_config_clone.set(Some(initial_config));
            }
        });

        Self { config, set_config }
    }

    #[allow(dead_code)]
    pub async fn update_and_refetch<F, R>(
        &self,
        update_fn: F,
    ) -> std::result::Result<(), String>
    where
        F: FnOnce() -> R + 'static,
        R: std::future::Future<Output = std::result::Result<Config, String>>,
    {
        let new_config = update_fn().await?;
        self.set_config.set(Some(new_config));
        Ok(())
    }

    #[allow(dead_code)]
    pub fn refetch(&self) {
        let set_config = self.set_config;
        spawn_local(async move {
            if let Ok(config) = get_global_config().await {
                set_config.set(Some(config));
            }
        });
    }
}
