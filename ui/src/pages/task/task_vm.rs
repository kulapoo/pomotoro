use crate::utils::{ViewModel, invoke_command, invoke_command_no_args};
use domain::{Task, TaskId, TimerState, event_names};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::{from_value, to_value};

pub struct TasksViewModel {
    tasks: ReadSignal<Vec<Task>>,
    set_tasks: WriteSignal<Vec<Task>>,
    active_task: ReadSignal<Option<Task>>,
    set_active_task: WriteSignal<Option<Task>>,
    selected_task: ReadSignal<Option<TaskId>>,
    set_selected_task: WriteSignal<Option<TaskId>>,
    is_creating: ReadSignal<bool>,
    set_is_creating: WriteSignal<bool>,
}

impl ViewModel for TasksViewModel {
    type State = Vec<Task>;

    fn new() -> Self {
        let (tasks, set_tasks) = signal(Vec::<Task>::new());
        let (active_task, set_active_task) = signal(None::<Task>);
        let (selected_task, set_selected_task) = signal(None::<TaskId>);
        let (is_creating, set_is_creating) = signal(false);

        let vm = Self {
            tasks,
            set_tasks,
            active_task,
            set_active_task,
            selected_task,
            set_selected_task,
            is_creating,
            set_is_creating,
        };

        vm.load_initial_data();
        vm
    }

    fn state(&self) -> ReadSignal<Self::State> {
        self.tasks
    }

    fn set_state(&self) -> WriteSignal<Self::State> {
        self.set_tasks
    }
}

impl TasksViewModel {
    fn load_initial_data(&self) {
        let set_tasks = self.set_tasks;
        let set_active_task = self.set_active_task;

        spawn_local(async move {
            if let Ok(result) =
                invoke_command_no_args(event_names::task::GET_ALL).await
            {
                if let Ok(task_list) = from_value::<Vec<Task>>(result) {
                    set_tasks.set(task_list);
                }
            }

            if let Ok(result) =
                invoke_command_no_args(event_names::timer::GET_STATE).await
            {
                if let Ok(timer_state) = from_value::<TimerState>(result) {
                    if let Some(entity_id_str) = timer_state.active_entity_id()
                    {
                        if let Ok(task_id) = TaskId::from_string(&entity_id_str)
                        {
                            if let Ok(task_args) = to_value(&task_id) {
                                if let Ok(task_result) = invoke_command(
                                    event_names::task::GET,
                                    task_args,
                                )
                                .await
                                {
                                    if let Ok(task) =
                                        from_value::<Task>(task_result)
                                    {
                                        set_active_task.set(Some(task));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        self.tasks.get()
    }

    pub fn get_active_task(&self) -> Option<Task> {
        self.active_task.get()
    }

    pub fn get_selected_task(&self) -> Option<TaskId> {
        self.selected_task.get()
    }

    pub fn is_creating_task(&self) -> bool {
        self.is_creating.get()
    }

    pub fn set_creating_task(&self, creating: bool) {
        self.set_is_creating.set(creating);
    }

    pub fn select_task(&self, task_id: Option<TaskId>) {
        self.set_selected_task.set(task_id);
    }

    pub fn create_task(&self, name: String, description: String) {
        let set_tasks = self.set_tasks;
        let set_is_creating = self.set_is_creating;
        let tasks = self.tasks;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CreateTaskArgs {
                name: String,
                description: String,
            }

            let args = CreateTaskArgs { name, description };

            if let Ok(args_value) = to_value(&args) {
                if let Ok(result) =
                    invoke_command(event_names::task::CREATE, args_value).await
                {
                    if let Ok(new_task) = from_value::<Task>(result) {
                        let mut current_tasks = tasks.get_untracked();
                        current_tasks.push(new_task);
                        set_tasks.set(current_tasks);
                        set_is_creating.set(false);
                    }
                }
            }
        });
    }

    pub fn update_task(
        &self,
        task_id: TaskId,
        name: String,
        description: String,
    ) {
        let set_tasks = self.set_tasks;
        let tasks = self.tasks;

        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct UpdateTaskArgs {
                id: TaskId,
                name: String,
                description: String,
            }

            let args = UpdateTaskArgs {
                id: task_id,
                name,
                description,
            };

            if let Ok(args_value) = to_value(&args) {
                if let Ok(result) =
                    invoke_command(event_names::task::UPDATE, args_value).await
                {
                    if let Ok(updated_task) = from_value::<Task>(result) {
                        let mut current_tasks = tasks.get_untracked();
                        if let Some(index) =
                            current_tasks.iter().position(|t| t.id == task_id)
                        {
                            current_tasks[index] = updated_task;
                            set_tasks.set(current_tasks);
                        }
                    }
                }
            }
        });
    }

    pub fn delete_task(&self, task_id: TaskId) {
        let set_tasks = self.set_tasks;
        let tasks = self.tasks;
        let set_selected_task = self.set_selected_task;

        spawn_local(async move {
            if let Ok(args_value) = to_value(&task_id) {
                if (invoke_command(event_names::task::DELETE, args_value).await).is_ok() {
                    let mut current_tasks = tasks.get_untracked();
                    current_tasks.retain(|t| t.id != task_id);
                    set_tasks.set(current_tasks);
                    set_selected_task.set(None);
                }
            }
        });
    }

    pub fn switch_active_task(&self, task_id: TaskId) {
        let set_active_task = self.set_active_task;
        let tasks = self.tasks;

        spawn_local(async move {
            if let Ok(args) = to_value(&task_id) {
                if let Ok(result) =
                    invoke_command(event_names::timer::SWITCH_ACTIVE_TASK, args)
                        .await
                {
                    if let Ok(timer_state) = from_value::<TimerState>(result) {
                        if let Some(entity_id_str) =
                            timer_state.active_entity_id()
                        {
                            if let Ok(active_id) =
                                TaskId::from_string(&entity_id_str)
                            {
                                let task_list = tasks.get_untracked();
                                let active_task = task_list
                                    .iter()
                                    .find(|t| t.id == active_id)
                                    .cloned();
                                set_active_task.set(active_task);
                            }
                        } else {
                            set_active_task.set(None);
                        }
                    }
                }
            }
        });
    }

    pub fn refetch_tasks(&self) {
        let set_tasks = self.set_tasks;
        let set_active_task = self.set_active_task;
        let tasks = self.tasks;

        spawn_local(async move {
            if let Ok(result) =
                invoke_command_no_args(event_names::task::GET_ALL).await
            {
                if let Ok(task_list) = from_value::<Vec<Task>>(result) {
                    set_tasks.set(task_list);
                }
            }

            if let Ok(result) =
                invoke_command_no_args(event_names::timer::GET_STATE).await
            {
                if let Ok(timer_state) = from_value::<TimerState>(result) {
                    if let Some(entity_id_str) = timer_state.active_entity_id()
                    {
                        if let Ok(task_id) = TaskId::from_string(&entity_id_str)
                        {
                            let task_list = tasks.get_untracked();
                            let active_task = task_list
                                .iter()
                                .find(|t| t.id == task_id)
                                .cloned();
                            set_active_task.set(active_task);
                        }
                    } else {
                        set_active_task.set(None);
                    }
                }
            }
        });
    }
}
