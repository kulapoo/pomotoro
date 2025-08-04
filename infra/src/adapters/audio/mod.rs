//! Audio Domain Infrastructure
//!
//! Contains all audio-related infrastructure implementations:
//! - Audio service implementation
//! - Audio asset provider for loading sound files

pub mod service;
pub mod asset_provider;

pub use service::RodioAudioService;
pub use asset_provider::{DefaultAudioAssetProvider, BG_SOUNDS};