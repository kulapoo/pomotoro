use std::sync::Arc;
use tauri::AppHandle;

use super::{CompositeEventPublisher, DomainEventBus, TauriEventPublisher};

pub type EventPublisherArc = Arc<dyn domain::EventPublisher + Send + Sync>;

/// Create an event publisher that combines internal handlers and frontend emission
pub fn create_composite_event_publisher(app_handle: AppHandle) -> EventPublisherArc {
    let mut composite = CompositeEventPublisher::new();
    
    // Add internal event bus for application-level handlers
    composite.add_publisher(Arc::new(DomainEventBus::new()));
    
    // Add Tauri publisher for frontend emission
    composite.add_publisher(Arc::new(TauriEventPublisher::new(app_handle)));
    
    Arc::new(composite)
}

/// Create an event publisher and return both the composite and the domain event bus
/// for handler registration
pub fn create_event_publisher_with_bus(app_handle: AppHandle) -> (EventPublisherArc, Arc<DomainEventBus>) {
    let mut composite = CompositeEventPublisher::new();
    
    // Create the domain event bus that we'll register handlers on
    let domain_bus = Arc::new(DomainEventBus::new());
    
    // Add internal event bus for application-level handlers
    composite.add_publisher(domain_bus.clone());
    
    // Add Tauri publisher for frontend emission
    composite.add_publisher(Arc::new(TauriEventPublisher::new(app_handle)));
    
    (Arc::new(composite), domain_bus)
}

// Infrastructure-specific event constants
// These are technical/system integration events, not domain business logic

pub mod audio {
    pub const GET_LIBRARY: &str = "get_audio_library";
    pub const PLAY: &str = "play_audio";
    pub const STOP: &str = "stop_audio";
    pub const PAUSE: &str = "pause_audio";
    pub const RESUME: &str = "resume_audio";
    pub const SET_VOLUME: &str = "set_audio_volume";
    pub const GET_ACTIVE: &str = "get_active_playbacks";
    pub const STOP_ALL: &str = "stop_all_audio";
    pub const PLAY_NOTIFICATION: &str = "play_notification_sound";
    pub const PLAY_BACKGROUND: &str = "play_background_audio";
    pub const STOP_BACKGROUND: &str = "stop_background_audio";
    pub const ADD_ASSET: &str = "add_custom_audio_asset";
    pub const REMOVE_ASSET: &str = "remove_audio_asset";
    pub const CLEANUP: &str = "cleanup_finished_audio";
}