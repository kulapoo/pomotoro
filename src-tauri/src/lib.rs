mod timer;
mod task;

use timer::{
    TimerManager, get_timer_state, start_timer, pause_timer, reset_timer, skip_phase,
    get_timer_state_with_task, switch_active_task
};
use task::{
    InMemoryTaskRepository, TaskRepository,
    create_task, get_task, get_all_tasks, get_active_tasks, update_task, delete_task,
    get_tasks_by_tags, complete_task_session, reset_task_sessions
};
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let task_repository: TaskRepository = Arc::new(InMemoryTaskRepository::with_default_task());
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .manage(TimerManager::new())
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
            reset_task_sessions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
