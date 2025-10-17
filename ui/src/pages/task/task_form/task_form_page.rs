use super::TaskFormViewModel;
use crate::pages::task::types::TaskDto;
use crate::utils::{ViewModel, invoke};
use domain::{Task, TaskId, TimerConfiguration};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_params_map};
use std::time::Duration;
use uuid::Uuid;

#[component]
pub fn TaskFormPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    let vm = StoredValue::new(TaskFormViewModel::new());

    // Extract task ID from route params if in edit mode
    let task_id = move || {
        params.with_untracked(|p| {
            p.get("id").and_then(|id| {
                Uuid::parse_str(&id)
                    .ok()
                    .map(|uuid| TaskId::from_uuid(uuid))
            })
        })
    };

    // State for the loaded task
    let (task, set_task) = signal(None::<Task>);
    let (is_loading, set_is_loading) = signal(true);

    // Fetch task if in edit mode
    let is_update = move || task_id().is_some();

    // Load task data when in edit mode
    if let Some(id) = task_id() {
        spawn_local(async move {
            // Fetch all tasks and find the one we need
            invoke::<Vec<TaskDto>, ()>(
                domain::event_names::commands::task::GET_ALL,
                None,
            )
            .await
            .ok()
            .and_then(|task_dto_list| {
                task_dto_list
                    .into_iter()
                    .filter_map(|dto| dto.to_task().ok())
                    .find(|fetched_task| fetched_task.id == id)
            })
            .map(|fetched_task| set_task.set(Some(fetched_task)))
            .unwrap_or_else(|| {
                web_sys::console::error_1(
                    &format!("Failed to fetch task with id: {}", id).into(),
                );
            });
            set_is_loading.set(false);
        });
    } else {
        set_is_loading.set(false);
    }

    let (task_name, set_task_name) = signal(String::new());
    let (task_description, set_task_description) = signal(String::new());
    let (max_sessions, set_max_sessions) = signal(4u32);
    let (tags_input, set_tags_input) = signal(String::new());
    let (use_custom_config, set_use_custom_config) = signal(false);
    let (work_duration, set_work_duration) = signal(25u64);
    let (short_break, set_short_break) = signal(5u64);
    let (long_break, set_long_break) = signal(15u64);
    let (sessions_until_long_break, set_sessions_until_long_break) =
        signal(4usize);

    // Update form fields when task is loaded
    Effect::new(move || {
        if let Some(loaded_task) = task.get() {
            set_task_name.set(loaded_task.name.clone());
            set_task_description
                .set(loaded_task.description.clone().unwrap_or_default());
            set_max_sessions.set(loaded_task.max_sessions as u32);
            set_tags_input.set(loaded_task.tags.join(", "));

            let has_custom_config =
                loaded_task.config.timer != TimerConfiguration::default();
            set_use_custom_config.set(has_custom_config);

            let timer_config = &loaded_task.config.timer;
            set_work_duration.set(timer_config.work_duration.as_secs() / 60);
            set_short_break
                .set(timer_config.short_break_duration.as_secs() / 60);
            set_long_break.set(timer_config.long_break_duration.as_secs() / 60);
            set_sessions_until_long_break
                .set(timer_config.sessions_until_long_break as usize);
        }
    });

    let (is_submitting, set_is_submitting) = signal(false);
    let (validation_error, set_validation_error) = signal(None::<String>);

    let validate_form = move || -> Result<(), String> {
        let name = task_name.get();
        if name.trim().is_empty() {
            return Err("Task name is required".to_string());
        }
        if name.len() > 100 {
            return Err(
                "Task name must be less than 100 characters".to_string()
            );
        }
        if max_sessions.get() < 1 || max_sessions.get() > 100 {
            return Err("Max sessions must be between 1 and 100".to_string());
        }
        if use_custom_config.get() {
            if work_duration.get() < 1 || work_duration.get() > 90 {
                return Err("Work duration must be between 1 and 90 minutes"
                    .to_string());
            }
            if short_break.get() < 1 || short_break.get() > 30 {
                return Err(
                    "Short break must be between 1 and 30 minutes".to_string()
                );
            }
            if long_break.get() < 5 || long_break.get() > 60 {
                return Err(
                    "Long break must be between 5 and 60 minutes".to_string()
                );
            }
            if sessions_until_long_break.get() < 2
                || sessions_until_long_break.get() > 10
            {
                return Err(
                    "Sessions until long break must be between 2 and 10"
                        .to_string(),
                );
            }
        }
        Ok(())
    };

    let submit_task = {
        let navigate = navigate.clone();
        move |_| {
            set_validation_error.set(None);

            match validate_form() {
                Err(error) => {
                    set_validation_error.set(Some(error));
                    return;
                }
                Ok(_) => {}
            }

            if is_submitting.get() {
                return;
            }

            set_is_submitting.set(true);

            let name = task_name.get().trim().to_string();
            let description = if task_description.get().trim().is_empty() {
                None
            } else {
                Some(task_description.get().trim().to_string())
            };

            let tags: Vec<String> = if tags_input.get().trim().is_empty() {
                Vec::new()
            } else {
                tags_input
                    .get()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };

            // Prepare individual timer config fields
            let timer_work_duration = if use_custom_config.get() {
                Some(Duration::from_secs(work_duration.get() * 60))
            } else {
                None
            };
            let timer_short_break = if use_custom_config.get() {
                Some(Duration::from_secs(short_break.get() * 60))
            } else {
                None
            };
            let timer_long_break = if use_custom_config.get() {
                Some(Duration::from_secs(long_break.get() * 60))
            } else {
                None
            };
            let timer_sessions_until_long_break = if use_custom_config.get() {
                Some(sessions_until_long_break.get() as u8)
            } else {
                None
            };

            vm.with_value(|v| {
                if let Some(id) = task_id() {
                    web_sys::console::log_1(
                        &format!("Updating task: {:?}", id).into(),
                    );
                    v.update_task(
                        id,
                        Some(name.clone()),
                        description.clone(),
                        Some(max_sessions.get() as usize),
                        Some(tags.clone()),
                        timer_work_duration,
                        timer_short_break,
                        timer_long_break,
                        timer_sessions_until_long_break,
                        None, // enable_screen_blocking
                        None, // audio_config
                    );

                    // Navigate back to tasks after update
                    let navigate = navigate.clone();
                    navigate("/tasks", Default::default());
                } else {
                    web_sys::console::log_1(
                        &format!(
                            "Creating task: {} with {} sessions",
                            name,
                            max_sessions.get()
                        )
                        .into(),
                    );
                    v.create_task_full(
                        name.clone(),
                        description.clone(),
                        max_sessions.get() as usize,
                        tags.clone(),
                        timer_work_duration,
                        timer_short_break,
                        timer_long_break,
                        timer_sessions_until_long_break,
                        None, // enable_screen_blocking
                        None, // audio_config
                    );

                    let navigate = navigate.clone();
                    set_timeout(
                        move || {
                            set_is_submitting.set(false);
                            navigate("/tasks", Default::default());
                        },
                        std::time::Duration::from_millis(500),
                    );
                }
            });
        }
    };

    view! {
        <Show
            when=move || !is_loading.get()
            fallback=|| view! {
                <div class="task-creation-form">
                    <p>"Loading task..."</p>
                </div>
            }
        >
            <div class="task-creation-form">
                <h4 class="form-title">
                    {move || if is_update() { "Update Task" } else { "Create New Task" }}
                </h4>

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
                    prop:disabled=move || is_submitting.get()
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
                    prop:disabled=move || is_submitting.get()
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
                    prop:disabled=move || is_submitting.get()
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
                    prop:disabled=move || is_submitting.get()
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
                        prop:disabled=move || is_submitting.get()
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
                            prop:disabled=move || is_submitting.get()
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
                            prop:disabled=move || is_submitting.get()
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
                            prop:disabled=move || is_submitting.get()
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
                            prop:disabled=move || is_submitting.get()
                        />
                    </div>
                </div>
            </Show>

            <div class="form-actions">
                <button
                    class="btn btn-primary"
                    prop:disabled=move || task_name.get().trim().is_empty() || is_submitting.get()
                    on:click={
                        let submit_task = submit_task.clone();
                        submit_task
                    }
                >
                    {move || {
                        if is_submitting.get() {
                            if is_update() { "Updating..." } else { "Creating..." }
                        } else {
                            if is_update() { "Update Task" } else { "Create Task" }
                        }
                    }}
                </button>

                <button
                    class="btn btn-secondary"
                    on:click={
                        let navigate = navigate.clone();
                        move |_| navigate("/tasks", Default::default())
                    }
                    prop:disabled=move || is_submitting.get()
                >
                    "Cancel"
                </button>
            </div>
        </div>
        </Show>
    }
}
