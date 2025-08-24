use leptos::prelude::*;
use crate::pages::settings::SettingsViewModel;
use crate::utils::ViewModel;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let vm = StoredValue::new(SettingsViewModel::new());

    view! {
        <div class="settings-container">
            <h2 class="settings-title">"Settings"</h2>

            {move || {
                vm.with_value(|v| v.get_config()).map(|config| {
                    // Extract values to avoid move issues
                    let auto_start_breaks = config.general.auto_start_breaks;
                    let auto_start_work = config.general.auto_start_work_after_break;

                    view! {
                        <>
                            <div class="setting-group">
                                <label class="setting-label">"Focus Duration (minutes)"</label>
                                <input type="number" class="setting-input" id="focusDuration" value="25" min="1" max="60" />
                            </div>
                            <div class="setting-group">
                                <label class="setting-label">"Short Break Duration (minutes)"</label>
                                <input type="number" class="setting-input" id="shortBreakDuration" value="5" min="1" max="30" />
                            </div>
                            <div class="setting-group">
                                <label class="setting-label">"Long Break Duration (minutes)"</label>
                                <input type="number" class="setting-input" id="longBreakDuration" value="15" min="1" max="60" />
                            </div>
                            <div class="setting-group">
                                <label class="setting-label">"Sessions Until Long Break"</label>
                                <input type="number" class="setting-input" id="sessionsUntilLong" value="4" min="2" max="10" />
                            </div>
                            <div class="setting-group">
                                <label class="setting-label">"Auto-start Breaks"</label>
                                <input
                                    type="checkbox"
                                    checked=auto_start_breaks
                                    id="autoStartBreaks"
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
                            </div>
                            <div class="setting-group">
                                <label class="setting-label">"Auto-start Focus Sessions"</label>
                                <input
                                    type="checkbox"
                                    checked=auto_start_work
                                    id="autoStartFocus"
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
                            </div>
                            <button 
                                class="btn btn-primary save-settings-btn"
                                on:click=move |_| {
                                    vm.with_value(|v| v.save_settings());
                                }
                            >
                                "SAVE SETTINGS"
                            </button>
                        </>
                    }.into_any()
                }).unwrap_or_else(|| {
                    view! {
                        <p>"Loading settings..."</p>
                    }.into_any()
                })
            }}
        </div>
    }
}