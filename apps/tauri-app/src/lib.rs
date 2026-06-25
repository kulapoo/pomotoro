pub mod adapters;
pub mod commands;
pub mod tray;

use commands::*;
use std::sync::Arc;
use tauri::{Emitter, Manager};

use crate::adapters::{NotificationService, TauriAppHandleEmitter};

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
                    // Include logs from our own crates plus JS-forwarded messages
                    // routed through tauri-plugin-log from the React UI.
                    metadata.target().starts_with("infra::")
                        || metadata.target().starts_with("domain::")
                        || metadata.target().starts_with("usecases::")
                        || metadata.target().starts_with("tauri_app::")
                        || metadata.target().starts_with("tauri_plugin_log::")
                        || metadata.target().starts_with("app:webview")
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
                // Create the Tauri-specific emitter
                let emitter: Arc<dyn infra::adapters::events::app_emitter::Emitter> =
                    Arc::new(TauriAppHandleEmitter::new(app_handle.clone()));

                // Get initial config for notification service
                let storage_path = dirs::data_dir()
                    .ok_or_else(|| anyhow::anyhow!("Failed to get user data directory"))?
                    .join("pomotoro");

                std::fs::create_dir_all(&storage_path)
                    .map_err(|e| anyhow::anyhow!("Failed to create storage directory: {}", e))?;

                let db_path = storage_path.join("pomotoro.db");
                let db_pool = Arc::new(
                    infra::adapters::establish_connection(&db_path)
                        .map_err(|e| anyhow::anyhow!("Failed to establish database connection: {}", e))?,
                );

                infra::adapters::run_migrations(&db_pool)
                    .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

                let config_repository: Arc<dyn domain::ConfigRepository + Send + Sync> =
                    Arc::new(infra::adapters::SqliteConfigRepository::new(db_pool.clone()));

                let config = config_repository
                    .get_config()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get config: {}", e))?;

                // Create the Tauri-specific notification service
                let notification_service: Arc<dyn infra::adapters::notifications::NotificationServiceTrait> =
                    Arc::new(NotificationService::new(
                        app_handle.clone(),
                        config.notification,
                    ));

                // Bootstrap the core infrastructure with our Tauri implementations
                infra::bootstrap(emitter, notification_service).await
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

                    // Build the system tray (depends on managed repositories).
                    if let Err(e) = tray::build_tray(app.handle()) {
                        log::error!("Failed to build system tray: {}", e);
                    }

                    // Honor `start_minimized`: hide the window on launch.
                    if tray::current_general(app.handle()).start_minimized {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
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
        .on_window_event(move |window, event| {
            // Close-to-tray: when minimize_to_tray is enabled, intercept the
            // close request and hide the window instead of exiting.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if tray::current_general(window.app_handle()).minimize_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                    let _ = tray::refresh(window.app_handle(), None);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Timer commands
            get_timer_state,
            start_timer,
            pause_timer,
            resume_timer,
            reset_timer,
            reset_timer_phase,
            skip_phase,
            update_timer_secs,
            switch_active_task,
            // Task commands
            create_task,
            get_task,
            get_all_tasks,
            get_active_task,
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
            // Config commands
            get_global_config,
            save_global_config,
            reset_config_to_defaults,
            update_general_config,
            update_notification_config,
            update_appearance_config,
            update_audio_config,
            update_timing_config,
            get_effective_audio_config,
            // Task settings commands
            update_task_settings,
            reset_task_settings_to_defaults,
            // Audio commands
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
            // Storage commands
            open_data_directory,
            clear_all_data,
            validate_storage_path,
            update_storage_path,
            export_settings,
            import_settings,
            // Notification commands
            test_notification,
            request_notification_permission,
            // Screen blocker commands
            activate_screen_block,
            deactivate_screen_block
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
