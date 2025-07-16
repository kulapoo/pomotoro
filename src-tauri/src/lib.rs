pub mod timer;
pub mod task;
pub mod config;
pub mod audio;

use tauri::Manager;
use timer::{
    TimerService, get_timer_state, start_timer, pause_timer, reset_timer, skip_phase,
    get_timer_state_with_task, switch_active_task
};
use task::{
    InMemoryTaskRepository, TaskRepository,
    create_task, get_task, get_all_tasks, get_active_tasks, update_task, delete_task,
    get_tasks_by_tags, complete_task_session, reset_task_sessions
};
use config::{
    FileConfigRepository, ConfigRepository,
    get_global_config, save_global_config, reset_global_config_to_defaults,
    update_default_timings, update_default_cycle_length, update_app_preferences,
    update_notification_preferences, update_ui_preferences, update_default_audio_config,
    get_effective_task_config, get_effective_audio_config
};
use audio::{
    AudioService,
    get_audio_library, play_audio, stop_audio, pause_audio, resume_audio,
    set_audio_volume, get_active_playbacks, stop_all_audio,
    play_notification_sound, play_background_audio, stop_background_audio,
    add_custom_audio_asset, remove_audio_asset, cleanup_finished_audio, test_audio_preview
};
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let task_repository: TaskRepository = Arc::new(InMemoryTaskRepository::with_default_task());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let config_repository: ConfigRepository = Arc::new(
                FileConfigRepository::new(app.handle())
                    .expect("Failed to initialize config repository")
            );
            app.manage(config_repository);

            let audio_service = AudioService::new()
                .expect("Failed to initialize audio service");
            app.manage(Mutex::new(audio_service));

            Ok(())
        })
        .manage(TimerService::new())
        .manage(task_repository)
        .invoke_handler(tauri::generate_handler![
            get_timer_state,
            start_timer,
            pause_timer,
            reset_timer,
            skip_phase,
            get_timer_state_with_task,
            switch_active_task,
            create_task,
            get_task,
            get_all_tasks,
            get_active_tasks,
            update_task,
            delete_task,
            get_tasks_by_tags,
            complete_task_session,
            reset_task_sessions,
            get_global_config,
            save_global_config,
            reset_global_config_to_defaults,
            update_default_timings,
            update_default_cycle_length,
            update_app_preferences,
            update_notification_preferences,
            update_ui_preferences,
            update_default_audio_config,
            get_effective_task_config,
            get_effective_audio_config,
            get_audio_library,
            play_audio,
            stop_audio,
            pause_audio,
            resume_audio,
            set_audio_volume,
            get_active_playbacks,
            stop_all_audio,
            play_notification_sound,
            play_background_audio,
            stop_background_audio,
            add_custom_audio_asset,
            remove_audio_asset,
            cleanup_finished_audio,
            test_audio_preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
