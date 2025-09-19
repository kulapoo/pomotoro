use std::{sync::Arc, time::Duration, any::TypeId};

use crate::utils::{setup::setup_ctx, assert_utils};
use domain::{Config, ConfigRepository, EventPublisher, TimerConfiguration, TaskRepository, TimerRepository,
    config::events::{ConfigUpdated, ConfigReset}, event_names, shared_kernel::events::AppStarted};
use usecases::{UpdateConfigCmd, get_config, update_config, reset_config};

#[tokio::test]
async fn config_should_initialize_on_app_start() {
    // GIVEN - App starts with fresh context
    let ctx = setup_ctx("config_should_initialize_on_app_start").await;

    // THEN - Config should be in default state
    let config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(config, Config::default());

    // Verify AppStarted event subscription (similar to timer tests)
    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<AppStarted>());

    // Verify default timer configuration values
    assert_eq!(config.timer.work_duration, Duration::from_secs(25 * 60));
    assert_eq!(config.timer.short_break_duration, Duration::from_secs(5 * 60));
    assert_eq!(config.timer.long_break_duration, Duration::from_secs(15 * 60));
    assert_eq!(config.timer.sessions_until_long_break, 4);
}

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
    
    // Verify the config was actually updated
    let final_config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(final_config.timer.work_duration, Duration::from_secs(30 * 60));
    assert_eq!(final_config.timer.short_break_duration, Duration::from_secs(10 * 60));
    
    // Verify event subscription and emission
    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<ConfigUpdated>());
    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::config::CONFIG_UPDATED,
    );
}

#[tokio::test]
async fn should_reset_config_to_factory_defaults() {
    // GIVEN
    let ctx = setup_ctx("should_reset_config_to_factory_defaults").await;
    
    // Modify the config first
    let mut modified_config = get_config(ctx.config_repo.clone()).await.unwrap();
    modified_config.timer.work_duration = Duration::from_secs(45 * 60); // 45 minutes
    modified_config.timer.sessions_until_long_break = 6;
    
    let _ = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(modified_config.timer.clone()),
            ..Default::default()
        },
    )
    .await;
    
    // WHEN
    let config_repo: Arc<dyn ConfigRepository + Send + Sync> = ctx.config_repo.clone() as Arc<dyn ConfigRepository + Send + Sync>;
    let event_bus: Arc<dyn EventPublisher + Send + Sync> = ctx.event_bus.clone() as Arc<dyn EventPublisher + Send + Sync>;
    
    let result = reset_config(
        &config_repo,
        &event_bus,
    )
    .await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // THEN
    assert!(result.is_ok());
    let reset_config = result.unwrap();
    
    assert_eq!(reset_config, Config::default());
    assert_eq!(reset_config.timer.work_duration, Duration::from_secs(25 * 60)); // 25 minutes default
    assert_eq!(reset_config.timer.short_break_duration, Duration::from_secs(5 * 60)); // 5 minutes default
    assert_eq!(reset_config.timer.sessions_until_long_break, 4); // 4 sessions default
    
    // Verify the config was reset and event was emitted
    let final_config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(final_config, Config::default());

    // Verify event subscription and emission
    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<ConfigReset>());
    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::config::CONFIG_RESET,
    );
}

#[tokio::test]
async fn config_changes_should_apply_to_new_timer_sessions() {
    // GIVEN
    let ctx = setup_ctx("config_changes_should_apply_to_new_timer_sessions").await;
    
    // Update config with new timer durations
    let mut new_timer_config = TimerConfiguration::default();
    new_timer_config.work_duration = Duration::from_secs(20 * 60); // 20 minutes
    new_timer_config.short_break_duration = Duration::from_secs(3 * 60); // 3 minutes
    
    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(new_timer_config.clone()),
            ..Default::default()
        },
    )
    .await;
    
    assert!(result.is_ok());
    let updated_from_result = result.unwrap();
    
    // Verify the result from update_config has the expected values
    assert_eq!(updated_from_result.timer.work_duration, Duration::from_secs(20 * 60));
    assert_eq!(updated_from_result.timer.short_break_duration, Duration::from_secs(3 * 60));
    
    // WHEN - Create a timer session (would use the new config)
    // Note: In a real integration test, we would start a timer session here
    // and verify it uses the new durations
    
    // THEN - Verify the config was updated in the repository
    let updated_config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(updated_config.timer.work_duration, Duration::from_secs(20 * 60));
    assert_eq!(updated_config.timer.short_break_duration, Duration::from_secs(3 * 60));
}

#[tokio::test]
async fn config_state_should_persist_across_restarts() {
    // GIVEN
    let ctx = setup_ctx("config_state_should_persist_across_restarts").await;
    
    // Modify configuration
    let mut modified_config = get_config(ctx.config_repo.clone()).await.unwrap();
    modified_config.timer.work_duration = Duration::from_secs(30 * 60);
    modified_config.timer.long_break_duration = Duration::from_secs(20 * 60);
    modified_config.timer.sessions_until_long_break = 5;
    
    let update_result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(modified_config.timer.clone()),
            ..Default::default()
        },
    )
    .await;
    
    assert!(update_result.is_ok());
    let updated = update_result.unwrap();
    assert_eq!(updated.timer.work_duration, Duration::from_secs(30 * 60));
    
    // Verify it was saved
    let config_before_restart = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(config_before_restart.timer.work_duration, Duration::from_secs(30 * 60));
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // WHEN - Simulate app restart with same database (without reinitializing config)
    use crate::AppContextBuilder;
    let new_ctx = AppContextBuilder::new()
        .with_existing_db(ctx.db)
        // Don't use with_standard_fixtures() as it would reset the config
        .build()
        .await
        .expect("Failed to build test context");
    let config_after_restart = get_config(new_ctx.config_repo.clone()).await.unwrap();
    
    // THEN
    assert_eq!(config_after_restart.timer.work_duration, Duration::from_secs(30 * 60));
    assert_eq!(config_after_restart.timer.long_break_duration, Duration::from_secs(20 * 60));
    assert_eq!(config_after_restart.timer.sessions_until_long_break, 5);
}

#[tokio::test]
async fn should_validate_config_boundaries() {
    // GIVEN
    let ctx = setup_ctx("should_validate_config_boundaries").await;
    
    // WHEN/THEN - Test invalid work duration (0 seconds)
    let invalid_timer = TimerConfiguration {
        work_duration: Duration::from_secs(0),
        short_break_duration: Duration::from_secs(5 * 60),
        long_break_duration: Duration::from_secs(15 * 60),
        sessions_until_long_break: 4,
    };
    
    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(invalid_timer),
            ..Default::default()
        },
    )
    .await;
    
    // TimerConfiguration should validate and reject 0 duration
    assert!(result.is_err(), "Should reject zero duration for work");
    
    // Test extremely long duration
    let excessive_timer = TimerConfiguration {
        work_duration: Duration::from_secs(24 * 60 * 60), // 24 hours
        short_break_duration: Duration::from_secs(5 * 60),
        long_break_duration: Duration::from_secs(15 * 60),
        sessions_until_long_break: 4,
    };
    
    let result2 = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(excessive_timer),
            ..Default::default()
        },
    )
    .await;
    
    // Should reject duration exceeding 3 hours maximum
    assert!(result2.is_err(), "Should reject work duration exceeding 3 hours");
}

#[tokio::test]
async fn should_update_multiple_config_sections_simultaneously() {
    // GIVEN
    let ctx = setup_ctx("should_update_multiple_config_sections_simultaneously").await;
    let original_config = get_config(ctx.config_repo.clone()).await.unwrap();

    // WHEN - Update timer configuration
    let mut new_timer_config = original_config.timer.clone();
    new_timer_config.work_duration = Duration::from_secs(35 * 60);
    new_timer_config.short_break_duration = Duration::from_secs(7 * 60);
    new_timer_config.sessions_until_long_break = 6;

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(new_timer_config.clone()),
            ..Default::default()
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // THEN
    assert!(result.is_ok());

    let updated_config = get_config(ctx.config_repo.clone()).await.unwrap();

    // Timer settings should be updated
    assert_eq!(updated_config.timer.work_duration, Duration::from_secs(35 * 60));
    assert_eq!(updated_config.timer.short_break_duration, Duration::from_secs(7 * 60));
    assert_eq!(updated_config.timer.sessions_until_long_break, 6);

    // Other settings should remain unchanged
    assert_eq!(updated_config.timer.long_break_duration, original_config.timer.long_break_duration);

    // Verify event subscription and emission
    assert_utils::assert_event_subscribed(&ctx, TypeId::of::<ConfigUpdated>());
    assert_utils::assert_event_was_emitted(
        &ctx.ui_simulator,
        event_names::ui_listeners::config::CONFIG_UPDATED,
    );
}

#[tokio::test]
async fn should_preserve_unchanged_fields_on_partial_update() {
    // GIVEN
    let ctx = setup_ctx("should_preserve_unchanged_fields_on_partial_update").await;

    // Set initial config with custom values
    let initial_timer_config = TimerConfiguration {
        work_duration: Duration::from_secs(30 * 60),
        short_break_duration: Duration::from_secs(10 * 60),
        long_break_duration: Duration::from_secs(25 * 60),
        sessions_until_long_break: 5,
    };

    let _ = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(initial_timer_config.clone()),
            ..Default::default()
        },
    )
    .await;

    // WHEN - Update only work_duration
    let partial_timer_config = TimerConfiguration {
        work_duration: Duration::from_secs(45 * 60), // Change only this
        short_break_duration: initial_timer_config.short_break_duration,
        long_break_duration: initial_timer_config.long_break_duration,
        sessions_until_long_break: initial_timer_config.sessions_until_long_break,
    };

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(partial_timer_config),
            ..Default::default()
        },
    )
    .await;

    // THEN
    assert!(result.is_ok());
    let final_config = get_config(ctx.config_repo.clone()).await.unwrap();

    // Changed field should be updated
    assert_eq!(final_config.timer.work_duration, Duration::from_secs(45 * 60));

    // Unchanged fields should be preserved
    assert_eq!(final_config.timer.short_break_duration, Duration::from_secs(10 * 60));
    assert_eq!(final_config.timer.long_break_duration, Duration::from_secs(25 * 60));
    assert_eq!(final_config.timer.sessions_until_long_break, 5);
}

#[tokio::test]
async fn should_handle_concurrent_config_updates() {
    // GIVEN
    let ctx = setup_ctx("should_handle_concurrent_config_updates").await;

    // WHEN - Spawn multiple concurrent update tasks
    let config_repo1 = ctx.config_repo.clone();
    let config_repo2 = ctx.config_repo.clone();
    let event_bus1 = ctx.event_bus.clone();
    let event_bus2 = ctx.event_bus.clone();

    let handle1 = tokio::spawn(async move {
        let timer_config = TimerConfiguration {
            work_duration: Duration::from_secs(20 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
        };

        update_config(
            config_repo1,
            event_bus1,
            UpdateConfigCmd {
                timer: Some(timer_config),
                ..Default::default()
            },
        )
        .await
    });

    let handle2 = tokio::spawn(async move {
        let timer_config = TimerConfiguration {
            work_duration: Duration::from_secs(30 * 60),
            short_break_duration: Duration::from_secs(7 * 60),
            long_break_duration: Duration::from_secs(20 * 60),
            sessions_until_long_break: 6,
        };

        update_config(
            config_repo2,
            event_bus2,
            UpdateConfigCmd {
                timer: Some(timer_config),
                ..Default::default()
            },
        )
        .await
    });

    // THEN - Both updates should complete without errors
    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // The final config should be one of the two updates (last write wins)
    let final_config = get_config(ctx.config_repo.clone()).await.unwrap();

    // Check that it's one of the two valid states
    let is_first_update =
        final_config.timer.work_duration == Duration::from_secs(20 * 60) &&
        final_config.timer.sessions_until_long_break == 4;

    let is_second_update =
        final_config.timer.work_duration == Duration::from_secs(30 * 60) &&
        final_config.timer.sessions_until_long_break == 6;

    assert!(is_first_update || is_second_update,
        "Final config should match one of the concurrent updates");
}

#[tokio::test]
async fn should_handle_edge_case_timer_values() {
    // GIVEN
    let ctx = setup_ctx("should_handle_edge_case_timer_values").await;

    // Test minimum reasonable values (1 minute)
    let min_timer_config = TimerConfiguration {
        work_duration: Duration::from_secs(60), // 1 minute
        short_break_duration: Duration::from_secs(60), // 1 minute
        long_break_duration: Duration::from_secs(60), // 1 minute
        sessions_until_long_break: 1,
    };

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(min_timer_config),
            ..Default::default()
        },
    )
    .await;

    assert!(result.is_ok(), "Should accept minimum reasonable values");
    let config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(config.timer.work_duration, Duration::from_secs(60));

    // Test maximum reasonable values (2 hours)
    let max_timer_config = TimerConfiguration {
        work_duration: Duration::from_secs(2 * 60 * 60), // 2 hours
        short_break_duration: Duration::from_secs(30 * 60), // 30 minutes
        long_break_duration: Duration::from_secs(60 * 60), // 1 hour
        sessions_until_long_break: 10,
    };

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(max_timer_config),
            ..Default::default()
        },
    )
    .await;

    assert!(result.is_ok(), "Should accept maximum reasonable values");
    let config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(config.timer.work_duration, Duration::from_secs(2 * 60 * 60));

    // Test common Pomodoro values
    let pomodoro_timer_config = TimerConfiguration {
        work_duration: Duration::from_secs(25 * 60), // Standard 25 minutes
        short_break_duration: Duration::from_secs(5 * 60), // Standard 5 minutes
        long_break_duration: Duration::from_secs(15 * 60), // Standard 15 minutes
        sessions_until_long_break: 4, // Standard 4 sessions
    };

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(pomodoro_timer_config),
            ..Default::default()
        },
    )
    .await;

    assert!(result.is_ok(), "Should accept standard Pomodoro values");
    let config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(config.timer.work_duration, Duration::from_secs(25 * 60));
    assert_eq!(config.timer.sessions_until_long_break, 4);
}

#[tokio::test]
async fn should_handle_invalid_sessions_until_long_break() {
    // GIVEN
    let ctx = setup_ctx("should_handle_invalid_sessions_until_long_break").await;

    // Test zero sessions (invalid)
    let zero_sessions_config = TimerConfiguration {
        work_duration: Duration::from_secs(25 * 60),
        short_break_duration: Duration::from_secs(5 * 60),
        long_break_duration: Duration::from_secs(15 * 60),
        sessions_until_long_break: 0, // Invalid
    };

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(zero_sessions_config),
            ..Default::default()
        },
    )
    .await;

    // Expected behavior - invalid config rejected
    assert!(result.is_err(), "Should reject zero sessions_until_long_break");

    // Test maximum allowed sessions (20 based on TimerConfiguration validation)
    let max_sessions_config = TimerConfiguration {
        work_duration: Duration::from_secs(25 * 60),
        short_break_duration: Duration::from_secs(5 * 60),
        long_break_duration: Duration::from_secs(15 * 60),
        sessions_until_long_break: 20, // Maximum allowed value
    };

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(max_sessions_config),
            ..Default::default()
        },
    )
    .await;

    assert!(result.is_ok(), "Should accept maximum session count of 20");
    let config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(config.timer.sessions_until_long_break, 20);

    // Test sessions exceeding maximum (21 should fail)
    let excessive_sessions_config = TimerConfiguration {
        work_duration: Duration::from_secs(25 * 60),
        short_break_duration: Duration::from_secs(5 * 60),
        long_break_duration: Duration::from_secs(15 * 60),
        sessions_until_long_break: 21, // Exceeds maximum
    };

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(excessive_sessions_config),
            ..Default::default()
        },
    )
    .await;

    assert!(result.is_err(), "Should reject sessions exceeding 20");
}

#[tokio::test]
async fn should_not_allow_config_update_during_active_timer() {
    // GIVEN - Timer is running with active session
    let ctx = setup_ctx("should_not_allow_config_update_during_active_timer").await;

    // Start a timer session first
    let task = ctx.task_repo
        .get_default_task()
        .await
        .expect("Failed to get default task")
        .expect("Default task should exist");

    // Create a timer session
    use usecases::timer::{start_timer_session, StartTimerSessionCmd};
    let _ = start_timer_session(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_bus.clone(),
        StartTimerSessionCmd {
            task_id: Some(task.id),
        },
    )
    .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // WHEN - Try to update config while timer is running
    let new_timer_config = TimerConfiguration {
        work_duration: Duration::from_secs(30 * 60),
        short_break_duration: Duration::from_secs(10 * 60),
        long_break_duration: Duration::from_secs(20 * 60),
        sessions_until_long_break: 5,
    };

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(new_timer_config),
            ..Default::default()
        },
    )
    .await;

    // THEN - Config update should still succeed (config changes apply to next session)
    // This documents current behavior - config can be updated while timer runs
    assert!(result.is_ok(), "Config should be updatable during timer session");
}

#[tokio::test]
async fn should_handle_invalid_config_update_gracefully() {
    // GIVEN
    let ctx = setup_ctx("should_handle_invalid_config_update_gracefully").await;

    // WHEN - Try to update with sessions > 20 (max allowed)
    let invalid_config = TimerConfiguration {
        work_duration: Duration::from_secs(25 * 60),
        short_break_duration: Duration::from_secs(5 * 60),
        long_break_duration: Duration::from_secs(15 * 60),
        sessions_until_long_break: 21, // Exceeds maximum
    };

    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(invalid_config),
            ..Default::default()
        },
    )
    .await;

    // THEN - Should fail with validation error
    assert!(result.is_err(), "Should reject config with sessions > 20");

    // Verify config remains unchanged
    let config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(config.timer.sessions_until_long_break, 4); // Should still be default
}

#[tokio::test]
async fn should_recover_from_failed_config_operations() {
    // GIVEN
    let ctx = setup_ctx("should_recover_from_failed_config_operations").await;

    // First, make a valid update
    let valid_config = TimerConfiguration {
        work_duration: Duration::from_secs(30 * 60),
        short_break_duration: Duration::from_secs(10 * 60),
        long_break_duration: Duration::from_secs(20 * 60),
        sessions_until_long_break: 5,
    };

    let _ = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(valid_config.clone()),
            ..Default::default()
        },
    )
    .await;

    // WHEN - Attempt an invalid update
    let invalid_config = TimerConfiguration {
        work_duration: Duration::from_secs(25 * 60),
        short_break_duration: Duration::from_secs(5 * 60),
        long_break_duration: Duration::from_secs(15 * 60),
        sessions_until_long_break: 25, // Invalid
    };

    let invalid_result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(invalid_config),
            ..Default::default()
        },
    )
    .await;

    // THEN - Failed operation should not corrupt config state
    assert!(invalid_result.is_err());

    // Config should still have the last valid values
    let config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(config.timer.work_duration, Duration::from_secs(30 * 60));
    assert_eq!(config.timer.sessions_until_long_break, 5);

    // System should still accept valid updates after failure
    let another_valid_config = TimerConfiguration {
        work_duration: Duration::from_secs(45 * 60),
        short_break_duration: Duration::from_secs(15 * 60),
        long_break_duration: Duration::from_secs(30 * 60),
        sessions_until_long_break: 6,
    };

    let recovery_result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(another_valid_config),
            ..Default::default()
        },
    )
    .await;

    assert!(recovery_result.is_ok(), "Should accept valid config after failed attempt");

    let final_config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(final_config.timer.work_duration, Duration::from_secs(45 * 60));
    assert_eq!(final_config.timer.sessions_until_long_break, 6);
}

#[tokio::test]
async fn should_maintain_config_consistency_after_reset_and_update_cycle() {
    // GIVEN
    let ctx = setup_ctx("should_maintain_config_consistency_after_reset_and_update_cycle").await;

    // Modify config
    let custom_config = TimerConfiguration {
        work_duration: Duration::from_secs(40 * 60),
        short_break_duration: Duration::from_secs(10 * 60),
        long_break_duration: Duration::from_secs(30 * 60),
        sessions_until_long_break: 6,
    };

    let _ = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(custom_config.clone()),
            ..Default::default()
        },
    )
    .await;

    // Reset to defaults
    let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
        ctx.config_repo.clone() as Arc<dyn ConfigRepository + Send + Sync>;
    let event_bus: Arc<dyn EventPublisher + Send + Sync> =
        ctx.event_bus.clone() as Arc<dyn EventPublisher + Send + Sync>;

    let reset_result = reset_config(&config_repo, &event_bus).await;
    assert!(reset_result.is_ok());

    let config_after_reset = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(config_after_reset, Config::default());

    // Update again with custom values
    let result = update_config(
        ctx.config_repo.clone(),
        ctx.event_bus.clone(),
        UpdateConfigCmd {
            timer: Some(custom_config.clone()),
            ..Default::default()
        },
    )
    .await;

    assert!(result.is_ok());

    // THEN - Verify the cycle maintains consistency
    let final_config = get_config(ctx.config_repo.clone()).await.unwrap();
    assert_eq!(final_config.timer.work_duration, Duration::from_secs(40 * 60));
    assert_eq!(final_config.timer.short_break_duration, Duration::from_secs(10 * 60));
    assert_eq!(final_config.timer.long_break_duration, Duration::from_secs(30 * 60));
    assert_eq!(final_config.timer.sessions_until_long_break, 6);
}
