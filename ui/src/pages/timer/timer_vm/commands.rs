use domain::{Task, Timer, event_names::commands};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::handle_command_error;
use crate::utils::invoke;

use super::TimerViewModel;
use super::task_ops;

impl TimerViewModel {
    pub fn start_pause_timer(&self) {
        let current_state = self.timer_state.get_untracked();
        let set_timer_state = self.set_timer_state;
        let set_error_state = self.set_error_state;
        let active_task = self.active_task.get_untracked();

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

            #[derive(serde::Serialize)]
            struct TimerArgs {
                task_id: Option<String>,
            }

            let args = TimerArgs {
                task_id: active_task.map(|t| t.id.to_string()),
            };

            invoke::<Timer, TimerArgs>(command, Some(args)).await
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

    pub fn complete_phase(&self) {
        web_sys::console::log_1(&"Phase completion is handled automatically by the backend".into());
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

                    if let Some(task_id) = timer.active_task_id() {
                        let task_id_str = task_id.to_string();
                        spawn_local(async move {
                            task_ops::fetch_task_by_id(&task_id_str, set_active_task).await;
                        });
                    } else {
                        set_active_task.set(None);
                    }
                });
        });
    }
}
