use leptos::prelude::*;

#[component]
pub fn AudioConfigComponent(
    volume: ReadSignal<f32>,
    set_volume: WriteSignal<f32>,
    muted: ReadSignal<bool>,
    set_muted: WriteSignal<bool>,
    enable_background_audio: ReadSignal<bool>,
    set_enable_background_audio: WriteSignal<bool>,
) -> impl IntoView {
    let volume_percentage = move || (volume.get() * 100.0) as u32;
    
    view! {
        <div class="settings-section">
            <h3>"Audio Configuration"</h3>
            
            <div class="form-group checkbox-group">
                <label class="checkbox-label">
                    <input
                        type="checkbox"
                        prop:checked=move || !muted.get()
                        on:change=move |ev| set_muted.set(!event_target_checked(&ev))
                    />
                    <span class="checkmark"></span>
                    "Enable Audio Notifications"
                </label>
                <small class="help-text">"Play sounds when phases transition"</small>
            </div>

            <div class="form-group">
                <label for="volume">"Volume"</label>
                <div class="volume-control">
                    <input
                        type="range"
                        id="volume"
                        min="0"
                        max="100"
                        prop:value=volume_percentage
                        disabled=muted
                        on:input=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                set_volume.set((val as f32) / 100.0);
                            }
                        }
                    />
                    <span class="volume-display">
                        {move || if muted.get() { "Muted".to_string() } else { format!("{}%", volume_percentage()) }}
                    </span>
                </div>
            </div>

            <div class="form-group checkbox-group">
                <label class="checkbox-label">
                    <input
                        type="checkbox"
                        prop:checked=enable_background_audio
                        disabled=muted
                        on:change=move |ev| set_enable_background_audio.set(event_target_checked(&ev))
                    />
                    <span class="checkmark"></span>
                    "Enable Background Audio"
                </label>
                <small class="help-text">"Play ambient sounds during work sessions"</small>
            </div>

            <div class="audio-options">
                <div class="form-group">
                    <label for="work-notification-sound">"Work Session End Sound"</label>
                    <select id="work-notification-sound" disabled=muted>
                        <option value="">"Default"</option>
                        <option value="bell">"Bell"</option>
                        <option value="chime">"Chime"</option>
                        <option value="ding">"Ding"</option>
                    </select>
                </div>

                <div class="form-group">
                    <label for="break-notification-sound">"Break End Sound"</label>
                    <select id="break-notification-sound" disabled=muted>
                        <option value="">"Default"</option>
                        <option value="bell">"Bell"</option>
                        <option value="chime">"Chime"</option>
                        <option value="ding">"Ding"</option>
                    </select>
                </div>

                <Show when=move || enable_background_audio.get() && !muted.get()>
                    <div class="form-group">
                        <label for="background-sound">"Background Sound"</label>
                        <select id="background-sound">
                            <option value="">"None"</option>
                            <option value="rain">"Rain"</option>
                            <option value="forest">"Forest"</option>
                            <option value="ocean">"Ocean Waves"</option>
                            <option value="white-noise">"White Noise"</option>
                            <option value="brown-noise">"Brown Noise"</option>
                            <option value="cafe">"Café Ambience"</option>
                        </select>
                    </div>
                </Show>
            </div>

            <div class="audio-preview">
                <h4>"Test Audio"</h4>
                <div class="preview-buttons">
                    <button 
                        class="btn secondary small"
                        disabled=muted
                    >
                        "🔔 Test Notification"
                    </button>
                    <Show when=move || enable_background_audio.get() && !muted.get()>
                        <button 
                            class="btn secondary small"
                        >
                            "🎵 Preview Background"
                        </button>
                    </Show>
                </div>
            </div>
        </div>
    }
}