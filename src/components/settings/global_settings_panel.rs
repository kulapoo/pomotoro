use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use super::{AudioConfigComponent, TimerConfigComponent};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub default_task_config: TaskConfig,
    pub default_audio_config: AudioConfig,
    pub app_preferences: AppPreferences,
    pub notification_preferences: NotificationPreferences,
    pub ui_preferences: UiPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub sessions_until_long_break: u8,
    pub enable_screen_blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub work_notification_sound: Option<String>,
    pub break_notification_sound: Option<String>,
    pub background_sound: Option<String>,
    pub volume: f32,
    pub enable_background_audio: bool,
    pub muted: bool,
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
pub fn GlobalSettingsPanel(
    #[prop(optional)] _is_open: Signal<bool>,
    #[prop(optional)] _on_close: Option<Callback<()>>,
) -> impl IntoView {
    let (config, set_config) = signal::<Option<GlobalConfig>>(None);
    let (loading, set_loading) = signal(false);
    let (error_message, set_error_message) = signal::<Option<String>>(None);
    let (active_tab, set_active_tab) = signal("general");

    let (work_minutes, set_work_minutes) = signal(25u32);
    let (short_break_minutes, set_short_break_minutes) = signal(5u32);
    let (long_break_minutes, set_long_break_minutes) = signal(15u32);
    let (sessions_until_long_break, set_sessions_until_long_break) = signal(4u8);
    let (enable_screen_blocking, set_enable_screen_blocking) = signal(false);
    
    let (volume, set_volume) = signal(0.7f32);
    let (muted, set_muted) = signal(false);
    let (enable_background_audio, set_enable_background_audio) = signal(false);

    let (max_sessions_default, set_max_sessions_default) = signal(4u8);
    let (auto_start_breaks, set_auto_start_breaks) = signal(true);
    let (auto_start_work_after_break, set_auto_start_work_after_break) = signal(false);
    let (minimize_to_tray, set_minimize_to_tray) = signal(true);
    let (start_minimized, set_start_minimized) = signal(false);

    let (enable_desktop_notifications, set_enable_desktop_notifications) = signal(true);
    let (show_phase_transition_notifications, set_show_phase_transition_notifications) = signal(true);
    let (show_task_completion_notifications, set_show_task_completion_notifications) = signal(true);
    let (auto_dismiss_delay_seconds, set_auto_dismiss_delay_seconds) = signal(5u32);

    let (theme, set_theme) = signal("System".to_string());
    let (show_seconds_in_display, set_show_seconds_in_display) = signal(true);
    let (always_on_top, set_always_on_top) = signal(false);
    let (compact_mode, set_compact_mode) = signal(false);
    let (show_task_list_sidebar, set_show_task_list_sidebar) = signal(true);
    let (animate_progress, set_animate_progress) = signal(true);

    Effect::new(move |_| {
        if _is_open.get() {
            load_global_config(set_config, set_loading, set_error_message);
        }
    });

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

    let close_panel = move |_| {
        if let Some(cb) = _on_close {
            cb.run(());
        }
    };

    let save_config = move |_| {
        spawn_local(async move {
            set_loading.set(true);
            set_error_message.set(None);

            let updated_config = GlobalConfig {
                default_task_config: TaskConfig {
                    work_duration: Duration { secs: (work_minutes.get() * 60) as u64, nanos: 0 },
                    short_break_duration: Duration { secs: (short_break_minutes.get() * 60) as u64, nanos: 0 },
                    long_break_duration: Duration { secs: (long_break_minutes.get() * 60) as u64, nanos: 0 },
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
                    let result = invoke("save_global_config", config_value).await;
                    match serde_wasm_bindgen::from_value::<()>(result) {
                        Ok(_) => {
                            set_config.set(Some(updated_config));
                        }
                        Err(e) => {
                            set_error_message.set(Some(format!("Failed to save config: {}", e)));
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
            let result = invoke("reset_global_config_to_defaults", JsValue::NULL).await;
            match serde_wasm_bindgen::from_value::<GlobalConfig>(result) {
                Ok(default_config) => {
                    set_config.set(Some(default_config));
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Failed to reset config: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <Show when=move || _is_open.get()>
            <div class="modal-overlay" on:click=close_panel>
                <div class="modal-content global-settings-panel" on:click=|e| e.stop_propagation()>
                    <div class="modal-header">
                        <h2>"Global Settings"</h2>
                        <button class="close-btn" on:click=close_panel>"×"</button>
                    </div>

                    <Show when=move || loading.get()>
                        <div class="loading-spinner">"Loading..."</div>
                    </Show>

                    <Show when=move || error_message.get().is_some()>
                        <div class="error-message">
                            {move || error_message.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <div class="settings-tabs">
                        <button
                            class=move || if active_tab.get() == "general" { "tab-btn active" } else { "tab-btn" }
                            on:click=move |_| set_active_tab.set("general")
                        >
                            "General"
                        </button>
                        <button
                            class=move || if active_tab.get() == "timer" { "tab-btn active" } else { "tab-btn" }
                            on:click=move |_| set_active_tab.set("timer")
                        >
                            "Default Timer"
                        </button>
                        <button
                            class=move || if active_tab.get() == "audio" { "tab-btn active" } else { "tab-btn" }
                            on:click=move |_| set_active_tab.set("audio")
                        >
                            "Audio"
                        </button>
                        <button
                            class=move || if active_tab.get() == "notifications" { "tab-btn active" } else { "tab-btn" }
                            on:click=move |_| set_active_tab.set("notifications")
                        >
                            "Notifications"
                        </button>
                        <button
                            class=move || if active_tab.get() == "ui" { "tab-btn active" } else { "tab-btn" }
                            on:click=move |_| set_active_tab.set("ui")
                        >
                            "Interface"
                        </button>
                    </div>

                    <div class="modal-body">
                        <Show when=move || active_tab.get() == "general">
                            <div class="settings-section">
                                <h3>"Application Preferences"</h3>
                                
                                <div class="form-group">
                                    <label for="max-sessions-default">"Default Max Sessions for New Tasks"</label>
                                    <input
                                        type="number"
                                        id="max-sessions-default"
                                        min="1"
                                        max="20"
                                        prop:value=max_sessions_default
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                                                set_max_sessions_default.set(val.clamp(1, 20));
                                            }
                                        }
                                    />
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=auto_start_breaks
                                            on:change=move |ev| set_auto_start_breaks.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Auto-start Breaks"
                                    </label>
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=auto_start_work_after_break
                                            on:change=move |ev| set_auto_start_work_after_break.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Auto-start Work After Break"
                                    </label>
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=minimize_to_tray
                                            on:change=move |ev| set_minimize_to_tray.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Minimize to System Tray"
                                    </label>
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=start_minimized
                                            on:change=move |ev| set_start_minimized.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Start Minimized"
                                    </label>
                                </div>
                            </div>
                        </Show>

                        <Show when=move || active_tab.get() == "timer">
                            <TimerConfigComponent
                                work_minutes=work_minutes
                                set_work_minutes=set_work_minutes
                                short_break_minutes=short_break_minutes
                                set_short_break_minutes=set_short_break_minutes
                                long_break_minutes=long_break_minutes
                                set_long_break_minutes=set_long_break_minutes
                                sessions_until_long_break=sessions_until_long_break
                                set_sessions_until_long_break=set_sessions_until_long_break
                                enable_screen_blocking=enable_screen_blocking
                                set_enable_screen_blocking=set_enable_screen_blocking
                            />
                        </Show>

                        <Show when=move || active_tab.get() == "audio">
                            <AudioConfigComponent
                                volume=volume
                                set_volume=set_volume
                                muted=muted
                                set_muted=set_muted
                                enable_background_audio=enable_background_audio
                                set_enable_background_audio=set_enable_background_audio
                            />
                        </Show>

                        <Show when=move || active_tab.get() == "notifications">
                            <div class="settings-section">
                                <h3>"Notification Preferences"</h3>
                                
                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=enable_desktop_notifications
                                            on:change=move |ev| set_enable_desktop_notifications.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Enable Desktop Notifications"
                                    </label>
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=show_phase_transition_notifications
                                            disabled=move || !enable_desktop_notifications.get()
                                            on:change=move |ev| set_show_phase_transition_notifications.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Show Phase Transition Notifications"
                                    </label>
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=show_task_completion_notifications
                                            disabled=move || !enable_desktop_notifications.get()
                                            on:change=move |ev| set_show_task_completion_notifications.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Show Task Completion Notifications"
                                    </label>
                                </div>

                                <div class="form-group">
                                    <label for="auto-dismiss-delay">"Auto-dismiss After (seconds)"</label>
                                    <input
                                        type="number"
                                        id="auto-dismiss-delay"
                                        min="1"
                                        max="30"
                                        prop:value=auto_dismiss_delay_seconds
                                        disabled=move || !enable_desktop_notifications.get()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                                set_auto_dismiss_delay_seconds.set(val.clamp(1, 30));
                                            }
                                        }
                                    />
                                </div>
                            </div>
                        </Show>

                        <Show when=move || active_tab.get() == "ui">
                            <div class="settings-section">
                                <h3>"Interface Preferences"</h3>
                                
                                <div class="form-group">
                                    <label for="theme">"Theme"</label>
                                    <select 
                                        id="theme" 
                                        prop:value=theme
                                        on:change=move |ev| set_theme.set(event_target_value(&ev))
                                    >
                                        <option value="Light">"Light"</option>
                                        <option value="Dark">"Dark"</option>
                                        <option value="System">"System"</option>
                                    </select>
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=show_seconds_in_display
                                            on:change=move |ev| set_show_seconds_in_display.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Show Seconds in Timer Display"
                                    </label>
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=always_on_top
                                            on:change=move |ev| set_always_on_top.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Always on Top"
                                    </label>
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=compact_mode
                                            on:change=move |ev| set_compact_mode.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Compact Mode"
                                    </label>
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=show_task_list_sidebar
                                            on:change=move |ev| set_show_task_list_sidebar.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Show Task List Sidebar"
                                    </label>
                                </div>

                                <div class="form-group checkbox-group">
                                    <label class="checkbox-label">
                                        <input
                                            type="checkbox"
                                            prop:checked=animate_progress
                                            on:change=move |ev| set_animate_progress.set(event_target_checked(&ev))
                                        />
                                        <span class="checkmark"></span>
                                        "Animate Progress Ring"
                                    </label>
                                </div>
                            </div>
                        </Show>
                    </div>

                    <div class="modal-footer">
                        <button class="btn secondary" on:click=reset_to_defaults>"Reset to Defaults"</button>
                        <div class="button-group">
                            <button class="btn secondary" on:click=close_panel>"Cancel"</button>
                            <button 
                                class="btn primary" 
                                on:click=save_config
                                disabled=move || loading.get()
                            >
                                "Save Settings"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

fn load_global_config(
    set_config: WriteSignal<Option<GlobalConfig>>,
    set_loading: WriteSignal<bool>,
    set_error_message: WriteSignal<Option<String>>,
) {
    spawn_local(async move {
        set_loading.set(true);
        set_error_message.set(None);
        
        let result = invoke("get_global_config", JsValue::NULL).await;
        match serde_wasm_bindgen::from_value::<GlobalConfig>(result) {
            Ok(config) => {
                set_config.set(Some(config));
            }
            Err(e) => {
                set_error_message.set(Some(format!("Failed to load config: {}", e)));
            }
        }
        
        set_loading.set(false);
    });
}