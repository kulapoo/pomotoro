use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAsset {
    pub id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub category: AudioCategory,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioCategory {
    NotificationSound,
    BackgroundAmbient,
    CustomUpload,
}