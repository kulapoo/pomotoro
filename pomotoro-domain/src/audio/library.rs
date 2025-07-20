use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::audio_asset::AudioAsset;
use super::error::AudioError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLibrary {
    pub assets: HashMap<String, AudioAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackRequest {
    pub asset_id: String,
    pub volume: f32,
    pub looped: bool,
    pub fade_in_ms: Option<u32>,
    pub fade_out_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackHandle {
    pub id: String,
    pub asset_id: String,
    pub is_playing: bool,
    pub is_looped: bool,
    pub volume: f32,
}

impl AudioLibrary {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn add_asset(&mut self, asset: AudioAsset) {
        self.assets.insert(asset.id.clone(), asset);
    }

    pub fn get_asset(&self, id: &str) -> Option<&AudioAsset> {
        self.assets.get(id)
    }

    pub fn remove_asset(&mut self, id: &str) -> Option<AudioAsset> {
        self.assets.remove(id)
    }
}

impl PlaybackRequest {
    pub fn new(asset_id: String, volume: f32) -> Result<Self, AudioError> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(AudioError::VolumeOutOfRange(volume));
        }

        Ok(Self {
            asset_id,
            volume,
            looped: false,
            fade_in_ms: None,
            fade_out_ms: None,
        })
    }

    pub fn with_loop(mut self) -> Self {
        self.looped = true;
        self
    }

    pub fn with_fade_in(mut self, fade_in_ms: u32) -> Self {
        self.fade_in_ms = Some(fade_in_ms);
        self
    }
}