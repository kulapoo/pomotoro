use crate::components::error_toast::handle_command_error;
use crate::utils::invoke;
use domain::event_names::commands;
use domain::{Task, TaskId};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Serialize;

use super::TaskDirectoryViewModel;

// Helper function to refetch all tasks
async fn refetch_all_tasks(set_tasks: WriteSignal<Vec<Task>>, command: &str) {
    let tasks = invoke::<Vec<Task>, ()>(command, None)
        .await
        .ok()
        .unwrap_or_default();

    set_tasks.set(tasks);
}

impl TaskDirectoryViewModel {
    pub fn delete_task(&self, task_id: TaskId) -> bool {
        let task_name = self
            .tasks
            .get()
            .iter()
            .find(|t| t.id == task_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "this task".to_string());

        let confirmed = leptos::prelude::window()
            .confirm_with_message(&format!(
                "Are you sure you want to delete \"{}\"?",
                task_name
            ))
            .unwrap_or(false);

        web_sys::console::log_1(&format!("Confirmed: {:?}", confirmed).into());

        if !confirmed {
            return false;
        }

        let set_tasks = self.set_tasks;
        let tasks = self.tasks;
        let set_selected_task = self.set_selected_task;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(Serialize)]
            struct DeleteTaskArgs {
                id: String,
            }

            let args = DeleteTaskArgs {
                id: task_id.to_string(),
            };

            web_sys::console::log_1(
                &format!("Invoking delete_task for task_id: {:?}", task_id)
                    .into(),
            );

            invoke::<(), _>(commands::task::DELETE, Some(args))
                .await
                .map(|_result| {
                    web_sys::console::log_1(
                        &format!("Successfully deleted task: {:?}", task_id)
                            .into(),
                    );
                    let mut current_tasks = tasks.get_untracked();
                    current_tasks.retain(|t| t.id != task_id);
                    set_tasks.set(current_tasks);
                    set_selected_task.set(None);
                    // Clear any existing errors on success
                    set_error_state.set(None);
                })
                .map_err(|e| {
                    handle_command_error(
                        format!("Failed to delete task: {}", e),
                        set_error_state,
                    );
                })
                .ok();
        });

        true
    }

    pub fn switch_active_task(
        &self,
        task_id: TaskId,
        on_success: Option<Box<dyn Fn() + 'static>>,
    ) {
        let set_active_task = self.set_active_task;
        let tasks = self.tasks;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            web_sys::console::log_1(
                &format!("Switching to task: {:?}", task_id).into(),
            );

            #[derive(Serialize)]
            struct SwitchTaskArgs {
                task_id: String,
            }

            let args = SwitchTaskArgs {
                task_id: task_id.to_string(),
            };

            web_sys::console::log_1(
                &format!(
                    "Invoking switch_active_task for task_id: {:?}",
                    task_id
                )
                .into(),
            );

            invoke::<serde_json::Value, _>(
                commands::timer::SWITCH_ACTIVE_TASK,
                Some(args),
            )
            .await
            .map(|timer_info| {
                web_sys::console::log_1(
                    &format!("Timer info received: {:?}", timer_info).into(),
                );
                let active_id = task_id;
                let task_list = tasks.get_untracked();
                let active_task =
                    task_list.iter().find(|t| t.id == active_id).cloned();
                let task_name = active_task.as_ref().map(|t| t.name.clone());
                set_active_task.set(active_task);
                web_sys::console::log_1(
                    &format!("Active task set to: {:?}", task_name).into(),
                );
                // Clear any existing errors on success
                set_error_state.set(None);

                // Call the success callback if provided
                if let Some(callback) = on_success {
                    callback();
                }
            })
            .map_err(|e| {
                handle_command_error(
                    format!("Failed to switch active task: {}", e),
                    set_error_state,
                );
            })
            .ok();
        });
    }

    pub fn cycle_to_next_incomplete_task(&self) {
        let set_tasks = self.set_tasks;
        let tasks = self.tasks;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CycleArgs {
                current_task_id: Option<String>,
                direction: String,
            }

            let current_id =
                tasks.get_untracked().first().map(|t| t.id.to_string());

            let args = CycleArgs {
                current_task_id: current_id,
                direction: "next".to_string(),
            };

            #[derive(serde::Deserialize)]
            struct CycleResult {
                task: Option<Task>,
                position: usize,
                total_incomplete: usize,
            }

            invoke::<CycleResult, _>(
                commands::task::CYCLE_INCOMPLETE_TASK,
                Some(args),
            )
            .await
            .map(|_cycle_result| {
                spawn_local(async move {
                    refetch_all_tasks(set_tasks, commands::task::GET_ALL).await;
                });
                // Clear any existing errors on success
                set_error_state.set(None);
            })
            .map_err(|e| {
                handle_command_error(
                    format!("Failed to cycle to next incomplete task: {}", e),
                    set_error_state,
                );
            })
            .ok();
        });
    }

    pub fn cycle_to_previous_incomplete_task(&self) {
        let set_tasks = self.set_tasks;
        let tasks = self.tasks;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CycleArgs {
                current_task_id: Option<String>,
                direction: String,
            }

            let current_id =
                tasks.get_untracked().first().map(|t| t.id.to_string());

            let args = CycleArgs {
                current_task_id: current_id,
                direction: "previous".to_string(),
            };

            #[derive(serde::Deserialize)]
            struct CycleResult {
                task: Option<Task>,
                position: usize,
                total_incomplete: usize,
            }

            invoke::<CycleResult, _>(
                commands::task::CYCLE_INCOMPLETE_TASK,
                Some(args),
            )
            .await
            .map(|_cycle_result| {
                spawn_local(async move {
                    refetch_all_tasks(set_tasks, commands::task::GET_ALL).await;
                });
                // Clear any existing errors on success
                set_error_state.set(None);
            })
            .map_err(|e| {
                handle_command_error(
                    format!(
                        "Failed to cycle to previous incomplete task: {}",
                        e
                    ),
                    set_error_state,
                );
            })
            .ok();
        });
    }

    pub fn reset_task_to_queued(&self, task_id: TaskId) {
        let set_tasks = self.set_tasks;
        let tasks = self.tasks;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct ResetTaskStatusArgs {
                task_id: String,
            }

            let args = ResetTaskStatusArgs {
                task_id: task_id.to_string(),
            };

            if let Ok((_timer, updated_task)) =
                invoke::<(domain::Timer, Task), _>(
                    commands::task::RESET_TASK,
                    Some(args),
                )
                .await
                .map_err(|e| {
                    handle_command_error(
                        format!("Failed to reset task status: {}", e),
                        set_error_state,
                    );
                })
            {
                web_sys::console::log_1(
                    &format!("Reset task status result: {:?}", updated_task)
                        .into(),
                );
                web_sys::console::log_1(
                    &format!(
                        "Successfully reset task: id={}, status={:?}",
                        updated_task.id, updated_task.status
                    )
                    .into(),
                );
                let mut current_tasks = tasks.get_untracked();
                if let Some(index) =
                    current_tasks.iter().position(|t| t.id == task_id)
                {
                    current_tasks[index] = updated_task;
                    set_tasks.set(current_tasks);
                }
                // Clear any existing errors on success
                set_error_state.set(None);
            }
        });
    }
}
