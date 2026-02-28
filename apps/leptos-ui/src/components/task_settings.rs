use domain::{Config, GeneralConfig, TaskId, TimerConfiguration};
use leptos::prelude::*;
use std::rc::Rc;

#[component]
pub fn TaskSettingsModal(
    _task_id: TaskId,
    task_name: String,
    settings: Option<Config>,
    on_save: impl Fn(Config) + 'static,
    on_close: impl Fn() + 'static,
) -> impl IntoView {
    let (use_global, set_use_global) = signal(false);
    let (use_seconds_for_duration, set_use_seconds_for_duration) =
        signal(false);

    // Detect if we should use seconds mode based on stored values
    // If any duration is not divisible by 60, we should use seconds mode
    let initial_seconds_mode = settings
        .as_ref()
        .map(|s| {
            s.timer.work_duration.as_secs() % 60 != 0
                || s.timer.short_break_duration.as_secs() % 60 != 0
                || s.timer.long_break_duration.as_secs() % 60 != 0
        })
        .unwrap_or(false);

    if initial_seconds_mode {
        set_use_seconds_for_duration.set(true);
    }

    let (work_duration, set_work_duration) = signal(
        settings
            .as_ref()
            .map(|s| {
                if initial_seconds_mode {
                    s.timer.work_duration.as_secs() as u32
                } else {
                    (s.timer.work_duration.as_secs() / 60) as u32
                }
            })
            .unwrap_or(if initial_seconds_mode { 1500 } else { 25 }),
    );

    let (short_break_duration, set_short_break_duration) = signal(
        settings
            .as_ref()
            .map(|s| {
                if initial_seconds_mode {
                    s.timer.short_break_duration.as_secs() as u32
                } else {
                    (s.timer.short_break_duration.as_secs() / 60) as u32
                }
            })
            .unwrap_or(if initial_seconds_mode { 300 } else { 5 }),
    );

    let (long_break_duration, set_long_break_duration) = signal(
        settings
            .as_ref()
            .map(|s| {
                if initial_seconds_mode {
                    s.timer.long_break_duration.as_secs() as u32
                } else {
                    (s.timer.long_break_duration.as_secs() / 60) as u32
                }
            })
            .unwrap_or(if initial_seconds_mode { 900 } else { 15 }),
    );

    let (sessions_until_long_break, set_sessions_until_long_break) = signal(
        settings
            .as_ref()
            .map(|s| s.timer.sessions_until_long_break)
            .unwrap_or(4),
    );

    let (max_sessions, set_max_sessions) = signal(4);

    let (enable_screen_blocking, set_enable_screen_blocking) = signal(
        settings
            .as_ref()
            .map(|s| s.general.enable_screen_blocking)
            .unwrap_or(false),
    );

    let handle_save = move |_| {
        use std::time::Duration;
        let is_seconds_mode = use_seconds_for_duration.get();

        let work_secs = if is_seconds_mode {
            work_duration.get() as u64
        } else {
            work_duration.get() as u64 * 60
        };

        let short_break_secs = if is_seconds_mode {
            short_break_duration.get() as u64
        } else {
            short_break_duration.get() as u64 * 60
        };

        let long_break_secs = if is_seconds_mode {
            long_break_duration.get() as u64
        } else {
            long_break_duration.get() as u64 * 60
        };

        let new_config = Config {
            timer: TimerConfiguration {
                work_duration: Duration::from_secs(work_secs),
                short_break_duration: Duration::from_secs(short_break_secs),
                long_break_duration: Duration::from_secs(long_break_secs),
                sessions_until_long_break: sessions_until_long_break.get(),
            },
            audio: settings
                .as_ref()
                .map(|s| s.audio.clone())
                .unwrap_or_default(),
            general: GeneralConfig {
                enable_screen_blocking: enable_screen_blocking.get(),
                ..settings
                    .as_ref()
                    .map(|s| s.general.clone())
                    .unwrap_or_default()
            },
            notification: settings
                .as_ref()
                .map(|s| s.notification.clone())
                .unwrap_or_default(),
            appearance: settings
                .as_ref()
                .map(|s| s.appearance.clone())
                .unwrap_or_default(),
        };

        on_save(new_config);
    };

    let handle_reset_to_global = move |_| {
        set_use_global.set(true);
    };

    let on_close = Rc::new(on_close);
    let on_close_1 = Rc::clone(&on_close);
    let on_close_2 = Rc::clone(&on_close);
    let on_close_3 = Rc::clone(&on_close);

    view! {
        <div class="task-settings-modal">
            <div class="modal-overlay" on:click=move |_| on_close_1()></div>
            <div class="modal-content">
                <div class="modal-header">
                    <h2>"Task Settings: " {task_name}</h2>
                    <button class="close-button" on:click=move |_| on_close_2()>"×"</button>
                </div>

                <div class="modal-body">
                    <div class="settings-toggle">
                        <label>
                            <input
                                type="checkbox"
                                checked=use_global
                                on:change=move |ev| set_use_global.set(event_target_checked(&ev))
                            />
                            " Use global settings"
                        </label>
                        {move || if !use_global.get() {
                            view! {
                                <span class="custom-indicator">" (Using custom settings)"</span>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>

                    <div class="settings-form" class:disabled=use_global>
                        <div class="form-group">
                            <label>
                                <input
                                    type="checkbox"
                                    checked=use_seconds_for_duration
                                    disabled=use_global
                                    on:change=move |ev| set_use_seconds_for_duration.set(event_target_checked(&ev))
                                />
                                " Use seconds instead of minutes for durations"
                            </label>
                        </div>

                        <div class="form-group">
                            <label>
                                {move || if use_seconds_for_duration.get() { "Work Duration (seconds):" } else { "Work Duration (minutes):" }}
                            </label>
                            <input
                                type="number"
                                prop:min=move || if use_seconds_for_duration.get() { "5" } else { "1" }
                                prop:max=move || if use_seconds_for_duration.get() { "10800" } else { "180" }
                                value=work_duration
                                disabled=use_global
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                        set_work_duration.set(val);
                                    }
                                }
                            />
                        </div>

                        <div class="form-group">
                            <label>
                                {move || if use_seconds_for_duration.get() { "Short Break (seconds):" } else { "Short Break (minutes):" }}
                            </label>
                            <input
                                type="number"
                                prop:min=move || if use_seconds_for_duration.get() { "5" } else { "1" }
                                prop:max=move || if use_seconds_for_duration.get() { "3600" } else { "60" }
                                value=short_break_duration
                                disabled=use_global
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                        set_short_break_duration.set(val);
                                    }
                                }
                            />
                        </div>

                        <div class="form-group">
                            <label>
                                {move || if use_seconds_for_duration.get() { "Long Break (seconds):" } else { "Long Break (minutes):" }}
                            </label>
                            <input
                                type="number"
                                prop:min=move || if use_seconds_for_duration.get() { "5" } else { "1" }
                                prop:max=move || if use_seconds_for_duration.get() { "7200" } else { "120" }
                                value=long_break_duration
                                disabled=use_global
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                        set_long_break_duration.set(val);
                                    }
                                }
                            />
                        </div>

                        <div class="form-group">
                            <label>"Sessions Until Long Break:"</label>
                            <input
                                type="number"
                                min="2"
                                max="10"
                                value=sessions_until_long_break
                                disabled=use_global
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                                        set_sessions_until_long_break.set(val);
                                    }
                                }
                            />
                        </div>

                        <div class="form-group">
                            <label>"Max Sessions:"</label>
                            <input
                                type="number"
                                min="1"
                                max="10"
                                value=max_sessions
                                disabled=use_global
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                                        set_max_sessions.set(val);
                                    }
                                }
                            />
                        </div>

                        <div class="form-group">
                            <label>
                                <input
                                    type="checkbox"
                                    checked=enable_screen_blocking
                                    disabled=use_global
                                    on:change=move |ev| set_enable_screen_blocking.set(event_target_checked(&ev))
                                />
                                " Enable screen blocking during breaks"
                            </label>
                        </div>
                    </div>
                </div>

                <div class="modal-footer">
                    <button class="btn btn-secondary" on:click=move |_| on_close_3()>"Cancel"</button>
                    {move || if !use_global.get() {
                        view! {
                            <button class="btn btn-warning" on:click=handle_reset_to_global>"Reset to Global"</button>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                    <button class="btn btn-primary" on:click=handle_save>"Save"</button>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn TaskSettingsIndicator(has_custom_settings: bool) -> impl IntoView {
    view! {
        <div class="task-settings-indicator">
            {if has_custom_settings {
                view! {
                    <span class="custom-badge" title="Using custom settings">
                        <i class="icon-settings"></i>
                        " Custom"
                    </span>
                }
            } else {
                view! {
                    <span class="global-badge" title="Using global settings">
                        <i class="icon-globe"></i>
                        " Global"
                    </span>
                }
            }}
        </div>
    }
}
