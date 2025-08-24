use leptos::prelude::*;
use crate::pages::settings::SettingsViewModel;
use crate::utils::ViewModel;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let vm = StoredValue::new(SettingsViewModel::new());

    view! {
        <div class="settings-section">
            <h1 class="section-title">"Settings"</h1>

            {move || {
                vm.with_value(|v| v.get_config()).map(|config| {
                    // Extract values to avoid move issues
                    let auto_start_breaks = config.general.auto_start_breaks;
                    let auto_start_work = config.general.auto_start_work_after_break;
                    let minimize_tray = config.general.minimize_to_tray;
                    let start_min = config.general.start_minimized;
                    let desktop_notif = config.notification.enable_desktop_notifications;
                    let sound_notif = config.notification.enable_sound_notifications;
                    let volume = config.audio.volume;
                    let theme = config.appearance.theme.clone();

                    view! {
                        <div class="settings-container">
                            <div class="setting-group">
                                <h3>"General Settings"</h3>
                                <div class="setting-item">
                                    <label>
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
                                        " Auto-start breaks"
                                    </label>
                                </div>
                                <div class="setting-item">
                                    <label>
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
                                        " Auto-start work after break"
                                    </label>
                                </div>
                                <div class="setting-item">
                                    <label>
                                        <input
                                            type="checkbox"
                                            checked=minimize_tray
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
                                        " Minimize to system tray"
                                    </label>
                                </div>
                                <div class="setting-item">
                                    <label>
                                        <input
                                            type="checkbox"
                                            checked=start_min
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
                                        " Start minimized"
                                    </label>
                                </div>
                            </div>

                            <div class="setting-group">
                                <h3>"Notifications"</h3>
                                <div class="setting-item">
                                    <label>
                                        <input
                                            type="checkbox"
                                            checked=desktop_notif
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
                                        " Desktop notifications"
                                    </label>
                                </div>
                                <div class="setting-item">
                                    <label>
                                        <input
                                            type="checkbox"
                                            checked=sound_notif
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
                                        " Sound notifications"
                                    </label>
                                </div>
                            </div>

                            <div class="setting-group">
                                <h3>"Audio"</h3>
                                <div class="setting-item">
                                    <label>"Volume"</label>
                                    <input
                                        type="range"
                                        min="0"
                                        max="100"
                                        value=(volume * 100.0) as i32
                                        on:change=move |ev| {
                                            let value = event_target_value(&ev).parse::<f32>().unwrap_or(50.0) / 100.0;
                                            vm.with_value(|v| {
                                                if let Some(mut cfg) = v.get_config() {
                                                    cfg.audio.volume = value;
                                                    v.update_audio(cfg.audio);
                                                }
                                            });
                                        }
                                    />
                                </div>
                            </div>

                            <div class="setting-group">
                                <h3>"Appearance"</h3>
                                <div class="setting-item">
                                    <label>"Theme"</label>
                                    <select
                                        on:change=move |ev| {
                                            let value = event_target_value(&ev);
                                            vm.with_value(|v| {
                                                if let Some(mut cfg) = v.get_config() {
                                                    cfg.appearance.theme = match value.as_str() {
                                                        "dark" => domain::Theme::Dark,
                                                        "light" => domain::Theme::Light,
                                                        _ => domain::Theme::System,
                                                    };
                                                    v.update_appearance(cfg.appearance);
                                                }
                                            });
                                        }
                                    >
                                        <option value="light" selected=theme == domain::Theme::Light>"Light"</option>
                                        <option value="dark" selected=theme == domain::Theme::Dark>"Dark"</option>
                                        <option value="system" selected=theme == domain::Theme::System>"System"</option>
                                    </select>
                                </div>
                            </div>

                            <div class="setting-group">
                                <button
                                    class="btn btn-secondary"
                                    on:click=move |_| {
                                        vm.with_value(|v| v.reset_to_defaults());
                                    }
                                    disabled=move || vm.with_value(|v| v.is_saving())
                                >
                                    "Reset to Defaults"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }).unwrap_or_else(|| {
                    view! {
                        <div class="settings-container">
                            <p>"Loading settings..."</p>
                        </div>
                    }.into_any()
                })
            }}
        </div>
    }
}