use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use domain::{AudioConfig};
use domain::events;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfigDto {
    pub default_task_config: TaskConfigDto,
    pub default_audio_config: AudioConfig,
    pub app_preferences: AppPreferences,
    pub notification_preferences: NotificationPreferences,
    pub ui_preferences: UiPreferences,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    pub task_cycling_behavior: String,
    pub max_sessions_default: u8,
    pub auto_start_breaks: bool,
    pub auto_start_work_after_break: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub enable_desktop_notifications: bool,
    pub enable_sound_notifications: bool,
    pub show_phase_transition_notifications: bool,
    pub show_task_completion_notifications: bool,
    pub notification_position: String,
    pub auto_dismiss_delay_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub theme: String,
    pub show_seconds_in_display: bool,
    pub always_on_top: bool,
    pub compact_mode: bool,
    pub show_task_list_sidebar: bool,
    pub animate_progress: bool,
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let (config, set_config) = signal::<Option<GlobalConfigDto>>(None);
    let (loading, set_loading) = signal(false);
    let (error_message, set_error_message) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);
    let (_active_tab, _set_active_tab) = signal("general");

    // Timer settings signals
    let (work_minutes, set_work_minutes) = signal(25u32);
    let (short_break_minutes, set_short_break_minutes) = signal(5u32);
    let (long_break_minutes, set_long_break_minutes) = signal(15u32);
    let (sessions_until_long_break, set_sessions_until_long_break) = signal(4u8);
    let (enable_screen_blocking, set_enable_screen_blocking) = signal(false);
    
    // Audio settings signals
    let (volume, set_volume) = signal(0.7f32);
    let (muted, set_muted) = signal(false);
    let (enable_background_audio, set_enable_background_audio) = signal(false);

    // General settings signals
    let (max_sessions_default, set_max_sessions_default) = signal(4u8);
    let (auto_start_breaks, set_auto_start_breaks) = signal(true);
    let (auto_start_work_after_break, set_auto_start_work_after_break) = signal(false);
    let (minimize_to_tray, set_minimize_to_tray) = signal(true);
    let (start_minimized, set_start_minimized) = signal(false);

    // Notification settings signals
    let (enable_desktop_notifications, set_enable_desktop_notifications) = signal(true);
    let (show_phase_transition_notifications, set_show_phase_transition_notifications) = signal(true);
    let (show_task_completion_notifications, set_show_task_completion_notifications) = signal(true);
    let (auto_dismiss_delay_seconds, set_auto_dismiss_delay_seconds) = signal(5u32);

    // UI settings signals
    let (theme, set_theme) = signal("System".to_string());
    let (show_seconds_in_display, set_show_seconds_in_display) = signal(true);
    let (always_on_top, set_always_on_top) = signal(false);
    let (compact_mode, set_compact_mode) = signal(false);
    let (show_task_list_sidebar, set_show_task_list_sidebar) = signal(true);
    let (animate_progress, set_animate_progress) = signal(true);

    // Load configuration on mount
    Effect::new(move |_| {
        load_global_config(set_config, set_loading, set_error_message);
    });

    // Update signals when config loads
    Effect::new(move |_| {
        if let Some(config_data) = config.get() {
            set_work_minutes.set((config_data.default_task_config.work_duration.secs / 60) as u32);
            set_short_break_minutes.set((config_data.default_task_config.short_break_duration.secs / 60) as u32);
            set_long_break_minutes.set((config_data.default_task_config.long_break_duration.secs / 60) as u32);
            set_sessions_until_long_break.set(config_data.default_task_config.sessions_until_long_break);
            set_enable_screen_blocking.set(config_data.default_task_config.enable_screen_blocking);
            
            set_volume.set(config_data.default_audio_config.volume);
            set_muted.set(config_data.default_audio_config.muted);
            set_enable_background_audio.set(config_data.default_audio_config.enable_background_audio);

            set_max_sessions_default.set(config_data.app_preferences.max_sessions_default);
            set_auto_start_breaks.set(config_data.app_preferences.auto_start_breaks);
            set_auto_start_work_after_break.set(config_data.app_preferences.auto_start_work_after_break);
            set_minimize_to_tray.set(config_data.app_preferences.minimize_to_tray);
            set_start_minimized.set(config_data.app_preferences.start_minimized);

            set_enable_desktop_notifications.set(config_data.notification_preferences.enable_desktop_notifications);
            set_show_phase_transition_notifications.set(config_data.notification_preferences.show_phase_transition_notifications);
            set_show_task_completion_notifications.set(config_data.notification_preferences.show_task_completion_notifications);
            set_auto_dismiss_delay_seconds.set(config_data.notification_preferences.auto_dismiss_delay_seconds);

            set_theme.set(config_data.ui_preferences.theme);
            set_show_seconds_in_display.set(config_data.ui_preferences.show_seconds_in_display);
            set_always_on_top.set(config_data.ui_preferences.always_on_top);
            set_compact_mode.set(config_data.ui_preferences.compact_mode);
            set_show_task_list_sidebar.set(config_data.ui_preferences.show_task_list_sidebar);
            set_animate_progress.set(config_data.ui_preferences.animate_progress);
        }
    });

    let save_config = move |_| {
        spawn_local(async move {
            set_loading.set(true);
            set_error_message.set(None);
            set_success_message.set(None);

            let updated_config = GlobalConfigDto {
                default_task_config: TaskConfigDto {
                    work_duration: DurationDto { secs: (work_minutes.get() * 60) as u64, nanos: 0 },
                    short_break_duration: DurationDto { secs: (short_break_minutes.get() * 60) as u64, nanos: 0 },
                    long_break_duration: DurationDto { secs: (long_break_minutes.get() * 60) as u64, nanos: 0 },
                    sessions_until_long_break: sessions_until_long_break.get(),
                    enable_screen_blocking: enable_screen_blocking.get(),
                },
                default_audio_config: AudioConfig {
                    work_notification_sound: None,
                    break_notification_sound: None,
                    background_sound: None,
                    volume: volume.get(),
                    enable_background_audio: enable_background_audio.get(),
                    muted: muted.get(),
                },
                app_preferences: AppPreferences {
                    task_cycling_behavior: "Manual".to_string(),
                    max_sessions_default: max_sessions_default.get(),
                    auto_start_breaks: auto_start_breaks.get(),
                    auto_start_work_after_break: auto_start_work_after_break.get(),
                    minimize_to_tray: minimize_to_tray.get(),
                    start_minimized: start_minimized.get(),
                },
                notification_preferences: NotificationPreferences {
                    enable_desktop_notifications: enable_desktop_notifications.get(),
                    enable_sound_notifications: !muted.get(),
                    show_phase_transition_notifications: show_phase_transition_notifications.get(),
                    show_task_completion_notifications: show_task_completion_notifications.get(),
                    notification_position: "TopRight".to_string(),
                    auto_dismiss_delay_seconds: auto_dismiss_delay_seconds.get(),
                },
                ui_preferences: UiPreferences {
                    theme: theme.get(),
                    show_seconds_in_display: show_seconds_in_display.get(),
                    always_on_top: always_on_top.get(),
                    compact_mode: compact_mode.get(),
                    show_task_list_sidebar: show_task_list_sidebar.get(),
                    animate_progress: animate_progress.get(),
                },
            };

            match serde_wasm_bindgen::to_value(&updated_config) {
                Ok(config_value) => {
                    let result = unsafe { invoke(events::config::SAVE_GLOBAL, config_value).await };
                    match serde_wasm_bindgen::from_value::<()>(result) {
                        Ok(_) => {
                            set_config.set(Some(updated_config));
                            set_success_message.set(Some("Settings saved successfully!".to_string()));
                        }
                        Err(e) => {
                            set_error_message.set(Some(format!("Failed to save settings: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Serialization error: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    let reset_to_defaults = move |_| {
        spawn_local(async move {
            set_loading.set(true);
            set_error_message.set(None);
            set_success_message.set(None);
            let result = unsafe { invoke(events::config::RESET_TO_DEFAULTS, JsValue::NULL).await };
            match serde_wasm_bindgen::from_value::<GlobalConfigDto>(result) {
                Ok(default_config) => {
                    set_config.set(Some(default_config));
                    set_success_message.set(Some("Settings reset to defaults!".to_string()));
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Failed to reset settings: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="settings-section">
            <h1 class="section-title">"Settings"</h1>

            // Status messages
            <Show when=move || loading.get()>
                <div class="setting-group" style="text-align: center;">
                    <p>"Loading settings..."</p>
                </div>
            </Show>

            <Show when=move || error_message.get().is_some()>
                <div class="setting-group" style="color: #ff6b6b;">
                    <p>{move || error_message.get().unwrap_or_default()}</p>
                </div>
            </Show>

            <Show when=move || success_message.get().is_some()>
                <div class="setting-group" style="color: #4ecdc4;">
                    <p>{move || success_message.get().unwrap_or_default()}</p>
                </div>
            </Show>

            // Timer Configuration
            <div class="setting-group">
                <div class="setting-label">"Focus Duration (minutes)"</div>
                <input
                    type="number"
                    min="5"
                    max="60"
                    prop:value=work_minutes
                    class="setting-input"
                    on:input=move |ev| {
                        if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                            set_work_minutes.set(val.clamp(5, 60));
                        }
                    }
                />
            </div>

            <div class="setting-group">
                <div class="setting-label">"Short Break (minutes)"</div>
                <input
                    type="number"
                    min="1"
                    max="30"
                    prop:value=short_break_minutes
                    class="setting-input"
                    on:input=move |ev| {
                        if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                            set_short_break_minutes.set(val.clamp(1, 30));
                        }
                    }
                />
            </div>

            <div class="setting-group">
                <div class="setting-label">"Long Break (minutes)"</div>
                <input
                    type="number"
                    min="5"
                    max="60"
                    prop:value=long_break_minutes
                    class="setting-input"
                    on:input=move |ev| {
                        if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                            set_long_break_minutes.set(val.clamp(5, 60));
                        }
                    }
                />
            </div>

            // Notifications
            <div class="setting-group">
                <div class="setting-label">"Sound Notifications"</div>
                <label style="display: flex; align-items: center; gap: 10px;">
                    <input
                        type="checkbox"
                        prop:checked=enable_desktop_notifications
                        on:change=move |ev| set_enable_desktop_notifications.set(event_target_checked(&ev))
                    />
                    <span>"Enable sound notifications"</span>
                </label>
            </div>

            <div class="setting-group">
                <div class="setting-label">"Desktop Notifications"</div>
                <label style="display: flex; align-items: center; gap: 10px;">
                    <input
                        type="checkbox"
                        prop:checked=enable_desktop_notifications
                        on:change=move |ev| set_enable_desktop_notifications.set(event_target_checked(&ev))
                    />
                    <span>"Enable desktop notifications"</span>
                </label>
            </div>

            // Action buttons
            <div class="setting-group" style="display: flex; gap: 20px; justify-content: center;">
                <button 
                    class="btn btn-secondary"
                    on:click=reset_to_defaults
                    disabled=move || loading.get()
                >
                    "Reset to Defaults"
                </button>
                <button 
                    class="btn btn-primary"
                    on:click=save_config
                    disabled=move || loading.get()
                >
                    {move || if loading.get() { "Saving..." } else { "Save Settings" }}
                </button>
            </div>
        </div>
    }
}

fn load_global_config(
    set_config: WriteSignal<Option<GlobalConfigDto>>,
    set_loading: WriteSignal<bool>,
    set_error_message: WriteSignal<Option<String>>,
) {
    spawn_local(async move {
        set_loading.set(true);
        set_error_message.set(None);
        
        let result = unsafe { invoke(events::config::GET_GLOBAL, JsValue::NULL).await };
        match serde_wasm_bindgen::from_value::<GlobalConfigDto>(result) {
            Ok(config) => {
                set_config.set(Some(config));
            }
            Err(e) => {
                set_error_message.set(Some(format!("Failed to load settings: {}", e)));
            }
        }
        
        set_loading.set(false);
    });
}