use crate::pages::task::TasksViewModel;
use leptos::prelude::*;
use domain::TimerConfiguration;
use std::time::Duration;

#[component]
pub fn TaskCreationForm<F>(
    vm: StoredValue<TasksViewModel>,
    on_close: F,
) -> impl IntoView
where
    F: Fn() + 'static + Copy,
{
    let (task_name, set_task_name) = signal(String::new());
    let (task_description, set_task_description) = signal(String::new());
    let (max_sessions, set_max_sessions) = signal(4u32);
    let (tags_input, set_tags_input) = signal(String::new());
    let (use_custom_config, set_use_custom_config) = signal(false);
    let (work_duration, set_work_duration) = signal(25u64);
    let (short_break, set_short_break) = signal(5u64);
    let (long_break, set_long_break) = signal(15u64);
    let (sessions_until_long_break, set_sessions_until_long_break) = signal(4usize);
    let (is_creating, set_is_creating) = signal(false);
    let (validation_error, set_validation_error) = signal(None::<String>);

    let validate_form = move || -> Result<(), String> {
        let name = task_name.get();
        if name.trim().is_empty() {
            return Err("Task name is required".to_string());
        }
        if name.len() > 100 {
            return Err("Task name must be less than 100 characters".to_string());
        }
        if max_sessions.get() < 1 || max_sessions.get() > 100 {
            return Err("Max sessions must be between 1 and 100".to_string());
        }
        if use_custom_config.get() {
            if work_duration.get() < 1 || work_duration.get() > 90 {
                return Err("Work duration must be between 1 and 90 minutes".to_string());
            }
            if short_break.get() < 1 || short_break.get() > 30 {
                return Err("Short break must be between 1 and 30 minutes".to_string());
            }
            if long_break.get() < 5 || long_break.get() > 60 {
                return Err("Long break must be between 5 and 60 minutes".to_string());
            }
            if sessions_until_long_break.get() < 2 || sessions_until_long_break.get() > 10 {
                return Err("Sessions until long break must be between 2 and 10".to_string());
            }
        }
        Ok(())
    };

    let create_task = move |_| {
        set_validation_error.set(None);

        match validate_form() {
            Err(error) => {
                set_validation_error.set(Some(error));
                return;
            }
            Ok(_) => {}
        }

        if is_creating.get() {
            return;
        }

        set_is_creating.set(true);

        let name = task_name.get().trim().to_string();
        let description = if task_description.get().trim().is_empty() {
            None
        } else {
            Some(task_description.get().trim().to_string())
        };

        let tags: Vec<String> = if tags_input.get().trim().is_empty() {
            Vec::new()
        } else {
            tags_input.get()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        let custom_config = if use_custom_config.get() {
            Some(TimerConfiguration::new(
                Duration::from_secs(work_duration.get() * 60),
                Duration::from_secs(short_break.get() * 60),
                Duration::from_secs(long_break.get() * 60),
                sessions_until_long_break.get() as u8,
            ).expect("Invalid timer configuration"))
        } else {
            None
        };

        web_sys::console::log_1(&format!("Creating task: {} with {} sessions", name, max_sessions.get()).into());

        // Clear form immediately for better UX
        set_task_name.set(String::new());
        set_task_description.set(String::new());
        set_max_sessions.set(4);
        set_tags_input.set(String::new());
        set_use_custom_config.set(false);
        set_work_duration.set(25);
        set_short_break.set(5);
        set_long_break.set(15);
        set_sessions_until_long_break.set(4);

        vm.with_value(|v| {
            v.create_task_full(name, description, max_sessions.get() as usize, tags, custom_config);
        });

        // Wait a bit for the async operation to complete
        set_timeout(
            move || {
                set_is_creating.set(false);
                on_close();
            },
            std::time::Duration::from_millis(500),
        );
    };

    view! {
        <div class="task-creation-form">
            <h4 class="form-title">"Create New Task"</h4>

            <Show when=move || validation_error.get().is_some()>
                <div class="validation-error">
                    {move || validation_error.get().unwrap_or_default()}
                </div>
            </Show>

            <div class="form-group">
                <label class="form-label">"Task Name"<span class="required">"*"</span></label>
                <input
                    type="text"
                    class="form-input"
                    placeholder="Enter task name..."
                    prop:value=move || task_name.get()
                    on:input=move |ev| {
                        set_task_name.set(event_target_value(&ev));
                    }
                    prop:disabled=move || is_creating.get()
                />
            </div>

            <div class="form-group">
                <label class="form-label">"Description"</label>
                <textarea
                    class="form-textarea"
                    placeholder="Enter task description..."
                    prop:value=move || task_description.get()
                    on:input=move |ev| {
                        set_task_description.set(event_target_value(&ev));
                    }
                    prop:disabled=move || is_creating.get()
                    rows="3"
                ></textarea>
            </div>

            <div class="form-group">
                <label class="form-label">"Max Sessions"</label>
                <input
                    type="number"
                    class="form-input"
                    min="1"
                    max="100"
                    prop:value=move || max_sessions.get()
                    on:input=move |ev| {
                        let value = event_target_value(&ev).parse::<u32>().unwrap_or(4);
                        set_max_sessions.set(value);
                    }
                    prop:disabled=move || is_creating.get()
                />
                <span class="form-help">"Number of pomodoro sessions for this task"</span>
            </div>

            <div class="form-group">
                <label class="form-label">"Tags"</label>
                <input
                    type="text"
                    class="form-input"
                    placeholder="Enter tags separated by commas..."
                    prop:value=move || tags_input.get()
                    on:input=move |ev| {
                        set_tags_input.set(event_target_value(&ev));
                    }
                    prop:disabled=move || is_creating.get()
                />
                <span class="form-help">"e.g., work, personal, urgent"</span>
            </div>

            <div class="form-group">
                <label class="form-checkbox">
                    <input
                        type="checkbox"
                        prop:checked=move || use_custom_config.get()
                        on:change=move |ev| {
                            set_use_custom_config.set(event_target_checked(&ev));
                        }
                        prop:disabled=move || is_creating.get()
                    />
                    <span>"Use custom timer settings for this task"</span>
                </label>
            </div>

            <Show when=move || use_custom_config.get()>
                <div class="custom-timer-settings">
                    <h5 class="settings-subtitle">"Custom Timer Settings"</h5>

                    <div class="form-group">
                        <label class="form-label">"Work Duration (minutes)"</label>
                        <input
                            type="number"
                            class="form-input"
                            min="1"
                            max="90"
                            prop:value=move || work_duration.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<u64>().unwrap_or(25);
                                set_work_duration.set(value);
                            }
                            prop:disabled=move || is_creating.get()
                        />
                    </div>

                    <div class="form-group">
                        <label class="form-label">"Short Break (minutes)"</label>
                        <input
                            type="number"
                            class="form-input"
                            min="1"
                            max="30"
                            prop:value=move || short_break.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<u64>().unwrap_or(5);
                                set_short_break.set(value);
                            }
                            prop:disabled=move || is_creating.get()
                        />
                    </div>

                    <div class="form-group">
                        <label class="form-label">"Long Break (minutes)"</label>
                        <input
                            type="number"
                            class="form-input"
                            min="5"
                            max="60"
                            prop:value=move || long_break.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<u64>().unwrap_or(15);
                                set_long_break.set(value);
                            }
                            prop:disabled=move || is_creating.get()
                        />
                    </div>

                    <div class="form-group">
                        <label class="form-label">"Sessions Until Long Break"</label>
                        <input
                            type="number"
                            class="form-input"
                            min="2"
                            max="10"
                            prop:value=move || sessions_until_long_break.get()
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<usize>().unwrap_or(4);
                                set_sessions_until_long_break.set(value);
                            }
                            prop:disabled=move || is_creating.get()
                        />
                    </div>
                </div>
            </Show>

            <div class="form-actions">
                <button
                    class="btn btn-primary"
                    prop:disabled=move || task_name.get().trim().is_empty() || is_creating.get()
                    on:click=create_task
                >
                    {move || if is_creating.get() { "Creating..." } else { "Create Task" }}
                </button>

                <button
                    class="btn btn-secondary"
                    on:click=move |_| on_close()
                    prop:disabled=move || is_creating.get()
                >
                    "Cancel"
                </button>
            </div>
        </div>
    }
}