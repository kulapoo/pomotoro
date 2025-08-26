pub mod adapters;
mod bootstrap;
pub mod commands;

use commands::{
    add_custom_audio_asset, cleanup_finished_audio, complete_task_session,
    create_task, delete_task, get_active_playbacks, get_active_tasks,
    get_all_tasks, get_audio_library, get_effective_audio_config,
    get_effective_task_config, get_global_config, get_task, get_tasks_by_tags,
    get_timer_state, pause_audio, pause_timer, play_audio,
    play_background_audio, play_notification_sound, remove_audio_asset,
    reset_config_to_defaults, reset_task_sessions, reset_timer, resume_audio,
    save_global_config, set_audio_volume, skip_phase, start_timer,
    stop_all_audio, stop_audio, stop_background_audio, switch_active_task,
    test_audio_preview, update_appearance_config, update_audio_config,
    update_default_cycle_length, update_general_config,
    update_notification_config, update_task, update_timing_config,
};
use std::sync::Mutex;
use tauri::Manager;

use crate::bootstrap::bootstrap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // only enable instrumentation in development builds
    #[cfg(debug_assertions)]
    let devtools = tauri_plugin_devtools::init();

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

            // services
            app.manage(Mutex::new(app_registry.audio_service));
            app.manage(app_registry.timer_service.clone());

            // events
            app.manage(app_registry.event_publisher.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_timer_state,
            start_timer,
            pause_timer,
            reset_timer,
            skip_phase,
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
