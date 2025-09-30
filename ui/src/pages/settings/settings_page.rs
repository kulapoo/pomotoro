use crate::pages::settings::SettingsViewModel;
use crate::utils::ViewModel;
use leptos::prelude::*;
use leptos::task::spawn_local;
use domain::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let vm = SettingsViewModel::new();
    let (active_tab, set_active_tab) = signal("timer");
    let (validation_errors, set_validation_errors) = signal(Vec::<String>::new());
    let (success_message, set_success_message) = signal(None::<String>);
    
    // Store VM in a StoredValue for access in closures
    let vm_stored = StoredValue::new(vm);

    let handle_save = move |_| {
        set_validation_errors.set(Vec::new());
        set_success_message.set(None);
        
        vm_stored.with_value(|v| {
            if let Err(e) = v.save_settings() {
                set_validation_errors.update(|errors| errors.push(e.to_string()));
            } else {
                set_success_message.set(Some("Settings saved successfully".to_string()));
                spawn_local(async move {
                    leptos::prelude::set_timeout(
                        move || set_success_message.set(None),
                        std::time::Duration::from_secs(3)
                    );
                });
            }
        });
    };

    let handle_reset = move |_| {
        set_validation_errors.set(Vec::new());
        set_success_message.set(None);
        
        vm_stored.with_value(|v| {
            v.reset_to_defaults();
            set_success_message.set(Some("Settings reset to defaults".to_string()));
            spawn_local(async move {
                leptos::prelude::set_timeout(
                    move || set_success_message.set(None),
                    std::time::Duration::from_secs(3)
                );
            });
        });
    };

    let handle_export = move |_| {
        vm_stored.with_value(|v| {
            v.export_settings();
            set_success_message.set(Some("Settings exported successfully".to_string()));
            spawn_local(async move {
                leptos::prelude::set_timeout(
                    move || set_success_message.set(None),
                    std::time::Duration::from_secs(3)
                );
            });
        });
    };

    let handle_import = move |_| {
        vm_stored.with_value(|v| {
            if let Err(e) = v.import_settings() {
                set_validation_errors.update(|errors| errors.push(e.to_string()));
            } else {
                set_success_message.set(Some("Settings imported successfully".to_string()));
                spawn_local(async move {
                    leptos::prelude::set_timeout(
                        move || set_success_message.set(None),
                        std::time::Duration::from_secs(3)
                    );
                });
            }
        });
    };

    view! {
        <div class="settings-page-wrapper">
        <div class="settings-container">
            <div class="settings-header">
                <h2 class="settings-title">"Global Settings"</h2>
                <div class="settings-actions">
                    <button class="btn btn-secondary" on:click=handle_export>"Export"</button>
                    <button class="btn btn-secondary" on:click=handle_import>"Import"</button>
                    <button class="btn btn-secondary" on:click=handle_reset>"Reset to Defaults"</button>
                </div>
            </div>

            <Show when=move || !validation_errors.get().is_empty()>
                <div class="settings-errors">
                    <For
                        each=move || validation_errors.get()
                        key=|error| error.clone()
                        children=move |error| view! {
                            <div class="error-message">{error}</div>
                        }
                    />
                </div>
            </Show>

            <Show when=move || success_message.get().is_some()>
                <div class="success-message">
                    {move || success_message.get().unwrap_or_default()}
                </div>
            </Show>

            <div class="settings-tabs">
                <button
                    class=move || if active_tab.get() == "timer" { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_tab.set("timer")
                >
                    "Timer"
                </button>
                <button
                    class=move || if active_tab.get() == "notifications" { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_tab.set("notifications")
                >
                    "Notifications"
                </button>
                <button
                    class=move || if active_tab.get() == "audio" { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_tab.set("audio")
                >
                    "Audio"
                </button>
                // Appearance tab disabled - using light theme only
                <button
                    class=move || if active_tab.get() == "general" { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_tab.set("general")
                >
                    "General"
                </button>
                <button
                    class=move || if active_tab.get() == "storage" { "tab-button active" } else { "tab-button" }
                    on:click=move |_| set_active_tab.set("storage")
                >
                    "Storage"
                </button>
            </div>

            <div class="settings-content">
                {move || {
                    let config_opt = vm_stored.with_value(|v| v.config.get());
                    match config_opt {
                        Some(config) => {
                            match active_tab.get() {
                                "timer" => view! { <TimerSettings config=config vm=vm_stored /> }.into_any(),
                                "notifications" => view! { <NotificationSettings config=config vm=vm_stored /> }.into_any(),
                                "audio" => view! { <AudioSettings config=config vm=vm_stored /> }.into_any(),
                                // "appearance" => disabled - using light theme only
                                "general" => view! { <GeneralSettings config=config vm=vm_stored /> }.into_any(),
                                "storage" => view! { <StorageSettings vm=vm_stored /> }.into_any(),
                                _ => view! { <TimerSettings config=config vm=vm_stored /> }.into_any()
                            }
                        },
                        None => {
                            view! {
                                <div class="settings-loading">"Loading settings..."</div>
                            }.into_any()
                        }
                    }
                }}
            </div>

            <div class="settings-footer">
                <button class="btn btn-cancel" on:click=move |_| {
                    vm_stored.with_value(|v| v.refetch_config());
                    set_success_message.set(Some("Changes discarded".to_string()));
                }>"Cancel"</button>
                <button class="btn btn-primary" on:click=handle_save>"Save All Settings"</button>
            </div>
        </div>
        </div>
    }
}


#[component]
fn TimerSettings(
    #[allow(unused)] config: Config,
    vm: StoredValue<SettingsViewModel>
) -> impl IntoView {
    let work_duration = move || vm.with_value(|v| {
        v.get_config().map(|c| c.timer.work_duration.as_secs() / 60).unwrap_or(25)
    });
    let short_break_duration = move || vm.with_value(|v| {
        v.get_config().map(|c| c.timer.short_break_duration.as_secs() / 60).unwrap_or(5)
    });
    let long_break_duration = move || vm.with_value(|v| {
        v.get_config().map(|c| c.timer.long_break_duration.as_secs() / 60).unwrap_or(15)
    });
    let sessions_until_long_break = move || vm.with_value(|v| {
        v.get_config().map(|c| c.timer.sessions_until_long_break).unwrap_or(4)
    });

    view! {
        <div class="settings-section">
            <h3 class="section-title">"Timer Settings"</h3>

            <div class="setting-group">
                <label class="setting-label">"Work Duration (minutes)"</label>
                <input
                    type="number"
                    class="setting-input"
                    value=work_duration
                    min="1"
                    max="90"
                    on:input=move |ev| {
                        let value = event_target_value(&ev).parse::<u64>().unwrap_or(25);
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.timer.work_duration = std::time::Duration::from_secs(value * 60);
                                v.update_timer(cfg.timer);
                            }
                        });
                    }
                />
                <span class="setting-help">"Duration of work sessions (1-90 minutes)"</span>
            </div>

            <div class="setting-group">
                <label class="setting-label">"Short Break Duration (minutes)"</label>
                <input
                    type="number"
                    class="setting-input"
                    value=short_break_duration
                    min="1"
                    max="30"
                    on:input=move |ev| {
                        let value = event_target_value(&ev).parse::<u64>().unwrap_or(5);
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.timer.short_break_duration = std::time::Duration::from_secs(value * 60);
                                v.update_timer(cfg.timer);
                            }
                        });
                    }
                />
                <span class="setting-help">"Duration of short breaks (1-30 minutes)"</span>
            </div>

            <div class="setting-group">
                <label class="setting-label">"Long Break Duration (minutes)"</label>
                <input
                    type="number"
                    class="setting-input"
                    value=long_break_duration
                    min="5"
                    max="60"
                    on:input=move |ev| {
                        let value = event_target_value(&ev).parse::<u64>().unwrap_or(15);
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.timer.long_break_duration = std::time::Duration::from_secs(value * 60);
                                v.update_timer(cfg.timer);
                            }
                        });
                    }
                />
                <span class="setting-help">"Duration of long breaks (5-60 minutes)"</span>
            </div>

            <div class="setting-group">
                <label class="setting-label">"Sessions Until Long Break"</label>
                <input
                    type="number"
                    class="setting-input"
                    value=sessions_until_long_break
                    min="2"
                    max="10"
                    on:input=move |ev| {
                        let value = event_target_value(&ev).parse::<u8>().unwrap_or(4);
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.timer.sessions_until_long_break = value;
                                v.update_timer(cfg.timer);
                            }
                        });
                    }
                />
                <span class="setting-help">"Number of work sessions before a long break (2-10)"</span>
            </div>
        </div>
    }
}

#[component]
fn NotificationSettings(
    #[allow(unused)] config: Config,
    vm: StoredValue<SettingsViewModel>
) -> impl IntoView {
    let enable_desktop = move || vm.with_value(|v| {
        v.get_config().map(|c| c.notification.enable_desktop_notifications).unwrap_or(true)
    });
    let enable_sound = move || vm.with_value(|v| {
        v.get_config().map(|c| c.notification.enable_sound_notifications).unwrap_or(true)
    });
    let show_phase_transitions = move || vm.with_value(|v| {
        v.get_config().map(|c| c.notification.show_phase_transition_notifications).unwrap_or(true)
    });
    let show_task_completions = move || vm.with_value(|v| {
        v.get_config().map(|c| c.notification.show_task_completion_notifications).unwrap_or(true)
    });
    let auto_dismiss_delay = move || vm.with_value(|v| {
        v.get_config().map(|c| c.notification.auto_dismiss_delay_seconds).unwrap_or(5)
    });
    let is_position_top_right = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.notification.notification_position, NotificationPosition::TopRight)).unwrap_or(true)
    });
    let is_position_top_left = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.notification.notification_position, NotificationPosition::TopLeft)).unwrap_or(false)
    });
    let is_position_bottom_right = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.notification.notification_position, NotificationPosition::BottomRight)).unwrap_or(false)
    });
    let is_position_bottom_left = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.notification.notification_position, NotificationPosition::BottomLeft)).unwrap_or(false)
    });
    let is_position_center = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.notification.notification_position, NotificationPosition::Center)).unwrap_or(false)
    });

    view! {
        <div class="settings-section">
            <h3 class="section-title">"Notification Settings"</h3>
            
            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=enable_desktop
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.notification.enable_desktop_notifications = checked;
                                    v.update_notifications(cfg.notification);
                                }
                            });
                        }
                    />
                    <span>"Enable Desktop Notifications"</span>
                </label>
                <span class="setting-help">"Show system notifications for timer events"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=enable_sound
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.notification.enable_sound_notifications = checked;
                                    v.update_notifications(cfg.notification);
                                }
                            });
                        }
                    />
                    <span>"Enable Sound Notifications"</span>
                </label>
                <span class="setting-help">"Play sounds for timer events"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=show_phase_transitions
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.notification.show_phase_transition_notifications = checked;
                                    v.update_notifications(cfg.notification);
                                }
                            });
                        }
                    />
                    <span>"Show Phase Transition Notifications"</span>
                </label>
                <span class="setting-help">"Notify when switching between work and break phases"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=show_task_completions
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.notification.show_task_completion_notifications = checked;
                                    v.update_notifications(cfg.notification);
                                }
                            });
                        }
                    />
                    <span>"Show Task Completion Notifications"</span>
                </label>
                <span class="setting-help">"Notify when tasks are completed"</span>
            </div>

            <div class="setting-group">
                <label class="setting-label">"Auto-Dismiss Delay (seconds)"</label>
                <input
                    type="number"
                    class="setting-input"
                    value=auto_dismiss_delay
                    min="1"
                    max="300"
                    on:input=move |ev| {
                        let value = event_target_value(&ev).parse::<u32>().unwrap_or(5);
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.notification.auto_dismiss_delay_seconds = value;
                                v.update_notifications(cfg.notification);
                            }
                        });
                    }
                />
                <span class="setting-help">"Time before notifications automatically close"</span>
            </div>

            <div class="setting-group">
                <label class="setting-label">"Notification Position"</label>
                <select
                    class="setting-select"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        let position = match value.as_str() {
                            "TopLeft" => NotificationPosition::TopLeft,
                            "TopRight" => NotificationPosition::TopRight,
                            "BottomLeft" => NotificationPosition::BottomLeft,
                            "BottomRight" => NotificationPosition::BottomRight,
                            "Center" => NotificationPosition::Center,
                            _ => NotificationPosition::TopRight,
                        };
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.notification.notification_position = position;
                                v.update_notifications(cfg.notification);
                            }
                        });
                    }
                >
                    <option value="TopRight" selected=is_position_top_right>"Top Right"</option>
                    <option value="TopLeft" selected=is_position_top_left>"Top Left"</option>
                    <option value="BottomRight" selected=is_position_bottom_right>"Bottom Right"</option>
                    <option value="BottomLeft" selected=is_position_bottom_left>"Bottom Left"</option>
                    <option value="Center" selected=is_position_center>"Center"</option>
                </select>
                <span class="setting-help">"Where notifications appear on screen"</span>
            </div>
        </div>
    }
}

#[component]
fn AudioSettings(
    #[allow(unused)] config: Config,
    vm: StoredValue<SettingsViewModel>
) -> impl IntoView {
    let volume = move || vm.with_value(|v| {
        v.get_config().map(|c| (c.audio.volume * 100.0) as u32).unwrap_or(50)
    });
    let enable_background = move || vm.with_value(|v| {
        v.get_config().map(|c| c.audio.enable_background_audio).unwrap_or(false)
    });
    let muted = move || vm.with_value(|v| {
        v.get_config().map(|c| c.audio.muted).unwrap_or(false)
    });
    let audio_enabled = move || !muted();
    let has_no_work_sound = move || vm.with_value(|v| {
        v.get_config().map(|c| c.audio.work_notification_sound.is_none()).unwrap_or(true)
    });
    let has_no_break_sound = move || vm.with_value(|v| {
        v.get_config().map(|c| c.audio.break_notification_sound.is_none()).unwrap_or(true)
    });
    let has_no_bg_sound = move || vm.with_value(|v| {
        v.get_config().map(|c| c.audio.background_sound.is_none()).unwrap_or(true)
    });

    view! {
        <div class="settings-section">
            <h3 class="section-title">"Audio Settings"</h3>
            
            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=audio_enabled
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.audio.muted = !checked;
                                    v.update_audio(cfg.audio);
                                }
                            });
                        }
                    />
                    <span>"Enable Audio"</span>
                </label>
                <span class="setting-help">"Master audio toggle"</span>
            </div>

            <div class="setting-group">
                <label class="setting-label">"Volume"</label>
                <div class="volume-control">
                    <input
                        type="range"
                        class="setting-slider"
                        value=volume
                        min="0"
                        max="100"
                        on:input=move |ev| {
                            let value = event_target_value(&ev).parse::<u32>().unwrap_or(70);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.audio.volume = (value as f32) / 100.0;
                                    v.update_audio(cfg.audio);
                                }
                            });
                        }
                    />
                    <span class="volume-value">{volume}"%"</span>
                </div>
                <span class="setting-help">"Master volume for all sounds"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=enable_background
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.audio.enable_background_audio = checked;
                                    v.update_audio(cfg.audio);
                                }
                            });
                        }
                    />
                    <span>"Enable Background Audio"</span>
                </label>
                <span class="setting-help">"Play ambient sounds during work sessions"</span>
            </div>

            <div class="setting-group">
                <label class="setting-label">"Work Notification Sound"</label>
                <select
                    class="setting-select"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        let sound = if value.is_empty() { None } else { Some(value) };
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.audio.work_notification_sound = sound;
                                v.update_audio(cfg.audio);
                            }
                        });
                    }
                >
                    <option value="" selected=has_no_work_sound>"None"</option>
                    <option value="bell.wav">"Bell"</option>
                    <option value="chime.wav">"Chime"</option>
                    <option value="gong.wav">"Gong"</option>
                </select>
                <button class="btn btn-small" on:click=move |_| {
                    vm.with_value(|v| v.test_audio_preview("work"));
                }>"Test"</button>
                <span class="setting-help">"Sound played when work session ends"</span>
            </div>

            <div class="setting-group">
                <label class="setting-label">"Break Notification Sound"</label>
                <select
                    class="setting-select"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        let sound = if value.is_empty() { None } else { Some(value) };
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.audio.break_notification_sound = sound;
                                v.update_audio(cfg.audio);
                            }
                        });
                    }
                >
                    <option value="" selected=has_no_break_sound>"None"</option>
                    <option value="bell.wav">"Bell"</option>
                    <option value="chime.wav">"Chime"</option>
                    <option value="gong.wav">"Gong"</option>
                </select>
                <button class="btn btn-small" on:click=move |_| {
                    vm.with_value(|v| v.test_audio_preview("break"));
                }>"Test"</button>
                <span class="setting-help">"Sound played when break session ends"</span>
            </div>

            <div class="setting-group">
                <label class="setting-label">"Background Sound"</label>
                <select
                    class="setting-select"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        let sound = if value.is_empty() { None } else { Some(value) };
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.audio.background_sound = sound;
                                v.update_audio(cfg.audio);
                            }
                        });
                    }
                >
                    <option value="" selected=has_no_bg_sound>"None"</option>
                    <option value="rain.wav">"Rain"</option>
                    <option value="forest.wav">"Forest"</option>
                    <option value="ocean.wav">"Ocean"</option>
                    <option value="whitenoise.wav">"White Noise"</option>
                </select>
                <button class="btn btn-small" on:click=move |_| {
                    vm.with_value(|v| v.test_audio_preview("background"));
                }>"Test"</button>
                <span class="setting-help">"Ambient sound during work sessions"</span>
            </div>
        </div>
    }
}

#[component]
fn AppearanceSettings(
    #[allow(unused)] config: Config,
    vm: StoredValue<SettingsViewModel>
) -> impl IntoView {
    let show_seconds = move || vm.with_value(|v| {
        v.get_config().map(|c| c.appearance.show_seconds_in_display).unwrap_or(true)
    });
    let always_on_top = move || vm.with_value(|v| {
        v.get_config().map(|c| c.appearance.always_on_top).unwrap_or(false)
    });
    let compact_mode = move || vm.with_value(|v| {
        v.get_config().map(|c| c.appearance.compact_mode).unwrap_or(false)
    });
    let show_sidebar = move || vm.with_value(|v| {
        v.get_config().map(|c| c.appearance.show_task_list_sidebar).unwrap_or(true)
    });
    let animate_progress = move || vm.with_value(|v| {
        v.get_config().map(|c| c.appearance.animate_progress).unwrap_or(true)
    });
    let is_theme_system = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.appearance.theme, Theme::System)).unwrap_or(true)
    });
    let is_theme_light = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.appearance.theme, Theme::Light)).unwrap_or(false)
    });
    let is_theme_dark = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.appearance.theme, Theme::Dark)).unwrap_or(false)
    });

    view! {
        <div class="settings-section">
            <h3 class="section-title">"Appearance Settings"</h3>
            
            <div class="setting-group">
                <label class="setting-label">"Theme"</label>
                <select
                    class="setting-select"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        let theme = match value.as_str() {
                            "Light" => Theme::Light,
                            "Dark" => Theme::Dark,
                            _ => Theme::System,
                        };
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.appearance.theme = theme;
                                v.update_appearance(cfg.appearance);
                            }
                        });
                    }
                >
                    <option value="System" selected=is_theme_system>"System"</option>
                    <option value="Light" selected=is_theme_light>"Light"</option>
                    <option value="Dark" selected=is_theme_dark>"Dark"</option>
                </select>
                <span class="setting-help">"Application color scheme"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=show_seconds
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.appearance.show_seconds_in_display = checked;
                                    v.update_appearance(cfg.appearance);
                                }
                            });
                        }
                    />
                    <span>"Show Seconds in Timer"</span>
                </label>
                <span class="setting-help">"Display seconds in the timer countdown"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=always_on_top
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.appearance.always_on_top = checked;
                                    v.update_appearance(cfg.appearance);
                                }
                            });
                        }
                    />
                    <span>"Always On Top"</span>
                </label>
                <span class="setting-help">"Keep window above other applications"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=compact_mode
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.appearance.compact_mode = checked;
                                    v.update_appearance(cfg.appearance);
                                }
                            });
                        }
                    />
                    <span>"Compact Mode"</span>
                </label>
                <span class="setting-help">"Use minimal interface layout"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=show_sidebar
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.appearance.show_task_list_sidebar = checked;
                                    v.update_appearance(cfg.appearance);
                                }
                            });
                        }
                    />
                    <span>"Show Task List Sidebar"</span>
                </label>
                <span class="setting-help">"Display task list in sidebar"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=animate_progress
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.appearance.animate_progress = checked;
                                    v.update_appearance(cfg.appearance);
                                }
                            });
                        }
                    />
                    <span>"Animate Progress"</span>
                </label>
                <span class="setting-help">"Show smooth animations for progress indicators"</span>
            </div>
        </div>
    }
}

#[component]
fn GeneralSettings(
    #[allow(unused)] config: Config,
    vm: StoredValue<SettingsViewModel>
) -> impl IntoView {
    let auto_start_breaks = move || vm.with_value(|v| {
        v.get_config().map(|c| c.general.auto_start_breaks).unwrap_or(true)
    });
    let auto_start_work = move || vm.with_value(|v| {
        v.get_config().map(|c| c.general.auto_start_work_after_break).unwrap_or(false)
    });
    let minimize_to_tray = move || vm.with_value(|v| {
        v.get_config().map(|c| c.general.minimize_to_tray).unwrap_or(true)
    });
    let start_minimized = move || vm.with_value(|v| {
        v.get_config().map(|c| c.general.start_minimized).unwrap_or(false)
    });
    let is_cycling_manual = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.general.task_cycling_behavior, TaskCyclingBehavior::Manual)).unwrap_or(true)
    });
    let is_cycling_auto_advance = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.general.task_cycling_behavior, TaskCyclingBehavior::AutoAdvance)).unwrap_or(false)
    });
    let is_cycling_round_robin = move || vm.with_value(|v| {
        v.get_config().map(|c| matches!(c.general.task_cycling_behavior, TaskCyclingBehavior::RoundRobin)).unwrap_or(false)
    });

    view! {
        <div class="settings-section">
            <h3 class="section-title">"General Settings"</h3>
            
            <div class="setting-group">
                <label class="setting-label">"Task Cycling Behavior"</label>
                <select
                    class="setting-select"
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        let behavior = match value.as_str() {
                            "AutoAdvance" => TaskCyclingBehavior::AutoAdvance,
                            "RoundRobin" => TaskCyclingBehavior::RoundRobin,
                            _ => TaskCyclingBehavior::Manual,
                        };
                        vm.with_value(|v| {
                            if let Some(mut cfg) = v.get_config() {
                                cfg.general.task_cycling_behavior = behavior;
                                v.update_general(cfg.general);
                            }
                        });
                    }
                >
                    <option value="Manual" selected=is_cycling_manual>"Manual"</option>
                    <option value="AutoAdvance" selected=is_cycling_auto_advance>"Auto Advance"</option>
                    <option value="RoundRobin" selected=is_cycling_round_robin>"Round Robin"</option>
                </select>
                <span class="setting-help">"How tasks cycle after completion"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=auto_start_breaks
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.general.auto_start_breaks = checked;
                                    v.update_general(cfg.general);
                                }
                            });
                        }
                    />
                    <span>"Auto-Start Breaks"</span>
                </label>
                <span class="setting-help">"Automatically start break sessions after work"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=auto_start_work
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.general.auto_start_work_after_break = checked;
                                    v.update_general(cfg.general);
                                }
                            });
                        }
                    />
                    <span>"Auto-Start Work After Break"</span>
                </label>
                <span class="setting-help">"Automatically start work sessions after break"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=minimize_to_tray
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.general.minimize_to_tray = checked;
                                    v.update_general(cfg.general);
                                }
                            });
                        }
                    />
                    <span>"Minimize to System Tray"</span>
                </label>
                <span class="setting-help">"Hide to system tray when minimized"</span>
            </div>

            <div class="setting-group">
                <label class="setting-checkbox">
                    <input
                        type="checkbox"
                        checked=start_minimized
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            vm.with_value(|v| {
                                if let Some(mut cfg) = v.get_config() {
                                    cfg.general.start_minimized = checked;
                                    v.update_general(cfg.general);
                                }
                            });
                        }
                    />
                    <span>"Start Minimized"</span>
                </label>
                <span class="setting-help">"Launch application minimized"</span>
            </div>
        </div>
    }
}

#[component]
fn StorageSettings(
    vm: StoredValue<SettingsViewModel>
) -> impl IntoView {
    let (storage_path, set_storage_path) = signal(String::from(""));
    let (validation_error, set_validation_error) = signal(None::<String>);

    Effect::new(move |_| {
        vm.with_value(|v| {
            let path = v.get_storage_path();
            set_storage_path.set(path);
        });
    });

    view! {
        <div class="settings-section">
            <h3 class="section-title">"Storage Settings"</h3>
            
            <div class="setting-group">
                <label class="setting-label">"Data Directory"</label>
                <div class="path-input-group">
                    <input
                        type="text"
                        class="setting-input path-input"
                        value=move || storage_path.get()
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            set_storage_path.set(value.clone());
                            set_validation_error.set(None);
                        }
                    />
                    <button class="btn btn-secondary" on:click=move |_| {
                        vm.with_value(|v| {
                            if let Some(path) = v.browse_for_directory() {
                                set_storage_path.set(path);
                                set_validation_error.set(None);
                            }
                        });
                    }>"Browse"</button>
                </div>
                <span class="setting-help">"Location where all application data is stored"</span>
                <Show when=move || validation_error.get().is_some()>
                    <div class="validation-error">
                        {move || validation_error.get().unwrap_or_default()}
                    </div>
                </Show>
            </div>

            <div class="setting-group">
                <button class="btn btn-secondary" on:click=move |_| {
                    let path = storage_path.get();
                    vm.with_value(|v| {
                        match v.validate_storage_path(&path) {
                            Ok(_) => {
                                v.update_storage_path(path);
                                set_validation_error.set(None);
                            },
                            Err(e) => {
                                set_validation_error.set(Some(e.to_string()));
                            }
                        }
                    });
                }>"Apply Storage Path"</button>
                <span class="setting-help">"Change storage location (requires restart)"</span>
            </div>

            <div class="setting-group">
                <button class="btn btn-secondary" on:click=move |_| {
                    vm.with_value(|v| v.open_data_directory());
                }>"Open Data Directory"</button>
                <span class="setting-help">"Browse current data directory in file manager"</span>
            </div>

            <div class="setting-group">
                <button class="btn btn-warning" on:click=move |_| {
                    if leptos::prelude::window().confirm_with_message("This will delete all application data. Are you sure?").unwrap_or(false) {
                        vm.with_value(|v| v.clear_all_data());
                    }
                }>"Clear All Data"</button>
                <span class="setting-help">"Delete all tasks, settings, and history"</span>
            </div>
        </div>
    }
}
