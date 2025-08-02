#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Audio asset not found: {0}")]
    AssetNotFound(String),
    #[error("Playback failed: {0}")]
    PlaybackFailed(String),
    #[error("Invalid audio file: {0}")]
    InvalidFile(String),
    #[error("Volume out of range: {0} (must be 0.0-1.0)")]
    VolumeOutOfRange(f32),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}