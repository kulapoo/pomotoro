use crate::pages::task::TasksViewModel;
use domain::{Task, TimerConfiguration};
use leptos::prelude::*;
use std::time::Duration;

#[component]
pub fn TaskUpdateForm<F>(
    vm: StoredValue<TasksViewModel>,
    task: Task,
    on_close: F,
) -> impl IntoView
where
    F: Fn() + 'static + Copy,
{
    let task_id = task.id;
    let (task_name, set_task_name) = signal(task.name.clone());
    let (task_description, set_task_description) = signal(
        task.description.clone().unwrap_or_default()
    );
    let (max_sessions, set_max_sessions) = signal(task.max_sessions as u32);
    let (tags_input, set_tags_input) = signal(task.tags.join(", "));
    let (use_custom_config, set_use_custom_config) = signal(
        task.config.timer != TimerConfiguration::default()
    );

    let timer_config = task.config.timer.clone();
    let (work_duration, set_work_duration) = signal(
        timer_config.work_duration.as_secs() / 60
    );
    let (short_break, set_short_break) = signal(
        timer_config.short_break_duration.as_secs() / 60
    );
    let (long_break, set_long_break) = signal(
        timer_config.long_break_duration.as_secs() / 60
    );
    let (sessions_until_long_break, set_sessions_until_long_break) = signal(
        timer_config.sessions_until_long_break as usize
    );
    let (is_updating, set_is_updating) = signal(false);
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

    let update_task = move |_| {
        set_validation_error.set(None);

        match validate_form() {
            Err(error) => {
                set_validation_error.set(Some(error));
                return;
            }
            Ok(_) => {}
        }

        if is_updating.get() {
            return;
        }

        set_is_updating.set(true);

        let name = Some(task_name.get().trim().to_string());
        let description = if task_description.get().trim().is_empty() {
            None
        } else {
            Some(task_description.get().trim().to_string())
        };

        let tags: Option<Vec<String>> = if tags_input.get().trim().is_empty() {
            Some(Vec::new())
        } else {
            Some(
                tags_input.get()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            )
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

        web_sys::console::log_1(&format!("Updating task: {:?}", task_id).into());

        vm.with_value(|v| {
            v.update_task(
                task_id,
                name,
                description,
                Some(max_sessions.get() as usize),
                tags,
                custom_config,
            );
        });

        on_close();
    };

    view! {
        <div class="task-form-overlay">
            <div class="task-form-modal">
                <div class="modal-header">
                    <h3>"Update Task"</h3>
                    <button
                        class="close-btn"
                        on:click=move |_| on_close()
                    >
                        "×"
                    </button>
                </div>

                <div class="modal-body">
                    <Show when=move || validation_error.get().is_some()>
                        {move || {
                            view! {
                                <div class="error-message">
                                    {validation_error.get().unwrap_or_default()}
                                </div>
                            }
                        }}
                    </Show>

                    <div class="form-group">
                        <label for="task-name">"Task Name"</label>
                        <input
                            type="text"
                            id="task-name"
                            class="form-input"
                            prop:value=move || task_name.get()
                            on:input=move |ev| {
                                set_task_name.set(event_target_value(&ev));
                            }
                            placeholder="e.g., Complete project proposal"
                        />
                    </div>

                    <div class="form-group">
                        <label for="task-description">"Description (optional)"</label>
                        <textarea
                            id="task-description"
                            class="form-textarea"
                            prop:value=move || task_description.get()
                            on:input=move |ev| {
                                set_task_description.set(event_target_value(&ev));
                            }
                            placeholder="Add details about this task..."
                            rows="3"
                        />
                    </div>

                    <div class="form-group">
                        <label for="max-sessions">"Maximum Pomodoro Sessions"</label>
                        <input
                            type="number"
                            id="max-sessions"
                            class="form-input"
                            prop:value=move || max_sessions.get()
                            on:input=move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                    set_max_sessions.set(val);
                                }
                            }
                            min="1"
                            max="100"
                        />
                    </div>

                    <div class="form-group">
                        <label for="tags">"Tags (comma-separated, optional)"</label>
                        <input
                            type="text"
                            id="tags"
                            class="form-input"
                            prop:value=move || tags_input.get()
                            on:input=move |ev| {
                                set_tags_input.set(event_target_value(&ev));
                            }
                            placeholder="e.g., work, urgent, project-x"
                        />
                    </div>

                    <div class="form-group checkbox-group">
                        <input
                            type="checkbox"
                            id="use-custom-config"
                            prop:checked=move || use_custom_config.get()
                            on:change=move |ev| {
                                set_use_custom_config.set(event_target_checked(&ev));
                            }
                        />
                        <label for="use-custom-config">"Use custom timer settings for this task"</label>
                    </div>

                    <Show when=move || use_custom_config.get()>
                        <div class="custom-config-section">
                            <h4>"Custom Timer Settings"</h4>

                            <div class="form-row">
                                <div class="form-group">
                                    <label for="work-duration">"Work Duration (minutes)"</label>
                                    <input
                                        type="number"
                                        id="work-duration"
                                        class="form-input"
                                        prop:value=move || work_duration.get()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<u64>() {
                                                set_work_duration.set(val);
                                            }
                                        }
                                        min="1"
                                        max="90"
                                    />
                                </div>

                                <div class="form-group">
                                    <label for="short-break">"Short Break (minutes)"</label>
                                    <input
                                        type="number"
                                        id="short-break"
                                        class="form-input"
                                        prop:value=move || short_break.get()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<u64>() {
                                                set_short_break.set(val);
                                            }
                                        }
                                        min="1"
                                        max="30"
                                    />
                                </div>
                            </div>

                            <div class="form-row">
                                <div class="form-group">
                                    <label for="long-break">"Long Break (minutes)"</label>
                                    <input
                                        type="number"
                                        id="long-break"
                                        class="form-input"
                                        prop:value=move || long_break.get()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<u64>() {
                                                set_long_break.set(val);
                                            }
                                        }
                                        min="5"
                                        max="60"
                                    />
                                </div>

                                <div class="form-group">
                                    <label for="sessions-long-break">"Sessions Until Long Break"</label>
                                    <input
                                        type="number"
                                        id="sessions-long-break"
                                        class="form-input"
                                        prop:value=move || sessions_until_long_break.get()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                                set_sessions_until_long_break.set(val);
                                            }
                                        }
                                        min="2"
                                        max="10"
                                    />
                                </div>
                            </div>
                        </div>
                    </Show>
                </div>

                <div class="modal-footer">
                    <button
                        class="btn btn-secondary"
                        on:click=move |_| on_close()
                    >
                        "Cancel"
                    </button>
                    <button
                        class="btn btn-primary"
                        on:click=update_task
                        disabled=move || is_updating.get()
                    >
                        {move || if is_updating.get() { "Updating..." } else { "Update Task" }}
                    </button>
                </div>
            </div>
        </div>
    }
}