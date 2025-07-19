use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::core::entities::{AudioAsset, AudioCategory};
use crate::core::utilities::AudioError;

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

    pub fn with_default_assets() -> Self {
        let mut library = Self::new();
        library.add_default_notification_sounds();
        library.add_default_background_sounds();
        library
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


    fn add_default_notification_sounds(&mut self) {
        let default_sounds = vec![
            ("bell", "Bell"),
            ("chime", "Chime"),
            ("ding", "Ding"),
            ("gentle-bell", "Gentle Bell"),
            ("wooden-block", "Wooden Block"),
        ];

        for (id, name) in default_sounds {
            let asset = AudioAsset {
                id: id.to_string(),
                name: name.to_string(),
                file_path: PathBuf::from(format!("assets/sounds/notifications/{}.mp3", id)),
                category: AudioCategory::NotificationSound,
                duration_ms: None,
            };
            self.add_asset(asset);
        }
    }

    fn add_default_background_sounds(&mut self) {
        let background_sounds = vec![
            ("rain", "Rain"),
            ("forest", "Forest Ambience"),
            ("ocean", "Ocean Waves"),
            ("white-noise", "White Noise"),
            ("brown-noise", "Brown Noise"),
            ("cafe", "Café Ambience"),
            ("fireplace", "Fireplace Crackling"),
            ("thunderstorm", "Thunderstorm"),
        ];

        for (id, name) in background_sounds {
            let asset = AudioAsset {
                id: id.to_string(),
                name: name.to_string(),
                file_path: PathBuf::from(format!("assets/sounds/background/{}.mp3", id)),
                category: AudioCategory::BackgroundAmbient,
                duration_ms: None,
            };
            self.add_asset(asset);
        }
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