use leptos::prelude::*;
use domain::{TaskId, TaskSettings};
use std::rc::Rc;

#[component]
pub fn TaskSettingsModal(
    _task_id: TaskId,
    task_name: String,
    settings: Option<TaskSettings>,
    on_save: impl Fn(TaskSettings) + 'static,
    on_close: impl Fn() + 'static,
) -> impl IntoView {
    let (use_global, set_use_global) = signal(
        settings.as_ref().map_or(true, |s| s.use_global_settings)
    );
    
    let (work_minutes, set_work_minutes) = signal(
        settings.as_ref()
            .and_then(|s| s.custom_work_duration)
            .map(|d| (d.as_secs() / 60) as u32)
            .unwrap_or(25)
    );
    
    let (short_break_minutes, set_short_break_minutes) = signal(
        settings.as_ref()
            .and_then(|s| s.custom_short_break_duration)
            .map(|d| (d.as_secs() / 60) as u32)
            .unwrap_or(5)
    );
    
    let (long_break_minutes, set_long_break_minutes) = signal(
        settings.as_ref()
            .and_then(|s| s.custom_long_break_duration)
            .map(|d| (d.as_secs() / 60) as u32)
            .unwrap_or(15)
    );
    
    let (sessions_until_long_break, set_sessions_until_long_break) = signal(
        settings.as_ref()
            .and_then(|s| s.custom_sessions_until_long_break)
            .unwrap_or(4)
    );
    
    let (max_sessions, set_max_sessions) = signal(
        settings.as_ref()
            .and_then(|s| s.custom_max_sessions)
            .unwrap_or(4)
    );
    
    let (enable_screen_blocking, set_enable_screen_blocking) = signal(
        settings.as_ref()
            .and_then(|s| s.custom_enable_screen_blocking)
            .unwrap_or(false)
    );

    let handle_save = move |_| {
        let mut new_settings = TaskSettings::default();
        new_settings.use_global_settings = use_global.get();
        
        if !use_global.get() {
            use std::time::Duration;
            new_settings.custom_work_duration = Some(Duration::from_secs(work_minutes.get() as u64 * 60));
            new_settings.custom_short_break_duration = Some(Duration::from_secs(short_break_minutes.get() as u64 * 60));
            new_settings.custom_long_break_duration = Some(Duration::from_secs(long_break_minutes.get() as u64 * 60));
            new_settings.custom_sessions_until_long_break = Some(sessions_until_long_break.get());
            new_settings.custom_max_sessions = Some(max_sessions.get());
            new_settings.custom_enable_screen_blocking = Some(enable_screen_blocking.get());
        }
        
        on_save(new_settings);
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
                            <label>"Work Duration (minutes):"</label>
                            <input
                                type="number"
                                min="1"
                                max="60"
                                value=work_minutes
                                disabled=use_global
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                        set_work_minutes.set(val);
                                    }
                                }
                            />
                        </div>
                        
                        <div class="form-group">
                            <label>"Short Break (minutes):"</label>
                            <input
                                type="number"
                                min="1"
                                max="30"
                                value=short_break_minutes
                                disabled=use_global
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                        set_short_break_minutes.set(val);
                                    }
                                }
                            />
                        </div>
                        
                        <div class="form-group">
                            <label>"Long Break (minutes):"</label>
                            <input
                                type="number"
                                min="5"
                                max="60"
                                value=long_break_minutes
                                disabled=use_global
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                        set_long_break_minutes.set(val);
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
pub fn TaskSettingsIndicator(
    has_custom_settings: bool,
) -> impl IntoView {
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