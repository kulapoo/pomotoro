use std::time::Duration;

use crate::utils::{assert_utils, setup::setup_ctx};
use domain::{Config, TimerConfiguration, event_names};
use usecases::{UpdateConfigCmd, get_config, update_config};

#[tokio::test]
async fn config_should_load_default_configuration() {
    let ctx = setup_ctx("config_should_load_default_configuration").await;
    let config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(config, Config::default());
}

#[tokio::test]
async fn should_update_timer_durations_in_config() {
    // GIVEN
    let ctx = setup_ctx("should_update_timer_durations_in_config").await;
    let config = get_config(ctx.config_repo.clone()).await.unwrap();

    // WHEN
    let mut new_timer_config = config.timer.clone();
    new_timer_config.work_duration = Duration::from_secs(30 * 60); // 30 minutes
    new_timer_config.short_break_duration = Duration::from_secs(10 * 60); // 10 minutes

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(new_timer_config),
            ..Default::default()
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // THEN
    assert!(result.is_ok());

    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::config::CONFIG_UPDATED,
    );
}
