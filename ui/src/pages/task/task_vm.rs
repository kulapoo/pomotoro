use crate::utils::{ViewModel, invoke_command, invoke_command_no_args};
use domain::{Task, TaskId, TimerState, event_names};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::{from_value, to_value};

pub struct TasksViewModel {
    tasks: ReadSignal<Vec<Task>>,
    set_tasks: WriteSignal<Vec<Task>>,
    filtered_tasks: ReadSignal<Vec<Task>>,
    set_filtered_tasks: WriteSignal<Vec<Task>>,
    active_task: ReadSignal<Option<Task>>,
    set_active_task: WriteSignal<Option<Task>>,
    selected_task: ReadSignal<Option<TaskId>>,
    set_selected_task: WriteSignal<Option<TaskId>>,
    is_creating: ReadSignal<bool>,
    set_is_creating: WriteSignal<bool>,
    search_query: ReadSignal<String>,
    set_search_query: WriteSignal<String>,
    sort_by: ReadSignal<String>,
    set_sort_by: WriteSignal<String>,
    status_filter: ReadSignal<String>,
    set_status_filter: WriteSignal<String>,
    cycle_position: ReadSignal<(usize, usize)>,
    set_cycle_position: WriteSignal<(usize, usize)>,
}

impl ViewModel for TasksViewModel {
    type State = Vec<Task>;

    fn new() -> Self {
        let (tasks, set_tasks) = signal(Vec::<Task>::new());
        let (filtered_tasks, set_filtered_tasks) = signal(Vec::<Task>::new());
        let (active_task, set_active_task) = signal(None::<Task>);
        let (selected_task, set_selected_task) = signal(None::<TaskId>);
        let (is_creating, set_is_creating) = signal(false);
        let (search_query, set_search_query) = signal(String::new());
        let (sort_by, set_sort_by) = signal("created_at".to_string());
        let (status_filter, set_status_filter) = signal("all".to_string());
        let (cycle_position, set_cycle_position) = signal((0, 0));

        let vm = Self {
            tasks,
            set_tasks,
            filtered_tasks,
            set_filtered_tasks,
            active_task,
            set_active_task,
            selected_task,
            set_selected_task,
            is_creating,
            set_is_creating,
            search_query,
            set_search_query,
            sort_by,
            set_sort_by,
            status_filter,
            set_status_filter,
            cycle_position,
            set_cycle_position,
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
        if self.search_query.get().is_empty() && self.status_filter.get() == "all" {
            self.tasks.get()
        } else {
            self.filtered_tasks.get()
        }
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
                description: Option<String>,
                max_sessions: u8,
                tags: Vec<String>,
                audio_config: Option<domain::AudioConfig>,
            }

            let args = CreateTaskArgs { 
                name, 
                description: if description.is_empty() { None } else { Some(description) },
                max_sessions: 4, // Default value
                tags: Vec::new(),
                audio_config: None,
            };

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
    
    pub fn search_tasks(&self, query: String) {
        self.set_search_query.set(query.clone());
        
        if query.is_empty() && self.status_filter.get() == "all" {
            self.set_filtered_tasks.set(self.tasks.get());
            return;
        }
        
        let set_filtered = self.set_filtered_tasks;
        let sort_by = self.sort_by.get();
        let status_filter = self.status_filter.get();
        
        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct SearchArgs {
                query: Option<String>,
                status: Option<String>,
                sort_by: Option<String>,
                sort_order: Option<String>,
            }
            
            let args = SearchArgs {
                query: if query.is_empty() { None } else { Some(query) },
                status: if status_filter == "all" { None } else { Some(status_filter) },
                sort_by: Some(sort_by),
                sort_order: Some("asc".to_string()),
            };
            
            if let Ok(args_value) = to_value(&args) {
                if let Ok(result) = invoke_command(event_names::task::SEARCH, args_value).await {
                    if let Ok(task_list) = from_value::<Vec<Task>>(result) {
                        set_filtered.set(task_list);
                    }
                }
            }
        });
    }
    
    pub fn set_sort(&self, sort_by: String) {
        self.set_sort_by.set(sort_by);
        self.search_tasks(self.search_query.get());
    }
    
    pub fn set_status_filter(&self, status: String) {
        self.set_status_filter.set(status);
        self.search_tasks(self.search_query.get());
    }
    
    pub fn get_search_query(&self) -> String {
        self.search_query.get()
    }
    
    pub fn get_sort_by(&self) -> String {
        self.sort_by.get()
    }
    
    pub fn get_status_filter(&self) -> String {
        self.status_filter.get()
    }
    
    pub fn cycle_to_next_incomplete_task(&self) {
        let set_active_task = self.set_active_task;
        let set_cycle_position = self.set_cycle_position;
        let active_task = self.active_task;
        
        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CycleArgs {
                current_task_id: Option<String>,
                direction: String,
            }
            
            let current_id = active_task
                .get_untracked()
                .map(|t| t.id.to_string());
            
            let args = CycleArgs {
                current_task_id: current_id,
                direction: "next".to_string(),
            };
            
            if let Ok(args_value) = to_value(&args) {
                if let Ok(result) =
                    invoke_command(event_names::task::CYCLE_INCOMPLETE_TASK, args_value).await
                {
                    #[derive(serde::Deserialize)]
                    struct CycleResult {
                        task: Option<Task>,
                        position: usize,
                        total_incomplete: usize,
                    }
                    
                    if let Ok(cycle_result) = from_value::<CycleResult>(result) {
                        set_active_task.set(cycle_result.task);
                        set_cycle_position.set((
                            cycle_result.position,
                            cycle_result.total_incomplete,
                        ));
                    }
                }
            }
        });
    }
    
    pub fn cycle_to_previous_incomplete_task(&self) {
        let set_active_task = self.set_active_task;
        let set_cycle_position = self.set_cycle_position;
        let active_task = self.active_task;
        
        spawn_local(async move {
            #[derive(serde::Serialize)]
            struct CycleArgs {
                current_task_id: Option<String>,
                direction: String,
            }
            
            let current_id = active_task
                .get_untracked()
                .map(|t| t.id.to_string());
            
            let args = CycleArgs {
                current_task_id: current_id,
                direction: "previous".to_string(),
            };
            
            if let Ok(args_value) = to_value(&args) {
                if let Ok(result) =
                    invoke_command(event_names::task::CYCLE_INCOMPLETE_TASK, args_value).await
                {
                    #[derive(serde::Deserialize)]
                    struct CycleResult {
                        task: Option<Task>,
                        position: usize,
                        total_incomplete: usize,
                    }
                    
                    if let Ok(cycle_result) = from_value::<CycleResult>(result) {
                        set_active_task.set(cycle_result.task);
                        set_cycle_position.set((
                            cycle_result.position,
                            cycle_result.total_incomplete,
                        ));
                    }
                }
            }
        });
    }
    
    pub fn get_cycle_position(&self) -> (usize, usize) {
        self.cycle_position.get()
    }
    
    pub fn update_cycle_position(&self) {
        let set_cycle_position = self.set_cycle_position;
        let active_task = self.active_task;
        
        spawn_local(async move {
            if let Some(task) = active_task.get_untracked() {
                if let Ok(args_value) = to_value(&task.id) {
                    if let Ok(result) =
                        invoke_command(event_names::task::GET_TASK_CYCLE_POSITION, args_value).await
                    {
                        if let Ok((position, total)) =
                            from_value::<(usize, usize)>(result)
                        {
                            set_cycle_position.set((position, total));
                        }
                    }
                }
            }
        });
    }
}
