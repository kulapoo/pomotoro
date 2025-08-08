pub mod adapters;
mod bootstrap;
pub mod commands;

use adapters::{
    create_event_publisher_with_bus, ConfigRepository, DomainEventBus, EventPublisherArc,
    FileConfigRepo, InMemoryTaskRepository, RodioAudioService, TaskRepositoryArc, TimerService,
};
use commands::*;
use domain::WorkSessionCompleted;
use std::sync::{Arc, Mutex};
use tauri::Manager;

use bootstrap::boostrap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools = tauri_plugin_devtools::init();
    let task_repository: TaskRepositoryArc = Arc::new(InMemoryTaskRepository::with_default_task());

    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            let app_registry = boostrap(app.handle().clone()).map_err(|e| {
                eprintln!("Failed to bootstrap app: {}", e.to_string());
                e
            })?;

            //  repositories
            app.manage(app_registry.config_repository);
            app.manage(task_repository.clone());

            // services
            app.manage(Mutex::new(app_registry.audio_service));
            app.manage(app_registry.timer_service.clone());

            // events
            app.manage(app_registry.event_publisher.clone());
            app.manage(app_registry.event_bus.clone());

            // tauri::async_runtime::spawn(async move {
            //     match task_repo_bootstrap_clone.get_active_tasks().await {
            //         Ok(tasks) => {
            //             if let Some(initial_task) = tasks.first().clone() {
            //                 timer_service_bootstrap_clone
            //                     .switch_task(initial_task.id, Some(initial_task))
            //                     .await;
            //             }
            //         }
            //         Err(e) => eprintln!("Failed to get active tasks: {}", e),
            //     }
            // });

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
            reset_config_to_defaults,
            update_timing_config,
            update_default_cycle_length,
            update_general_config,
            update_notification_config,
            update_appearance_config,
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
