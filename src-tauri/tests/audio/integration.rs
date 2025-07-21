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
    assert_eq!(audio_manager.get_asset_playback_count("chime"), 1);
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