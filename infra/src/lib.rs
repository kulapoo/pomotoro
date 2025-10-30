pub mod adapters;
mod bootstrap;
pub mod commands;

mod schema;

use commands::{
    add_custom_audio_asset, cleanup_finished_audio, clear_all_data,
    complete_task, create_task,
    delete_task, export_settings,
    filter_tasks_by_status, get_active_playbacks, get_active_tasks,
    get_all_tasks, get_audio_library, get_effective_audio_config,
    get_global_config, get_incomplete_tasks, get_task,
    get_tasks_by_tags, get_timer_state,
    update_timer_secs,
    import_settings, open_data_directory, pause_audio, pause_timer, play_audio,
    play_background_audio, play_notification_sound, remove_audio_asset,
    resume_timer,
    request_notification_permission, reset_config_to_defaults,
    reset_task_settings_to_defaults, reset_task, reset_timer,
    resume_audio, save_global_config, search_tasks, search_tasks_fuzzy,
    set_audio_volume, skip_phase, start_timer, stop_all_audio, stop_audio,
    stop_background_audio, switch_active_task, test_audio_preview,
    test_notification, update_appearance_config, update_audio_config,
    update_general_config, update_notification_config, update_storage_path,
    update_task, update_task_settings, update_timing_config, validate_storage_path,
};
use std::sync::Arc;
use tauri::{Emitter, Manager};

use crate::bootstrap::bootstrap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();

    // only enable instrumentation in development builds
    // #[cfg(debug_assertions)]
    // {
    //     let devtools = tauri_plugin_devtools::init();
    //     builder = builder.plugin(devtools);
    // }

    builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("pomotoro".to_string()),
                    }),
                ])
                .level(log::LevelFilter::Info)
                .filter(|metadata| {
                    // Only include logs from our crates
                    metadata.target().starts_with("infra::")
                        || metadata.target().starts_with("domain::")
                        || metadata.target().starts_with("usecases::")
                })
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            // Get the app handle for initialization
            let app_handle = app.handle().clone();

            // Initialize the app synchronously using async block
            let registry_result = tauri::async_runtime::block_on(async move {
                bootstrap(app_handle.clone()).await
            });

            match registry_result {
                Ok(registry) => {
                    let registry = Arc::new(registry);

                    // Store all the managed state in Tauri
                    app.manage(registry.config_repository.clone());
                    app.manage(registry.task_repository.clone());
                    app.manage(registry.timer_repository.clone());
                    app.manage(registry.audio_service.clone());
                    app.manage(registry.timer_tick_service.clone());
                    app.manage(registry.event_publisher.clone());

                    log::info!("Application initialized successfully");

                    // Emit an event to notify the frontend that initialization is complete
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("app:initialized", ());
                    }
                }
                Err(err) => {
                    let error_msg = format!("Failed to bootstrap app: {}", err);
                    log::error!("{}", error_msg);

                    // Emit an error event to the frontend
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("app:initialization-failed", &error_msg);
                    }

                    // Return error to prevent app from starting
                    return Err(error_msg.into());
                }
            }

            // Optional: Open devtools in debug mode with better timing
            #[cfg(debug_assertions)]
            {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    // Wait a bit for the window to be ready
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                    if let Some(window) = app_handle.get_webview_window("main") {
                        window.open_devtools();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Existing commands
            get_timer_state,
            start_timer,
            pause_timer,
            resume_timer,
            reset_timer,
            skip_phase,
            update_timer_secs,
            switch_active_task,
            create_task,
            get_task,
            get_all_tasks,
            get_active_tasks,
            update_task,
            delete_task,
            get_tasks_by_tags,
            complete_task,
            reset_task,
            search_tasks,
            search_tasks_fuzzy,
            filter_tasks_by_status,
            get_incomplete_tasks,
            get_global_config,
            save_global_config,
            reset_config_to_defaults,
            update_general_config,
            update_notification_config,
            update_appearance_config,
            update_audio_config,
            update_timing_config,
            get_effective_audio_config,
            update_task_settings,
            reset_task_settings_to_defaults,
            // get_task_effective_settings,
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
            test_audio_preview,
            open_data_directory,
            clear_all_data,
            validate_storage_path,
            update_storage_path,
            export_settings,
            import_settings,
            test_notification,
            request_notification_permission
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}