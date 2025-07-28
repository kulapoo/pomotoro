use crate::audio::models::{MockAudioManager, AudioConfigBuilder, AudioTestAssertions};
use pomotoro_domain::PlaybackRequest;

#[tokio::test]
async fn test_audio_manager_initialization() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    assert_eq!(audio_manager.get_playback_count(), 0);
    assert_eq!(audio_manager.get_playing_count(), 0);
    assert_eq!(audio_manager.get_paused_count(), 0);
    
    let library = audio_manager.get_library();
    assert!(!library.assets.is_empty());
}

#[tokio::test]
async fn test_audio_playback_lifecycle() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    let request = PlaybackRequest::new("test-sound".to_string(), 0.5).unwrap();
    let handle = audio_manager.play(request).unwrap();
    
    assert_eq!(audio_manager.get_playing_count(), 1);
    assert!(audio_manager.is_playing(&handle.id));
    assert!(!audio_manager.is_paused(&handle.id));
    
    // Pause playback
    audio_manager.pause_playback(&handle.id).unwrap();
    assert_eq!(audio_manager.get_playing_count(), 0);
    assert_eq!(audio_manager.get_paused_count(), 1);
    assert!(!audio_manager.is_playing(&handle.id));
    assert!(audio_manager.is_paused(&handle.id));
    
    // Resume playback
    audio_manager.resume_playback(&handle.id).unwrap();
    assert_eq!(audio_manager.get_playing_count(), 1);
    assert_eq!(audio_manager.get_paused_count(), 0);
    assert!(audio_manager.is_playing(&handle.id));
    assert!(!audio_manager.is_paused(&handle.id));
    
    // Stop playback
    audio_manager.stop_playback(&handle.id).unwrap();
    assert_eq!(audio_manager.get_playing_count(), 0);
    assert_eq!(audio_manager.get_paused_count(), 0);
    assert!(!audio_manager.is_playing(&handle.id));
    assert!(!audio_manager.is_paused(&handle.id));
}

#[tokio::test]
async fn test_audio_volume_control() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    let request = PlaybackRequest::new("test-sound".to_string(), 0.5).unwrap();
    let handle = audio_manager.play(request).unwrap();
    
    assert_eq!(audio_manager.get_playback_volume(&handle.id), Some(0.5));
    
    audio_manager.set_volume(&handle.id, 0.8).unwrap();
    assert_eq!(audio_manager.get_playback_volume(&handle.id), Some(0.8));
    
    audio_manager.set_volume(&handle.id, 0.0).unwrap();
    assert_eq!(audio_manager.get_playback_volume(&handle.id), Some(0.0));
}

#[tokio::test]
async fn test_audio_multiple_playbacks() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    let request1 = PlaybackRequest::new("sound1".to_string(), 0.5).unwrap();
    let request2 = PlaybackRequest::new("sound2".to_string(), 0.7).unwrap();
    let request3 = PlaybackRequest::new("sound3".to_string(), 0.3).unwrap();
    
    let handle1 = audio_manager.play(request1).unwrap();
    let handle2 = audio_manager.play(request2).unwrap();
    let _handle3 = audio_manager.play(request3).unwrap();
    
    assert_eq!(audio_manager.get_playing_count(), 3);
    assert_eq!(audio_manager.get_playback_count(), 3);
    
    audio_manager.pause_playback(&handle2.id).unwrap();
    assert_eq!(audio_manager.get_playing_count(), 2);
    assert_eq!(audio_manager.get_paused_count(), 1);
    
    audio_manager.stop_playback(&handle1.id).unwrap();
    assert_eq!(audio_manager.get_playing_count(), 1);
    assert_eq!(audio_manager.get_paused_count(), 1);
    
    audio_manager.stop_all_playbacks();
    assert_eq!(audio_manager.get_playing_count(), 0);
    assert_eq!(audio_manager.get_paused_count(), 0);
}

#[tokio::test]
async fn test_audio_background_playback() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    let handle = audio_manager.play_background_audio("nature-sounds", 0.6).unwrap();
    
    assert_eq!(audio_manager.get_playing_count(), 1);
    assert!(audio_manager.is_playing(&handle.id));
    assert_eq!(audio_manager.get_playback_volume(&handle.id), Some(0.6));
    
    audio_manager.stop_background_audio();
    assert_eq!(audio_manager.get_playing_count(), 0);
    assert!(!audio_manager.is_playing(&handle.id));
}

#[tokio::test]
async fn test_audio_notification_playback() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    let handle = audio_manager.play_notification("chime", 0.8).unwrap();
    
    assert_eq!(audio_manager.get_playing_count(), 1);
    assert!(audio_manager.is_playing(&handle.id));
    assert_eq!(audio_manager.get_playback_volume(&handle.id), Some(0.8));
    
    audio_manager.stop_playback(&handle.id).unwrap();
    assert!(!audio_manager.is_playing(&handle.id));
}

#[tokio::test]
async fn test_audio_asset_tracking() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    let handle1 = audio_manager.play_notification("chime", 0.5).unwrap();
    let _handle2 = audio_manager.play_notification("chime", 0.7).unwrap();
    let _handle3 = audio_manager.play_notification("bell", 0.6).unwrap();
    
    assert_eq!(audio_manager.get_asset_playback_count("chime"), 2);
    assert_eq!(audio_manager.get_asset_playback_count("bell"), 1);
    assert_eq!(audio_manager.get_asset_playback_count("nonexistent"), 0);
    
    audio_manager.stop_playback(&handle1.id).unwrap();
    assert_eq!(audio_manager.get_asset_playback_count("chime"), 2); // Historical count doesn't decrease
    assert_eq!(audio_manager.get_asset_playback_count("bell"), 1);
}

#[tokio::test]
async fn test_audio_cleanup() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    let handle1 = audio_manager.play_notification("sound1", 0.5).unwrap();
    let handle2 = audio_manager.play_notification("sound2", 0.5).unwrap();
    
    assert_eq!(audio_manager.get_playback_count(), 2);
    
    audio_manager.stop_playback(&handle1.id).unwrap();
    audio_manager.cleanup_finished_playbacks();
    
    // After cleanup, only active playbacks should remain
    assert_eq!(audio_manager.get_playback_count(), 1);
    assert!(audio_manager.is_playing(&handle2.id));
}

#[test]
fn test_audio_config_builder() {
    let config = AudioConfigBuilder::new()
        .with_volume(0.8)
        .with_background_audio_enabled(true)
        .with_background_sound(Some("nature-sounds".to_string()))
        .with_work_notification_sound(Some("work-chime".to_string()))
        .with_break_notification_sound(Some("break-bell".to_string()))
        .build();

    AudioTestAssertions::assert_volume_level(&config, 0.8);
    AudioTestAssertions::assert_has_background_audio(&config);
    AudioTestAssertions::assert_has_notification_sounds(&config);
    AudioTestAssertions::assert_is_unmuted(&config);
}

#[test]
fn test_audio_config_silent() {
    let config = AudioConfigBuilder::new()
        .silent()
        .build();

    AudioTestAssertions::assert_is_muted(&config);
    AudioTestAssertions::assert_has_no_background_audio(&config);
    AudioTestAssertions::assert_has_no_notification_sounds(&config);
    AudioTestAssertions::assert_volume_level(&config, 0.0);
}

#[test]
fn test_audio_config_custom_sounds() {
    let config = AudioConfigBuilder::new()
        .with_custom_sounds()
        .build();

    AudioTestAssertions::assert_has_notification_sounds(&config);
    AudioTestAssertions::assert_has_background_audio(&config);
    AudioTestAssertions::assert_volume_level(&config, 0.6);
    AudioTestAssertions::assert_is_unmuted(&config);
}

// MVP 2.0 Per-Task Audio Configuration Features

#[tokio::test]
async fn test_task_specific_audio_volumes() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    // Test different volume levels for different task types (MVP2 feature)
    
    // Work task with higher volume
    let work_background_handle = audio_manager.play_background_audio("binaural-focus", 0.7).unwrap();
    assert_eq!(audio_manager.get_playback_volume(&work_background_handle.id), Some(0.7));
    
    let work_notification_handle = audio_manager.play_notification("subtle-chime", 0.7).unwrap();
    assert_eq!(audio_manager.get_playback_volume(&work_notification_handle.id), Some(0.7));
    
    audio_manager.stop_all_playbacks();
    
    // Study task with lower volume
    let study_background_handle = audio_manager.play_background_audio("nature-sounds", 0.4).unwrap();
    assert_eq!(audio_manager.get_playback_volume(&study_background_handle.id), Some(0.4));
    
    let study_notification_handle = audio_manager.play_notification("soft-ping", 0.4).unwrap();
    assert_eq!(audio_manager.get_playback_volume(&study_notification_handle.id), Some(0.4));
}

#[tokio::test]
async fn test_toro_audio_background_soundscapes() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    // Test different background soundscape options (MVP2 "Toro audio")
    let soundscapes = vec![
        "binaural-focus",
        "nature-sounds", 
        "white-noise",
        "brown-noise",
        "rain-sounds",
        "ocean-waves",
        "forest-ambience"
    ];
    
    for soundscape in soundscapes {
        let handle = audio_manager.play_background_audio(soundscape, 0.5).unwrap();
        
        assert!(audio_manager.is_playing(&handle.id));
        assert_eq!(audio_manager.get_playback_volume(&handle.id), Some(0.5));
        
        // Verify soundscape volume can be adjusted
        audio_manager.set_volume(&handle.id, 0.8).unwrap();
        assert_eq!(audio_manager.get_playback_volume(&handle.id), Some(0.8));
        
        audio_manager.set_volume(&handle.id, 0.0).unwrap();
        assert_eq!(audio_manager.get_playback_volume(&handle.id), Some(0.0));
        
        audio_manager.stop_playback(&handle.id).unwrap();
    }
}

#[tokio::test]
async fn test_audio_session_transitions() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    // Test simulated session transitions using existing methods
    
    // Start work session - play background audio and work notification
    let background_handle = audio_manager.play_background_audio("focus-sounds", 0.6).unwrap();
    let _work_notification = audio_manager.play_notification("work-start", 0.6).unwrap();
    
    assert!(audio_manager.is_playing(&background_handle.id));
    assert_eq!(audio_manager.get_asset_playback_count("focus-sounds"), 1);
    assert_eq!(audio_manager.get_asset_playback_count("work-start"), 1);
    
    // Transition to break - stop background and play break notification
    audio_manager.stop_background_audio();
    let _break_notification = audio_manager.play_notification("break-start", 0.6).unwrap();
    
    assert!(!audio_manager.is_playing(&background_handle.id)); // Background stopped
    assert_eq!(audio_manager.get_asset_playback_count("break-start"), 1);
    
    // Return to work - restart background audio
    let new_background_handle = audio_manager.play_background_audio("focus-sounds", 0.6).unwrap();
    assert!(audio_manager.is_playing(&new_background_handle.id));
    assert_eq!(audio_manager.get_asset_playback_count("focus-sounds"), 2); // Second time
}

#[tokio::test]
async fn test_per_task_volume_independence() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    use pomotoro_domain::AudioConfig;
    
    // Create two different audio configurations
    let loud_task_audio = AudioConfig {
        volume: 0.9,
        enable_background_audio: true,
        background_sound: Some("energetic-beats".to_string()),
        work_notification_sound: Some("loud-chime".to_string()),
        break_notification_sound: Some("loud-bell".to_string()),
        muted: false,
    };
    
    let quiet_task_audio = AudioConfig {
        volume: 0.2,
        enable_background_audio: true,
        background_sound: Some("gentle-ambient".to_string()),
        work_notification_sound: Some("soft-whisper".to_string()),
        break_notification_sound: Some("quiet-ding".to_string()),
        muted: false,
    };
    
    // Test loud configuration volume
    let loud_handle = audio_manager.play_notification("loud-chime", 0.9).unwrap();
    assert_eq!(audio_manager.get_playback_volume(&loud_handle.id), Some(0.9));
    
    audio_manager.stop_all_playbacks();
    
    // Test quiet configuration volume independence
    let quiet_handle = audio_manager.play_notification("soft-whisper", 0.2).unwrap();
    assert_eq!(audio_manager.get_playback_volume(&quiet_handle.id), Some(0.2));
}

#[tokio::test]
async fn test_audio_volume_control_per_task() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    // Test muted task behavior (volume 0)
    let muted_handle = audio_manager.play_background_audio("ambient", 0.0).unwrap();
    assert_eq!(audio_manager.get_playback_volume(&muted_handle.id), Some(0.0));
    
    audio_manager.stop_all_playbacks();
    
    // Test normal task behavior
    let normal_handle = audio_manager.play_background_audio("normal-sounds", 0.6).unwrap();
    assert_eq!(audio_manager.get_playback_volume(&normal_handle.id), Some(0.6));
    assert!(audio_manager.is_playing(&normal_handle.id));
    assert_eq!(audio_manager.get_playing_count(), 1);
}

#[tokio::test]
async fn test_audio_asset_management_per_task() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    // Test that audio assets are properly loaded for different task configurations
    let library = audio_manager.get_library();
    
    // Verify audio library has assets loaded
    assert!(!library.assets.is_empty(), "Audio library should have assets loaded");
    
    // Test loading custom soundscape
    let custom_handle = audio_manager.play_background_audio("binaural-focus", 0.5).unwrap();
    assert!(audio_manager.is_playing(&custom_handle.id));
    assert_eq!(audio_manager.get_asset_playback_count("binaural-focus"), 1);
    
    // Test loading different notification sounds
    let _chime_handle = audio_manager.play_notification("subtle-chime", 0.7).unwrap();
    let _bell_handle = audio_manager.play_notification("gentle-bell", 0.6).unwrap();
    
    assert_eq!(audio_manager.get_asset_playback_count("subtle-chime"), 1);
    assert_eq!(audio_manager.get_asset_playback_count("gentle-bell"), 1);
    assert_eq!(audio_manager.get_playing_count(), 3); // All three playing
}

#[tokio::test]
async fn test_concurrent_task_audio_switching() {
    let audio_manager = MockAudioManager::new().unwrap();
    
    // Simulate task switching by stopping current audio and starting new audio
    
    // Start audio for task1
    let task1_background = audio_manager.play_background_audio("task1-ambient", 0.5).unwrap();
    
    assert!(audio_manager.is_playing(&task1_background.id));
    assert_eq!(audio_manager.get_playback_volume(&task1_background.id), Some(0.5));
    
    // Switch to task2 - stop task1 audio and start task2 audio
    audio_manager.stop_background_audio(); // Stops task1 background
    let task2_background = audio_manager.play_background_audio("task2-ambient", 0.8).unwrap();
    
    // Task1 audio should be stopped
    assert!(!audio_manager.is_playing(&task1_background.id));
    
    // Task2 audio should be playing with its volume
    assert!(audio_manager.is_playing(&task2_background.id));
    assert_eq!(audio_manager.get_playback_volume(&task2_background.id), Some(0.8));
    
    // Total playing count should be 1 (only task2)
    assert_eq!(audio_manager.get_playing_count(), 1);
}

#[test]
fn test_audio_config_builder_mvp2_features() {
    // Test MVP2 specific audio configuration patterns
    
    // Deep focus configuration
    let deep_focus_config = AudioConfigBuilder::new()
        .with_volume(0.4)
        .with_background_audio_enabled(true)
        .with_background_sound(Some("binaural-focus".to_string()))
        .with_work_notification_sound(Some("subtle-chime".to_string()))
        .with_break_notification_sound(Some("gentle-bell".to_string()))
        .build();
    
    AudioTestAssertions::assert_volume_level(&deep_focus_config, 0.4);
    AudioTestAssertions::assert_has_background_audio(&deep_focus_config);
    assert_eq!(deep_focus_config.background_sound, Some("binaural-focus".to_string()));
    
    // Energetic work configuration
    let energetic_config = AudioConfigBuilder::new()
        .with_volume(0.8)
        .with_background_audio_enabled(true)
        .with_background_sound(Some("upbeat-instrumental".to_string()))
        .with_work_notification_sound(Some("energetic-beep".to_string()))
        .with_break_notification_sound(Some("celebration-chime".to_string()))
        .build();
    
    AudioTestAssertions::assert_volume_level(&energetic_config, 0.8);
    AudioTestAssertions::assert_has_background_audio(&energetic_config);
    assert_eq!(energetic_config.background_sound, Some("upbeat-instrumental".to_string()));
    
    // Silent study configuration
    let silent_config = AudioConfigBuilder::new()
        .silent()
        .build();
    
    AudioTestAssertions::assert_is_muted(&silent_config);
    AudioTestAssertions::assert_has_no_background_audio(&silent_config);
    AudioTestAssertions::assert_volume_level(&silent_config, 0.0);
}