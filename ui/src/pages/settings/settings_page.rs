use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use domain::{AudioConfig};
use crate::components::PageHeader;
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
    let (active_tab, set_active_tab) = signal("general");

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
        <div class="w-full">
            <PageHeader title="Settings".to_string() />
            
            <div class="bg-white rounded-2xl shadow-lg overflow-hidden">
                // Status messages
                <Show when=move || loading.get()>
                    <div class="bg-blue-50 border-l-4 border-blue-400 p-4">
                        <div class="flex">
                            <div class="text-blue-700">
                                "Loading settings..."
                            </div>
                        </div>
                    </div>
                </Show>

                <Show when=move || error_message.get().is_some()>
                    <div class="bg-red-50 border-l-4 border-red-400 p-4">
                        <div class="flex">
                            <div class="text-red-700">
                                {move || error_message.get().unwrap_or_default()}
                            </div>
                        </div>
                    </div>
                </Show>

                <Show when=move || success_message.get().is_some()>
                    <div class="bg-green-50 border-l-4 border-green-400 p-4">
                        <div class="flex">
                            <div class="text-green-700">
                                {move || success_message.get().unwrap_or_default()}
                            </div>
                        </div>
                    </div>
                </Show>

                // Settings tabs
                <div class="border-b border-gray-200">
                    <nav class="flex space-x-8 px-6" aria-label="Tabs">
                        <button
                            class=move || if active_tab.get() == "general" { 
                                "py-4 px-1 border-b-2 border-blue-500 font-medium text-sm text-blue-600" 
                            } else { 
                                "py-4 px-1 border-b-2 border-transparent font-medium text-sm text-gray-500 hover:text-gray-700 hover:border-gray-300" 
                            }
                            on:click=move |_| set_active_tab.set("general")
                        >
                            "General"
                        </button>
                        <button
                            class=move || if active_tab.get() == "timer" { 
                                "py-4 px-1 border-b-2 border-blue-500 font-medium text-sm text-blue-600" 
                            } else { 
                                "py-4 px-1 border-b-2 border-transparent font-medium text-sm text-gray-500 hover:text-gray-700 hover:border-gray-300" 
                            }
                            on:click=move |_| set_active_tab.set("timer")
                        >
                            "Timer Defaults"
                        </button>
                        <button
                            class=move || if active_tab.get() == "audio" { 
                                "py-4 px-1 border-b-2 border-blue-500 font-medium text-sm text-blue-600" 
                            } else { 
                                "py-4 px-1 border-b-2 border-transparent font-medium text-sm text-gray-500 hover:text-gray-700 hover:border-gray-300" 
                            }
                            on:click=move |_| set_active_tab.set("audio")
                        >
                            "Audio"
                        </button>
                        <button
                            class=move || if active_tab.get() == "notifications" { 
                                "py-4 px-1 border-b-2 border-blue-500 font-medium text-sm text-blue-600" 
                            } else { 
                                "py-4 px-1 border-b-2 border-transparent font-medium text-sm text-gray-500 hover:text-gray-700 hover:border-gray-300" 
                            }
                            on:click=move |_| set_active_tab.set("notifications")
                        >
                            "Notifications"
                        </button>
                        <button
                            class=move || if active_tab.get() == "interface" { 
                                "py-4 px-1 border-b-2 border-blue-500 font-medium text-sm text-blue-600" 
                            } else { 
                                "py-4 px-1 border-b-2 border-transparent font-medium text-sm text-gray-500 hover:text-gray-700 hover:border-gray-300" 
                            }
                            on:click=move |_| set_active_tab.set("interface")
                        >
                            "Interface"
                        </button>
                    </nav>
                </div>

                // Settings content
                <div class="p-6">
                    <Show when=move || active_tab.get() == "general">
                        <div class="space-y-6">
                            <div>
                                <h3 class="text-lg font-medium text-gray-900 mb-4">"Application Preferences"</h3>
                                
                                <div class="space-y-4">
                                    <div>
                                        <label for="max-sessions-default" class="block text-sm font-medium text-gray-700 mb-1">
                                            "Default Max Sessions for New Tasks"
                                        </label>
                                        <input
                                            type="number"
                                            id="max-sessions-default"
                                            min="1"
                                            max="20"
                                            prop:value=max_sessions_default
                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            on:input=move |ev| {
                                                if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                                                    set_max_sessions_default.set(val.clamp(1, 20));
                                                }
                                            }
                                        />
                                    </div>

                                    <div class="space-y-3">
                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=auto_start_breaks
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_auto_start_breaks.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Auto-start Breaks"</span>
                                        </label>

                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=auto_start_work_after_break
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_auto_start_work_after_break.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Auto-start Work After Break"</span>
                                        </label>

                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=minimize_to_tray
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_minimize_to_tray.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Minimize to System Tray"</span>
                                        </label>

                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=start_minimized
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_start_minimized.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Start Minimized"</span>
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Show>

                    <Show when=move || active_tab.get() == "timer">
                        <div class="space-y-6">
                            <div>
                                <h3 class="text-lg font-medium text-gray-900 mb-4">"Timer Defaults"</h3>
                                
                                <div class="space-y-4">
                                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                        <div>
                                            <label for="work-minutes" class="block text-sm font-medium text-gray-700 mb-1">
                                                "Work Duration (minutes)"
                                            </label>
                                            <input
                                                type="number"
                                                id="work-minutes"
                                                min="5"
                                                max="60"
                                                prop:value=work_minutes
                                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                on:input=move |ev| {
                                                    if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                                        set_work_minutes.set(val.clamp(5, 60));
                                                    }
                                                }
                                            />
                                        </div>

                                        <div>
                                            <label for="short-break-minutes" class="block text-sm font-medium text-gray-700 mb-1">
                                                "Short Break (minutes)"
                                            </label>
                                            <input
                                                type="number"
                                                id="short-break-minutes"
                                                min="1"
                                                max="30"
                                                prop:value=short_break_minutes
                                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                on:input=move |ev| {
                                                    if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                                        set_short_break_minutes.set(val.clamp(1, 30));
                                                    }
                                                }
                                            />
                                        </div>

                                        <div>
                                            <label for="long-break-minutes" class="block text-sm font-medium text-gray-700 mb-1">
                                                "Long Break (minutes)"
                                            </label>
                                            <input
                                                type="number"
                                                id="long-break-minutes"
                                                min="5"
                                                max="60"
                                                prop:value=long_break_minutes
                                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                on:input=move |ev| {
                                                    if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                                        set_long_break_minutes.set(val.clamp(5, 60));
                                                    }
                                                }
                                            />
                                        </div>
                                    </div>

                                    <div>
                                        <label for="sessions-until-long-break" class="block text-sm font-medium text-gray-700 mb-1">
                                            "Sessions Until Long Break"
                                        </label>
                                        <input
                                            type="number"
                                            id="sessions-until-long-break"
                                            min="2"
                                            max="8"
                                            prop:value=sessions_until_long_break
                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            on:input=move |ev| {
                                                if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                                                    set_sessions_until_long_break.set(val.clamp(2, 8));
                                                }
                                            }
                                        />
                                    </div>

                                    <div>
                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=enable_screen_blocking
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_enable_screen_blocking.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Enable Screen Blocking During Sessions"</span>
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Show>

                    <Show when=move || active_tab.get() == "audio">
                        <div class="space-y-6">
                            <div>
                                <h3 class="text-lg font-medium text-gray-900 mb-4">"Audio Settings"</h3>
                                
                                <div class="space-y-4">
                                    <div>
                                        <label for="volume" class="block text-sm font-medium text-gray-700 mb-1">
                                            "Volume: " {move || format!("{}%", (volume.get() * 100.0).round() as u32)}
                                        </label>
                                        <input
                                            type="range"
                                            id="volume"
                                            min="0"
                                            max="1"
                                            step="0.01"
                                            prop:value=volume
                                            disabled=move || muted.get()
                                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer disabled:opacity-50"
                                            on:input=move |ev| {
                                                if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                                    set_volume.set(val.clamp(0.0, 1.0));
                                                }
                                            }
                                        />
                                    </div>

                                    <div class="space-y-3">
                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=muted
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_muted.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Mute All Sounds"</span>
                                        </label>

                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=enable_background_audio
                                                disabled=move || muted.get()
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500 disabled:opacity-50"
                                                on:change=move |ev| set_enable_background_audio.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Enable Background Audio During Work Sessions"</span>
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Show>

                    <Show when=move || active_tab.get() == "notifications">
                        <div class="space-y-6">
                            <div>
                                <h3 class="text-lg font-medium text-gray-900 mb-4">"Notification Preferences"</h3>
                                
                                <div class="space-y-4">
                                    <div class="space-y-3">
                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=enable_desktop_notifications
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_enable_desktop_notifications.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Enable Desktop Notifications"</span>
                                        </label>

                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=show_phase_transition_notifications
                                                disabled=move || !enable_desktop_notifications.get()
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500 disabled:opacity-50"
                                                on:change=move |ev| set_show_phase_transition_notifications.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Show Phase Transition Notifications"</span>
                                        </label>

                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=show_task_completion_notifications
                                                disabled=move || !enable_desktop_notifications.get()
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500 disabled:opacity-50"
                                                on:change=move |ev| set_show_task_completion_notifications.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Show Task Completion Notifications"</span>
                                        </label>
                                    </div>

                                    <div>
                                        <label for="auto-dismiss-delay" class="block text-sm font-medium text-gray-700 mb-1">
                                            "Auto-dismiss After (seconds)"
                                        </label>
                                        <input
                                            type="number"
                                            id="auto-dismiss-delay"
                                            min="1"
                                            max="30"
                                            prop:value=auto_dismiss_delay_seconds
                                            disabled=move || !enable_desktop_notifications.get()
                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
                                            on:input=move |ev| {
                                                if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                                    set_auto_dismiss_delay_seconds.set(val.clamp(1, 30));
                                                }
                                            }
                                        />
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Show>

                    <Show when=move || active_tab.get() == "interface">
                        <div class="space-y-6">
                            <div>
                                <h3 class="text-lg font-medium text-gray-900 mb-4">"Interface Preferences"</h3>
                                
                                <div class="space-y-4">
                                    <div>
                                        <label for="theme" class="block text-sm font-medium text-gray-700 mb-1">
                                            "Theme"
                                        </label>
                                        <select 
                                            id="theme" 
                                            prop:value=theme
                                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            on:change=move |ev| set_theme.set(event_target_value(&ev))
                                        >
                                            <option value="Light">"Light"</option>
                                            <option value="Dark">"Dark"</option>
                                            <option value="System">"System"</option>
                                        </select>
                                    </div>

                                    <div class="space-y-3">
                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=show_seconds_in_display
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_show_seconds_in_display.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Show Seconds in Timer Display"</span>
                                        </label>

                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=always_on_top
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_always_on_top.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Always on Top"</span>
                                        </label>

                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=compact_mode
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_compact_mode.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Compact Mode"</span>
                                        </label>

                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=show_task_list_sidebar
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_show_task_list_sidebar.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Show Task List Sidebar"</span>
                                        </label>

                                        <label class="flex items-center">
                                            <input
                                                type="checkbox"
                                                prop:checked=animate_progress
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                on:change=move |ev| set_animate_progress.set(event_target_checked(&ev))
                                            />
                                            <span class="ml-2 text-sm text-gray-700">"Animate Progress Ring"</span>
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Show>
                </div>

                // Action buttons
                <div class="border-t border-gray-200 px-6 py-4 bg-gray-50 flex justify-between">
                    <button 
                        class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        on:click=reset_to_defaults
                        disabled=move || loading.get()
                    >
                        "Reset to Defaults"
                    </button>
                    <button 
                        class="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
                        on:click=save_config
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Saving..." } else { "Save Settings" }}
                    </button>
                </div>
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