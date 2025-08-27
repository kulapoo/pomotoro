//! Audio Domain Infrastructure
//!
//! Contains all audio-related infrastructure implementations:
//! - Audio service implementation
//! - Audio asset provider for loading sound files

pub mod asset_provider;
pub mod audio_service_wrapper;
pub mod audio_srv;
pub mod event_handlers;
pub mod library_service;

pub use asset_provider::{BG_SOUNDS, DefaultAudioAssetProvider};
pub use audio_service_wrapper::AudioServiceWrapper;
pub use audio_srv::RodioAudioService;
pub use event_handlers::register_audio_event_handlers;
pub use library_service::InMemoryAudioLibraryService;
