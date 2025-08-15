//! Audio Domain Infrastructure
//!
//! Contains all audio-related infrastructure implementations:
//! - Audio service implementation
//! - Audio asset provider for loading sound files

pub mod audio_srv;
pub mod asset_provider;
pub mod library_service;

pub use audio_srv::RodioAudioService;
pub use asset_provider::{DefaultAudioAssetProvider, BG_SOUNDS};
pub use library_service::InMemoryAudioLibraryService;