use domain::{AudioAsset, AudioCategory, AudioLibrary};
use std::path::PathBuf;

pub struct DefaultAudioAssetProvider;

pub const BG_SOUNDS: &[(&str, &str)] = &[
    ("rain", "Rain"),
    ("forest", "Forest Ambience"),
    ("ocean", "Ocean Waves"),
    ("white-noise", "White Noise"),
    ("brown-noise", "Brown Noise"),
    ("cafe", "Café Ambience"),
    ("fireplace", "Fireplace Crackling"),
    ("thunderstorm", "Thunderstorm"),
];

impl DefaultAudioAssetProvider {
    pub fn create_library_with_default_assets() -> AudioLibrary {
        let mut library = AudioLibrary::new();
        Self::add_default_notification_sounds(&mut library);
        Self::add_default_background_sounds(&mut library);
        library
    }

    fn add_default_notification_sounds(library: &mut AudioLibrary) {
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
                file_path: PathBuf::from(format!(
                    "assets/sounds/notifications/{id}.mp3"
                )),
                category: AudioCategory::NotificationSound,
                duration_ms: None,
            };
            library.add_asset(asset);
        }
    }

    fn add_default_background_sounds(library: &mut AudioLibrary) {
        for (id, name) in BG_SOUNDS {
            let asset = AudioAsset {
                id: id.to_string(),
                name: name.to_string(),
                file_path: PathBuf::from(format!(
                    "assets/sounds/background/{id}.mp3"
                )),
                category: AudioCategory::BackgroundAmbient,
                duration_ms: None,
            };
            library.add_asset(asset);
        }
    }
}
