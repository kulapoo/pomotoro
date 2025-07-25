pub mod controllers;
pub mod application;
pub mod infrastructure;
pub mod commands;

use tauri::Manager;
use controllers::*;
use infrastructure::{
    TimerService, InMemoryTaskRepository, TaskRepositoryArc, 
    FileConfigRepo, ConfigRepository, AudioService,
    create_composite_event_publisher, EventPublisherArc
};
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let task_repository: TaskRepositoryArc = Arc::new(InMemoryTaskRepository::with_default_task());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            let config_repository: ConfigRepository = Arc::new(
                FileConfigRepo::new(app.handle())
                    .expect("Failed to initialize config repository")
            );
            app.manage(config_repository);

            let audio_service = AudioService::new()
                .expect("Failed to initialize audio service");
            app.manage(Mutex::new(audio_service));

            // Create composite event publisher for domain events
            let event_publisher: EventPublisherArc = create_composite_event_publisher(app.handle().clone());
            app.manage(event_publisher.clone());

            // Create timer service with domain services
            let timer_service = TimerService::new_with_services(event_publisher.clone(), task_repository.clone());
            app.manage(timer_service);

            // Manage the task repository for command handlers
            app.manage(task_repository.clone());

            // Dependencies are managed and injected into controllers as needed

            Ok(())
        })
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
            update_general,
            update_notification_preferences,
            update_appearance,
            update_audio_config,
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
