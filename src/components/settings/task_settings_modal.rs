use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use pomotoro_domain::{Task, TaskConfig, AudioConfig, TaskId, TaskStatus};
use super::{AudioConfigComponent, TimerConfigComponent};
use crate::app_events;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub current_sessions: u8,
    pub tags: Vec<String>,
    pub config: TaskConfigDto,
    pub audio_config: AudioConfig,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfigDto {
    pub work_duration: DurationDto,
    pub short_break_duration: DurationDto,
    pub long_break_duration: DurationDto,
    pub sessions_until_long_break: u8,
    pub enable_screen_blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationDto {
    pub secs: u64,
    pub nanos: u32,
}

impl From<Task> for TaskDto {
    fn from(task: Task) -> Self {
        Self {
            id: task.id.to_string(),
            name: task.name,
            description: task.description,
            max_sessions: task.max_sessions,
            current_sessions: task.current_sessions,
            tags: task.tags,
            config: TaskConfigDto {
                work_duration: DurationDto {
                    secs: task.config.work_duration.as_secs(),
                    nanos: task.config.work_duration.subsec_nanos(),
                },
                short_break_duration: DurationDto {
                    secs: task.config.short_break_duration.as_secs(),
                    nanos: task.config.short_break_duration.subsec_nanos(),
                },
                long_break_duration: DurationDto {
                    secs: task.config.long_break_duration.as_secs(),
                    nanos: task.config.long_break_duration.subsec_nanos(),
                },
                sessions_until_long_break: task.config.sessions_until_long_break,
                enable_screen_blocking: task.config.enable_screen_blocking,
            },
            audio_config: task.audio_config,
            status: match task.status {
                TaskStatus::Active => "active".to_string(),
                TaskStatus::Queued => "queued".to_string(),
                TaskStatus::Completed => "completed".to_string(),
                TaskStatus::Paused => "paused".to_string(),
            },
        }
    }
}

#[component]
pub fn TaskSettingsModal(
    _task_id: Signal<Option<String>>,
    #[prop(optional)] _on_close: Option<Callback<()>>,
    #[prop(optional)] _on_save: Option<Callback<TaskDto>>,
) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let (task, set_task) = signal::<Option<TaskDto>>(None);
    let (loading, set_loading) = signal(false);
    let (error_message, set_error_message) = signal::<Option<String>>(None);
    
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (max_sessions, set_max_sessions) = signal(4u8);
    let (tags_input, set_tags_input) = signal(String::new());
    
    let (work_minutes, set_work_minutes) = signal(25u32);
    let (short_break_minutes, set_short_break_minutes) = signal(5u32);
    let (long_break_minutes, set_long_break_minutes) = signal(15u32);
    let (sessions_until_long_break, set_sessions_until_long_break) = signal(4u8);
    let (enable_screen_blocking, set_enable_screen_blocking) = signal(false);
    
    let (volume, set_volume) = signal(0.7f32);
    let (muted, set_muted) = signal(false);
    let (enable_background_audio, set_enable_background_audio) = signal(false);

    Effect::new(move |_| {
        if let Some(id) = _task_id.get() {
            if !id.is_empty() {
                set_is_open.set(true);
                load_task(id, set_task, set_loading, set_error_message);
            }
        } else {
            set_is_open.set(false);
        }
    });

    Effect::new(move |_| {
        if let Some(task_data) = task.get() {
            set_name.set(task_data.name.clone());
            set_description.set(task_data.description.unwrap_or_default());
            set_max_sessions.set(task_data.max_sessions);
            set_tags_input.set(task_data.tags.join(", "));
            
            set_work_minutes.set((task_data.config.work_duration.secs / 60) as u32);
            set_short_break_minutes.set((task_data.config.short_break_duration.secs / 60) as u32);
            set_long_break_minutes.set((task_data.config.long_break_duration.secs / 60) as u32);
            set_sessions_until_long_break.set(task_data.config.sessions_until_long_break);
            set_enable_screen_blocking.set(task_data.config.enable_screen_blocking);
            
            set_volume.set(task_data.audio_config.volume);
            set_muted.set(task_data.audio_config.muted);
            set_enable_background_audio.set(task_data.audio_config.enable_background_audio);
        }
    });

    let close_modal = move |_| {
        set_is_open.set(false);
        if let Some(cb) = _on_close {
            cb.run(());
        }
    };

    let save_task = move |_| {
        if let Some(mut task_data) = task.get() {
            let tags: Vec<String> = tags_input.get()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            task_data.name = name.get();
            task_data.description = if description.get().is_empty() { None } else { Some(description.get()) };
            task_data.max_sessions = max_sessions.get();
            task_data.tags = tags;
            
            task_data.config.work_duration.secs = (work_minutes.get() * 60) as u64;
            task_data.config.short_break_duration.secs = (short_break_minutes.get() * 60) as u64;
            task_data.config.long_break_duration.secs = (long_break_minutes.get() * 60) as u64;
            task_data.config.sessions_until_long_break = sessions_until_long_break.get();
            task_data.config.enable_screen_blocking = enable_screen_blocking.get();
            
            task_data.audio_config.volume = volume.get();
            task_data.audio_config.muted = muted.get();
            task_data.audio_config.enable_background_audio = enable_background_audio.get();

            let task_clone = task_data.clone();
            spawn_local(async move {
                set_loading.set(true);
                set_error_message.set(None);
                
                match serde_wasm_bindgen::to_value(&task_clone) {
                    Ok(task_value) => {
                        let result = invoke(app_events::task::UPDATE, task_value).await;
                        match serde_wasm_bindgen::from_value::<TaskDto>(result) {
                            Ok(updated_task) => {
                                set_task.set(Some(updated_task.clone()));
                                if let Some(cb) = _on_save {
                                    cb.run(updated_task);
                                }
                                set_is_open.set(false);
                            }
                            Err(e) => {
                                set_error_message.set(Some(format!("Failed to save task: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        set_error_message.set(Some(format!("Serialization error: {}", e)));
                    }
                }
                set_loading.set(false);
            });
        }
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="modal-overlay" on:click=close_modal>
                <div class="modal-content task-settings-modal" on:click=|e| e.stop_propagation()>
                    <div class="modal-header">
                        <h2>"Task Settings"</h2>
                        <button class="close-btn" on:click=close_modal>"×"</button>
                    </div>

                    <Show when=move || loading.get()>
                        <div class="loading-spinner">"Loading..."</div>
                    </Show>

                    <Show when=move || error_message.get().is_some()>
                        <div class="error-message">
                            {move || error_message.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <div class="modal-body">
                        <div class="settings-section">
                            <h3>"General"</h3>
                            <div class="form-group">
                                <label for="task-name">"Name"</label>
                                <input
                                    type="text"
                                    id="task-name"
                                    prop:value=name
                                    on:input=move |ev| set_name.set(event_target_value(&ev))
                                />
                            </div>

                            <div class="form-group">
                                <label for="task-description">"Description"</label>
                                <textarea
                                    id="task-description"
                                    prop:value=description
                                    on:input=move |ev| set_description.set(event_target_value(&ev))
                                />
                            </div>

                            <div class="form-group">
                                <label for="max-sessions">"Max Sessions"</label>
                                <input
                                    type="number"
                                    id="max-sessions"
                                    min="1"
                                    max="20"
                                    prop:value=max_sessions
                                    on:input=move |ev| {
                                        if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                                            set_max_sessions.set(val.clamp(1, 20));
                                        }
                                    }
                                />
                            </div>

                            <div class="form-group">
                                <label for="tags">"Tags (comma-separated)"</label>
                                <input
                                    type="text"
                                    id="tags"
                                    placeholder="work, personal, urgent"
                                    prop:value=tags_input
                                    on:input=move |ev| set_tags_input.set(event_target_value(&ev))
                                />
                            </div>
                        </div>

                        <TimerConfigComponent />

                        <AudioConfigComponent
                            volume=volume
                            set_volume=set_volume
                            muted=muted
                            set_muted=set_muted
                            enable_background_audio=enable_background_audio
                            set_enable_background_audio=set_enable_background_audio
                        />
                    </div>

                    <div class="modal-footer">
                        <button class="btn secondary" on:click=close_modal>"Cancel"</button>
                        <button 
                            class="btn primary" 
                            on:click=save_task
                            disabled=move || loading.get()
                        >
                            "Save Changes"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

fn load_task(
    task_id: String,
    set_task: WriteSignal<Option<TaskDto>>,
    set_loading: WriteSignal<bool>,
    set_error_message: WriteSignal<Option<String>>,
) {
    spawn_local(async move {
        set_loading.set(true);
        set_error_message.set(None);
        
        let args = serde_wasm_bindgen::to_value(&task_id).unwrap_or(JsValue::NULL);
        let result = invoke(app_events::task::GET, args).await;
        
        match serde_wasm_bindgen::from_value::<Option<TaskDto>>(result) {
            Ok(Some(task)) => {
                set_task.set(Some(task));
            }
            Ok(None) => {
                set_error_message.set(Some("Task not found".to_string()));
            }
            Err(e) => {
                set_error_message.set(Some(format!("Failed to load task: {}", e)));
            }
        }
        
        set_loading.set(false);
    });
}