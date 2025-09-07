pub mod adapters;
mod bootstrap;
pub mod commands;
mod schema;

use commands::{
    add_custom_audio_asset, cleanup_finished_audio, clear_all_data,
    complete_task_session, create_task, cycle_incomplete_task, debug_create_test_task, delete_task,
    export_settings, filter_tasks_by_status, get_active_playbacks,
    get_active_tasks, get_all_tasks, get_audio_library,
    get_effective_audio_config, get_global_config,
    get_incomplete_tasks, get_task, get_task_cycle_position,
    /*get_task_effective_settings,*/ get_tasks_by_tags, get_timer_state,
    import_settings, open_data_directory, pause_audio, pause_timer, play_audio,
    play_background_audio, play_notification_sound, remove_audio_asset,
    request_notification_permission, reset_config_to_defaults, reset_task_sessions,
    reset_task_settings_to_defaults, reset_timer, resume_audio,
    save_global_config, search_tasks, search_tasks_fuzzy, set_audio_volume,
    skip_timer, start_timer, stop_all_audio, stop_audio, stop_background_audio,
    switch_timer_task_cmd, test_audio_preview, test_notification, update_appearance_config,
    update_audio_config, update_general_config,
    update_notification_config, update_storage_path, update_task,
    update_task_settings, validate_storage_path,
};
use tauri::Manager;

use crate::bootstrap::bootstrap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();

    // only enable instrumentation in development builds
    #[cfg(debug_assertions)]
    {
        let devtools = tauri_plugin_devtools::init();

        builder = builder.plugin(devtools);
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let webview = app.get_webview_window("main").unwrap();
                webview.open_devtools();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    webview.close_devtools();
                });
            }

            let app_registry =
            tauri::async_runtime::block_on(bootstrap(app.handle().clone()))
                .inspect_err(|e| {
                    eprintln!("Failed to bootstrap app: {e}");
                })?;

            //  repositories
            app.manage(app_registry.config_repository.clone());
            app.manage(app_registry.task_repository.clone());
            app.manage(app_registry.timer_repository.clone());

            // services
            app.manage(app_registry.audio_service);
            app.manage(app_registry.timer_tick_service.clone());
            app.manage(app_registry.task_cycling_service.clone());

            // events
            app.manage(app_registry.event_publisher.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_timer_state,
            start_timer,
            pause_timer,
            reset_timer,
            skip_timer,
            switch_timer_task_cmd,
            create_task,
            debug_create_test_task,
            get_task,
            get_all_tasks,
            get_active_tasks,
            update_task,
            delete_task,
            get_tasks_by_tags,
            complete_task_session,
            reset_task_sessions,
            search_tasks,
            search_tasks_fuzzy,
            filter_tasks_by_status,
            cycle_incomplete_task,
            get_task_cycle_position,
            get_incomplete_tasks,
            get_global_config,
            save_global_config,
            reset_config_to_defaults,
            update_general_config,
            update_notification_config,
            update_appearance_config,
            update_audio_config,
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
