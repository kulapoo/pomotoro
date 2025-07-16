use leptos::prelude::*;

#[component]
pub fn TimerConfigComponent(
    work_minutes: ReadSignal<u32>,
    set_work_minutes: WriteSignal<u32>,
    short_break_minutes: ReadSignal<u32>,
    set_short_break_minutes: WriteSignal<u32>,
    long_break_minutes: ReadSignal<u32>,
    set_long_break_minutes: WriteSignal<u32>,
    sessions_until_long_break: ReadSignal<u8>,
    set_sessions_until_long_break: WriteSignal<u8>,
    enable_screen_blocking: ReadSignal<bool>,
    set_enable_screen_blocking: WriteSignal<bool>,
) -> impl IntoView {
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
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                set_work_minutes.set(val.clamp(5, 60));
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
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                set_short_break_minutes.set(val.clamp(1, 30));
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
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                set_long_break_minutes.set(val.clamp(5, 60));
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
                                set_sessions_until_long_break.set(val.clamp(1, 8));
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
                        on:change=move |ev| set_enable_screen_blocking.set(event_target_checked(&ev))
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