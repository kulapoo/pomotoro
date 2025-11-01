use domain::{Task, Timer, event_names::commands};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::handle_command_error;
use crate::utils::invoke;

use super::TimerViewModel;

impl TimerViewModel {
    pub fn start_pause_timer(&self) {
        let current_state = self.timer_state.get_untracked();
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;

        spawn_local(async move {

            let command = if current_state.is_running() {
                commands::timer::PAUSE
            }
            else if current_state.is_paused() {
                commands::timer::RESUME
            } else {
                commands::timer::START
            };

            web_sys::console::log_1(
                &format!("Executing timer command: {} (current state: {:?})",
                        command, current_state.status()).into()
            );

            // Only sync state for PAUSE to capture the current UI time
            if command == commands::timer::PAUSE {
                #[derive(serde::Serialize)]
                struct TimerStateArgs {
                    remaining_seconds: u32,
                }

                let timer_state_args = TimerStateArgs {
                    remaining_seconds: current_state.remaining_seconds(),
                };

                invoke::<(), TimerStateArgs>(commands::timer::UPDATE_TIMER_SECS, Some(timer_state_args)).await
                    .map_err(|e| handle_command_error(e, set_error_state))
                    .ok();
            }

            invoke::<Timer, ()>(command, None).await
            .map(|timer| {
                let status = timer.state().status();

                web_sys::console::log_1(
                    &format!("Timer updated after {}: {:?}", command, timer).into()
                );
                set_timer_state.set(timer.state().clone());
                web_sys::console::log_1(
                    &format!("Timer state updated after {}: {:?}", command, status).into()
                );
            })
            .map_err(|e| handle_command_error(e, set_error_state))
            .ok();
        });
    }

    pub fn reset_timer(&self) {
        let set_timer_state = self.set_timer_state;
        let set_active_task = self.set_active_task;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            invoke::<(Timer, Task), ()>(commands::timer::RESET, None).await
                .map(|(timer, task)| {
                    set_timer_state.set(timer.state().clone());
                    set_active_task.set(Some(task));
                })
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok();
        });
    }

    pub fn reset_task(&self) {
        let set_timer_state = self.set_timer_state;
        let set_active_task = self.set_active_task;
        let set_error_state = self.set_error_state;
        let active_task_id = self.get_active_entity_id().expect("No active task");

        #[derive(serde::Serialize)]
        struct ResetTaskArgs {
            task_id: String,
        }

        let reset_task_args = ResetTaskArgs {
            task_id: active_task_id,
        };

        spawn_local(async move {
            invoke::<(Timer, Task), ResetTaskArgs>(commands::task::RESET_TASK, Some(reset_task_args)).await
                .map(|(timer, task)| {
                    set_timer_state.set(timer.state().clone());
                    set_active_task.set(Some(task));
                })
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok();
        });
    }

    pub fn skip_phase(&self) {
        let set_timer_state = self.set_timer_state;
        let set_active_task = self.set_active_task;
        let set_error_state = self.set_error_state;

        spawn_local(async move {
            invoke::<Timer, ()>(commands::timer::SKIP_PHASE, None).await
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok()
                .map(|timer| {
                    set_timer_state.set(timer.state().clone());

                    let task_id = timer.task_id();
                    let task_id_str = task_id.to_string();
                    spawn_local(async move {
                        use serde::Serialize;

                        // Inline the fetch_task_by_id functionality
                        if task_id_str.is_empty() {
                            set_active_task.set(None);
                            return;
                        }

                        #[derive(Serialize)]
                        struct GetTaskArgs {
                            id: String,
                        }

                        let args = GetTaskArgs {
                            id: task_id_str,
                        };

                        let task = invoke::<Option<Task>, _>(commands::task::GET, Some(args)).await
                            .ok()
                            .flatten();

                        set_active_task.set(task);
                    });
                });
        });
    }

    pub fn complete_task(&self) {
        let set_active_task = self.set_active_task;
        let set_error_state = self.set_error_state;

        let active_task_id = self.get_active_entity_id().expect("Complete Task: No active task");

        #[derive(serde::Serialize)]
        struct CompleteTaskArgs {
            task_id: String,
        }

        let complete_args = CompleteTaskArgs {
            task_id: active_task_id,
        };

        spawn_local(async move {
            if let Some(task) = invoke::<Task, CompleteTaskArgs>(commands::task::COMPLETE_TASK, Some(complete_args)).await
                .map_err(|e| handle_command_error(e, set_error_state))
                .ok() { set_active_task.set(Some(task)); }
        });
    }
}
