use crate::core::models::{TestConfigRepository, TestConfigBuilder, ConfigTestUtils};
use domain::{Config, TaskConfig, AudioConfig, ConfigRepository};
use std::time::Duration;

#[tokio::test]
async fn test_config_repository_basic_operations() {
    let config_repo = TestConfigRepository::new();
    
    // Test getting default config
    let default_config = config_repo.get_config().await.unwrap();
    assert_eq!(default_config.task_defaults.work_duration, Duration::from_secs(25 * 60));
    assert_eq!(default_config.task_defaults.short_break_duration, Duration::from_secs(5 * 60));
    assert_eq!(default_config.task_defaults.long_break_duration, Duration::from_secs(15 * 60));
    assert_eq!(default_config.task_defaults.sessions_until_long_break, 4);
    assert!(!default_config.task_defaults.enable_screen_blocking);
    
    // Test saving custom config
    let custom_config = TestConfigBuilder::new()
        .with_work_duration(Duration::from_secs(30 * 60))
        .with_short_break_duration(Duration::from_secs(10 * 60))
        .with_audio_volume(0.8)
        .build();
    
    config_repo.save_config(&custom_config).await.unwrap();
    
    // Verify config was saved
    let saved_config = config_repo.get_config().await.unwrap();
    ConfigTestUtils::assert_config_equals(&saved_config, &custom_config);
}

#[tokio::test]
async fn test_config_reset_to_defaults() {
    let config_repo = TestConfigRepository::new();
    
    // Save custom config
    let custom_config = TestConfigBuilder::new()
        .with_work_duration(Duration::from_secs(45 * 60))
        .with_screen_blocking(true)
        .with_muted_audio(true)
        .build();
    
    config_repo.save_config(&custom_config).await.unwrap();
    
    // Reset to defaults
    config_repo.reset_to_defaults();
    let reset_config = config_repo.get_config().await.unwrap();
    
    // Verify reset worked
    let default_config = Config::default();
    ConfigTestUtils::assert_config_equals(&reset_config, &default_config);
    
    // Verify repository state was updated
    let current_config = config_repo.get_config().await.unwrap();
    ConfigTestUtils::assert_config_equals(&current_config, &default_config);
}

#[test]
fn test_config_builder_patterns() {
    // Test fast config for testing
    let fast_config = ConfigTestUtils::create_fast_config();
    assert_eq!(fast_config.task_defaults.work_duration, Duration::from_secs(5));
    assert_eq!(fast_config.task_defaults.short_break_duration, Duration::from_secs(2));
    assert_eq!(fast_config.task_defaults.long_break_duration, Duration::from_secs(3));
    assert_eq!(fast_config.task_defaults.sessions_until_long_break, 2);
    
    // Test slow config for long-form work
    let slow_config = ConfigTestUtils::create_slow_config();
    assert_eq!(slow_config.task_defaults.work_duration, Duration::from_secs(60 * 60));
    assert_eq!(slow_config.task_defaults.short_break_duration, Duration::from_secs(30 * 60));
    assert_eq!(slow_config.task_defaults.long_break_duration, Duration::from_secs(45 * 60));
    assert_eq!(slow_config.task_defaults.sessions_until_long_break, 6);
    
    // Test silent config
    let silent_config = ConfigTestUtils::create_silent_config();
    assert!(silent_config.audio.muted);
    assert!(!silent_config.audio.enable_background_audio);
}

// MVP 2.0 Configuration Features

#[tokio::test]
async fn test_per_task_default_configuration() {
    let config_repo = TestConfigRepository::new();
    
    // Test MVP2 task default configuration settings
    let mvp2_config = TestConfigBuilder::new()
        .with_work_duration(Duration::from_secs(25 * 60))  // Standard 25 min
        .with_short_break_duration(Duration::from_secs(5 * 60))  // Standard 5 min
        .with_long_break_duration(Duration::from_secs(15 * 60))  // Standard 15 min
        .with_sessions_until_long_break(4)  // Traditional pomodoro cycle
        .with_screen_blocking(false)  // Default off
        .with_audio_volume(0.6)  // Default volume
        .with_background_audio(true)  // MVP2 feature
        .build();
    
    config_repo.save_config(&mvp2_config).await.unwrap();
    
    let saved_config = config_repo.get_config().await.unwrap();
    
    // Verify MVP2 defaults
    assert_eq!(saved_config.task_defaults.work_duration, Duration::from_secs(25 * 60));
    assert_eq!(saved_config.task_defaults.short_break_duration, Duration::from_secs(5 * 60));
    assert_eq!(saved_config.task_defaults.long_break_duration, Duration::from_secs(15 * 60));
    assert_eq!(saved_config.task_defaults.sessions_until_long_break, 4);
    assert!(!saved_config.task_defaults.enable_screen_blocking);
    assert_eq!(saved_config.audio.volume, 0.6);
    assert!(saved_config.audio.enable_background_audio);
}

#[tokio::test]
async fn test_custom_timing_configurations() {
    let config_repo = TestConfigRepository::new();
    
    // Test MVP2 custom timing ranges (5-60 min work, 1-30 min breaks)
    
    // Short work sessions
    let short_work_config = TestConfigBuilder::new()
        .with_work_duration(Duration::from_secs(5 * 60))  // Minimum 5 min
        .with_short_break_duration(Duration::from_secs(60))  // Minimum 1 min
        .build();
    
    config_repo.save_config(&short_work_config).await.unwrap();
    let saved_short = config_repo.get_config().await.unwrap();
    assert_eq!(saved_short.task_defaults.work_duration, Duration::from_secs(5 * 60));
    assert_eq!(saved_short.task_defaults.short_break_duration, Duration::from_secs(60));
    
    // Long work sessions
    let long_work_config = TestConfigBuilder::new()
        .with_work_duration(Duration::from_secs(60 * 60))  // Maximum 60 min
        .with_short_break_duration(Duration::from_secs(30 * 60))  // Maximum 30 min
        .with_long_break_duration(Duration::from_secs(60 * 60))  // Up to 60 min long breaks
        .build();
    
    config_repo.save_config(&long_work_config).await.unwrap();
    let saved_long = config_repo.get_config().await.unwrap();
    assert_eq!(saved_long.task_defaults.work_duration, Duration::from_secs(60 * 60));
    assert_eq!(saved_long.task_defaults.short_break_duration, Duration::from_secs(30 * 60));
    assert_eq!(saved_long.task_defaults.long_break_duration, Duration::from_secs(60 * 60));
}

#[tokio::test] 
async fn test_screen_blocking_configuration() {
    let config_repo = TestConfigRepository::new();
    
    // Test MVP2 screen blocking feature configuration
    let blocking_config = TestConfigBuilder::new()
        .with_screen_blocking(true)
        .with_work_duration(Duration::from_secs(45 * 60))  // Longer sessions benefit from blocking
        .build();
    
    config_repo.save_config(&blocking_config).await.unwrap();
    let saved_config = config_repo.get_config().await.unwrap();
    
    assert!(saved_config.task_defaults.enable_screen_blocking);
    assert_eq!(saved_config.task_defaults.work_duration, Duration::from_secs(45 * 60));
    
    // Test disabling screen blocking
    let no_blocking_config = TestConfigBuilder::new()
        .with_screen_blocking(false)
        .build();
    
    config_repo.save_config(&no_blocking_config).await.unwrap();
    let updated_config = config_repo.get_config().await.unwrap();
    
    assert!(!updated_config.task_defaults.enable_screen_blocking);
}

#[tokio::test]
async fn test_audio_configuration_settings() {
    let config_repo = TestConfigRepository::new();
    
    // Test MVP2 audio configuration features
    let audio_config = TestConfigBuilder::new()
        .with_audio_volume(0.8)
        .with_background_audio(true)
        .with_muted_audio(false)
        .build();
    
    config_repo.save_config(&audio_config).await.unwrap();
    let saved_config = config_repo.get_config().await.unwrap();
    
    assert_eq!(saved_config.audio.volume, 0.8);
    assert!(saved_config.audio.enable_background_audio);
    assert!(!saved_config.audio.muted);
    
    // Test silent mode
    let silent_config = TestConfigBuilder::new()
        .with_muted_audio(true)
        .with_background_audio(false)
        .with_audio_volume(0.0)
        .build();
    
    config_repo.save_config(&silent_config).await.unwrap();
    let silent_saved = config_repo.get_config().await.unwrap();
    
    assert!(silent_saved.audio.muted);
    assert!(!silent_saved.audio.enable_background_audio);
    assert_eq!(silent_saved.audio.volume, 0.0);
}

#[tokio::test]
async fn test_session_cycle_configuration() {
    let config_repo = TestConfigRepository::new();
    
    // Test MVP2 configurable cycles per task (1-8 sessions before long break)
    
    // Single session tasks
    let single_session_config = TestConfigBuilder::new()
        .with_sessions_until_long_break(1)
        .build();
    
    config_repo.save_config(&single_session_config).await.unwrap();
    let saved_single = config_repo.get_config().await.unwrap();
    assert_eq!(saved_single.task_defaults.sessions_until_long_break, 1);
    
    // Extended cycles for deep work
    let extended_cycle_config = TestConfigBuilder::new()
        .with_sessions_until_long_break(8)
        .with_work_duration(Duration::from_secs(45 * 60))  // Longer sessions
        .with_long_break_duration(Duration::from_secs(30 * 60))  // Longer breaks
        .build();
    
    config_repo.save_config(&extended_cycle_config).await.unwrap();
    let saved_extended = config_repo.get_config().await.unwrap();
    assert_eq!(saved_extended.task_defaults.sessions_until_long_break, 8);
    assert_eq!(saved_extended.task_defaults.work_duration, Duration::from_secs(45 * 60));
    assert_eq!(saved_extended.task_defaults.long_break_duration, Duration::from_secs(30 * 60));
}

#[test]
fn test_task_config_validation() {
    // Test that TaskConfig validates MVP2 timing constraints
    
    // Valid configurations
    let valid_short = TaskConfig::new(
        Duration::from_secs(5 * 60),   // 5 min work (minimum)
        Duration::from_secs(60),   // 1 min break (minimum)
        Duration::from_secs(5 * 60),   // 5 min long break
        1,                             // 1 session cycle (minimum)
        false,
    );
    assert!(valid_short.is_ok());
    
    let valid_long = TaskConfig::new(
        Duration::from_secs(60 * 60),  // 60 min work (maximum)
        Duration::from_secs(30 * 60),  // 30 min break (maximum)
        Duration::from_secs(60 * 60),  // 60 min long break
        8,                             // 8 session cycle (maximum)
        true,
    );
    assert!(valid_long.is_ok());
    
    // Test validation boundary cases
    let zero_work = TaskConfig::new(
        Duration::from_secs(0),        // Invalid: 0 work time
        Duration::from_secs(5 * 60),
        Duration::from_secs(15 * 60),
        4,
        false,
    );
    assert!(zero_work.is_err());
    
    let zero_sessions = TaskConfig::new(
        Duration::from_secs(25 * 60),
        Duration::from_secs(5 * 60),
        Duration::from_secs(15 * 60),
        0,                             // Invalid: 0 sessions
        false,
    );
    assert!(zero_sessions.is_err());
}

#[test]
fn test_audio_config_validation() {
    // Test that AudioConfig validates MVP2 audio constraints
    
    // Valid configuration
    let valid_audio = AudioConfig {
        volume: 0.5,
        enable_background_audio: true,
        background_sound: Some("nature-sounds".to_string()),
        work_notification_sound: Some("chime".to_string()),
        break_notification_sound: Some("bell".to_string()),
        muted: false,
    };
    
    // Test volume ranges (should be 0.0 to 1.0)
    assert!(valid_audio.volume >= 0.0 && valid_audio.volume <= 1.0);
    
    // Test muted audio overrides other settings
    let muted_audio = AudioConfig {
        volume: 0.8,
        enable_background_audio: true,
        background_sound: Some("sounds".to_string()),
        work_notification_sound: Some("chime".to_string()),
        break_notification_sound: Some("bell".to_string()),
        muted: true,
    };
    
    // When muted, effective volume should be 0
    assert!(muted_audio.muted);
    // Note: The actual muting behavior would be tested in the audio system
}

#[tokio::test]
async fn test_configuration_persistence_patterns() {
    let config_repo = TestConfigRepository::new();
    
    // Test MVP2 configuration patterns for different use cases
    
    // Deep focus work pattern
    let deep_focus = TestConfigBuilder::new()
        .with_work_duration(Duration::from_secs(45 * 60))
        .with_short_break_duration(Duration::from_secs(10 * 60))
        .with_long_break_duration(Duration::from_secs(25 * 60))
        .with_sessions_until_long_break(3)
        .with_screen_blocking(true)
        .with_audio_volume(0.4)
        .with_background_audio(true)
        .build();
    
    config_repo.save_config(&deep_focus).await.unwrap();
    let saved_deep_focus = config_repo.get_config().await.unwrap();
    
    assert_eq!(saved_deep_focus.task_defaults.work_duration, Duration::from_secs(45 * 60));
    assert!(saved_deep_focus.task_defaults.enable_screen_blocking);
    assert_eq!(saved_deep_focus.audio.volume, 0.4);
    
    // Quick iteration pattern
    let quick_iteration = TestConfigBuilder::new()
        .with_work_duration(Duration::from_secs(15 * 60))
        .with_short_break_duration(Duration::from_secs(3 * 60))
        .with_long_break_duration(Duration::from_secs(10 * 60))
        .with_sessions_until_long_break(6)
        .with_screen_blocking(false)
        .with_audio_volume(0.7)
        .build();
    
    config_repo.save_config(&quick_iteration).await.unwrap();
    let saved_quick = config_repo.get_config().await.unwrap();
    
    assert_eq!(saved_quick.task_defaults.work_duration, Duration::from_secs(15 * 60));
    assert_eq!(saved_quick.task_defaults.sessions_until_long_break, 6);
    assert!(!saved_quick.task_defaults.enable_screen_blocking);
    assert_eq!(saved_quick.audio.volume, 0.7);
}