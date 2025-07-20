use leptos::prelude::*;
use leptos::task::spawn_local;
use std::rc::Rc;
use crate::store::{ConfigResource, config::{update_default_timings}};

#[component]
pub fn TimerConfigComponent() -> impl IntoView {
    let config_resource = expect_context::<ConfigResource>();
    let config = config_resource.config;
    
    let work_minutes = Memo::new(move |_| {
        config.get().map(|c| c.task_config.work_duration.as_secs() / 60).unwrap_or(25)
    });
    
    let short_break_minutes = Memo::new(move |_| {
        config.get().map(|c| c.task_config.short_break_duration.as_secs() / 60).unwrap_or(5)
    });
    
    let long_break_minutes = Memo::new(move |_| {
        config.get().map(|c| c.task_config.long_break_duration.as_secs() / 60).unwrap_or(15)
    });
    
    let sessions_until_long_break = Memo::new(move |_| {
        config.get().map(|c| c.task_config.sessions_until_long_break).unwrap_or(4)
    });
    
    let enable_screen_blocking = Memo::new(move |_| {
        config.get().map(|c| c.task_config.enable_screen_blocking).unwrap_or(false)
    });
    
    let update_timing = Rc::new({
        let config_resource = config_resource.clone();
        move |work: u32, short: u32, long: u32| {
            let config_resource = config_resource.clone();
            spawn_local(async move {
                let _ = config_resource.update_and_refetch(move || {
                    update_default_timings(work, short, long)
                }).await;
            });
        }
    });
    view! {
        <div class="settings-section">
            <h3>"Timer Configuration"</h3>
            
            <div class="form-grid">
                <div class="form-group">
                    <label for="work-duration">"Work Duration (minutes)"</label>
                    <input
                        type="number"
                        id="work-duration"
                        min="5"
                        max="60"
                        prop:value=work_minutes
                        on:input={
                            let update_timing = update_timing.clone();
                            move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                    let clamped = val.clamp(5, 60);
                                    update_timing(clamped, short_break_minutes.get() as u32, long_break_minutes.get() as u32);
                                }
                            }
                        }
                    />
                    <small class="help-text">"5-60 minutes"</small>
                </div>

                <div class="form-group">
                    <label for="short-break-duration">"Short Break (minutes)"</label>
                    <input
                        type="number"
                        id="short-break-duration"
                        min="1"
                        max="30"
                        prop:value=short_break_minutes
                        on:input={
                            let update_timing = update_timing.clone();
                            move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                    let clamped = val.clamp(1, 30);
                                    update_timing(work_minutes.get() as u32, clamped, long_break_minutes.get() as u32);
                                }
                            }
                        }
                    />
                    <small class="help-text">"1-30 minutes"</small>
                </div>

                <div class="form-group">
                    <label for="long-break-duration">"Long Break (minutes)"</label>
                    <input
                        type="number"
                        id="long-break-duration"
                        min="5"
                        max="60"
                        prop:value=long_break_minutes
                        on:input={
                            let update_timing = update_timing.clone();
                            move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                    let clamped = val.clamp(5, 60);
                                    update_timing(work_minutes.get() as u32, short_break_minutes.get() as u32, clamped);
                                }
                            }
                        }
                    />
                    <small class="help-text">"5-60 minutes"</small>
                </div>

                <div class="form-group">
                    <label for="sessions-until-long-break">"Sessions Before Long Break"</label>
                    <input
                        type="number"
                        id="sessions-until-long-break"
                        min="1"
                        max="8"
                        prop:value=sessions_until_long_break
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<u8>() {
                                // Note: This would need a different update function for cycle length
                                // For now, just keeping it as display-only
                            }
                        }
                    />
                    <small class="help-text">"1-8 sessions"</small>
                </div>
            </div>

            <div class="form-group checkbox-group">
                <label class="checkbox-label">
                    <input
                        type="checkbox"
                        prop:checked=enable_screen_blocking
                        on:change=move |ev| {
                            // Note: This would need an update function for screen blocking
                            // For now, just keeping it as display-only
                        }
                    />
                    <span class="checkmark"></span>
                    "Enable Screen Blocking"
                </label>
                <small class="help-text">"Show full-screen overlay during work sessions"</small>
            </div>

            <div class="timer-preview">
                <h4>"Preview"</h4>
                <div class="preview-grid">
                    <div class="preview-item">
                        <span class="preview-label">"Work:"</span>
                        <span class="preview-value">{move || format!("{}:00", work_minutes.get())}</span>
                    </div>
                    <div class="preview-item">
                        <span class="preview-label">"Short Break:"</span>
                        <span class="preview-value">{move || format!("{}:00", short_break_minutes.get())}</span>
                    </div>
                    <div class="preview-item">
                        <span class="preview-label">"Long Break:"</span>
                        <span class="preview-value">{move || format!("{}:00", long_break_minutes.get())}</span>
                    </div>
                    <div class="preview-item">
                        <span class="preview-label">"Cycle:"</span>
                        <span class="preview-value">
                            {move || format!("{} sessions → long break", sessions_until_long_break.get())}
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}