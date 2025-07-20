use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::audio_category::AudioCategory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAsset {
    pub id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub category: AudioCategory,
    pub duration_ms: Option<u64>,
}