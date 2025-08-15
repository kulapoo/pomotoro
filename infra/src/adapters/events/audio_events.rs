
pub mod audio {
    pub const GET_LIBRARY: &str = "get_audio_library";
    pub const PLAY: &str = "play_audio";
    pub const STOP: &str = "stop_audio";
    pub const PAUSE: &str = "pause_audio";
    pub const RESUME: &str = "resume_audio";
    pub const SET_VOLUME: &str = "set_audio_volume";
    pub const GET_ACTIVE: &str = "get_active_playbacks";
    pub const STOP_ALL: &str = "stop_all_audio";
    pub const PLAY_NOTIFICATION: &str = "play_notification_sound";
    pub const PLAY_BACKGROUND: &str = "play_background_audio";
    pub const STOP_BACKGROUND: &str = "stop_background_audio";
    pub const ADD_ASSET: &str = "add_custom_audio_asset";
    pub const REMOVE_ASSET: &str = "remove_audio_asset";
    pub const CLEANUP: &str = "cleanup_finished_audio";
}