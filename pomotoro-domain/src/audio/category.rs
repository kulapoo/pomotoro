use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioCategory {
    NotificationSound,
    BackgroundAmbient,
    CustomUpload,
}