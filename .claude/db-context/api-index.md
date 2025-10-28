# API Index - Key Functions and Methods

## Domain APIs

### Timer API
```rust
// Timer Core Methods
Timer::new() -> Timer
Timer::start(&mut self) -> Result<()>
Timer::pause(&mut self) -> Result<()>
Timer::reset(&mut self) -> Result<()>
Timer::skip_phase(&mut self) -> Result<Phase>
Timer::tick(&mut self) -> Result<Option<Event>>
Timer::get_state(&self) -> &TimerState

// State Machine
StateTransitions::can_transition(from: &TimerStatus, to: &TimerStatus) -> bool
StateTransitions::apply_transition(state: TimerState, transition: TransitionType) -> TransitionResult

// Timer Service Trait
trait TimerService {
    async fn start_timer(&self) -> Result<()>
    async fn stop_timer(&self) -> Result<()>
    async fn get_state(&self) -> Result<TimerState>
    async fn save_state(&self, state: TimerState) -> Result<()>
}
```

### Task API
```rust
// Task Builder
Task::builder() -> TaskBuilder
TaskBuilder::title(self, &str) -> Self
TaskBuilder::description(self, &str) -> Self
TaskBuilder::status(self, TaskStatus) -> Self
TaskBuilder::build(self) -> Task

// Task Methods
Task::complete_session(&mut self)
Task::update_status(&mut self, status: TaskStatus)
Task::is_completed(&self) -> bool
Task::apply_settings(&mut self, settings: TaskSettings)

// Repository Trait
trait TaskRepository {
    async fn create(&self, task: Task) -> Result<Task>
    async fn update(&self, task: Task) -> Result<Task>
    async fn delete(&self, id: TaskId) -> Result<()>
    async fn find_by_id(&self, id: TaskId) -> Result<Option<Task>>
    async fn find_all(&self) -> Result<Vec<Task>>
    async fn search(&self, query: &str, status: Option<TaskStatus>) -> Result<Vec<Task>>
    async fn find_default(&self) -> Result<Option<Task>>
}

// Cycling Service
trait TaskCyclerService {
    async fn get_next_task(&self, current: Option<TaskId>) -> Result<Option<Task>>
    async fn cycle_to_next(&self) -> Result<Option<Task>>
    async fn reset_cycle(&self) -> Result<()>
}
```

### Config API
```rust
// Config Methods
Config::default() -> Config
Config::merge(&mut self, other: Config)
Config::validate(&self) -> Result<()>

// Repository Trait
trait ConfigRepository {
    async fn get(&self) -> Result<Config>
    async fn update(&self, config: Config) -> Result<Config>
    async fn reset(&self) -> Result<Config>
    async fn export(&self) -> Result<String>
    async fn import(&self, data: &str) -> Result<Config>
}
```

### Audio API
```rust
// Audio Service
trait AudioService {
    async fn play(&self, request: PlaybackRequest) -> Result<PlaybackHandle>
    async fn stop(&self, handle: PlaybackHandle) -> Result<()>
    async fn stop_all(&self) -> Result<()>
    async fn set_volume(&self, handle: PlaybackHandle, volume: f32) -> Result<()>
    async fn get_library(&self) -> Result<AudioLibrary>
}

// Audio Library
AudioLibrary::new() -> AudioLibrary
AudioLibrary::add_asset(&mut self, asset: AudioAsset)
AudioLibrary::get_by_category(&self, category: AudioCategory) -> Vec<&AudioAsset>
AudioLibrary::find_by_name(&self, name: &str) -> Option<&AudioAsset>
```

## Use Case APIs

### Timer Use Cases
```rust
// Start Timer Session
StartTimerSession::new(timer_service, event_publisher) -> Self
StartTimerSession::execute(&self, task_id: Option<TaskId>) -> Result<TimerState>

// Pause Timer Session
PauseTimerSession::new(timer_service, event_publisher) -> Self
PauseTimerSession::execute(&self) -> Result<TimerState>

// Reset Timer Session
ResetTimerSession::new(timer_service, event_publisher) -> Self
ResetTimerSession::execute(&self) -> Result<TimerState>

// Skip Timer Phase
SkipTimerPhase::new(timer_service, event_publisher) -> Self
SkipTimerPhase::execute(&self) -> Result<TimerState>

// Get Timer State
GetTimerState::new(timer_service) -> Self
GetTimerState::execute(&self) -> Result<TimerState>
```

### Task Use Cases
```rust
// Create Task
CreateTask::new(repository, event_publisher) -> Self
CreateTask::execute(&self, request: CreateTaskRequest) -> Result<Task>

// Update Task
UpdateTask::new(repository, event_publisher) -> Self
UpdateTask::execute(&self, id: TaskId, request: UpdateTaskRequest) -> Result<Task>

// Delete Task
DeleteTask::new(repository, event_publisher) -> Self
DeleteTask::execute(&self, id: TaskId) -> Result<()>

// Search Tasks
SearchTasks::new(repository) -> Self
SearchTasks::execute(&self, query: &str, filter: SearchFilter) -> Result<Vec<Task>>

// Complete Session
CompleteSession::new(repository, event_publisher) -> Self
CompleteSession::execute(&self, task_id: TaskId) -> Result<Task>

// Cycle Task
CycleTask::new(cycler_service, event_publisher) -> Self
CycleTask::execute(&self, strategy: CyclingStrategy) -> Result<Option<Task>>
```

### Config Use Cases
```rust
// Get Config
GetConfig::new(repository) -> Self
GetConfig::execute(&self) -> Result<Config>

// Update Config
UpdateConfig::new(repository, event_publisher) -> Self
UpdateConfig::execute(&self, config: Config) -> Result<Config>

// Reset Config
ResetConfig::new(repository, event_publisher) -> Self
ResetConfig::execute(&self) -> Result<Config>

// Import/Export Config
ExportConfig::new(repository) -> Self
ExportConfig::execute(&self) -> Result<String>

ImportConfig::new(repository, event_publisher) -> Self
ImportConfig::execute(&self, data: &str) -> Result<Config>
```

## Infrastructure APIs

### Tauri Commands
```rust
// Timer Commands
#[tauri::command]
async fn start_timer(task_id: Option<String>, state: State<AppState>) -> Result<TimerDto>

#[tauri::command]
async fn pause_timer(state: State<AppState>) -> Result<TimerDto>

#[tauri::command]
async fn reset_timer(state: State<AppState>) -> Result<TimerDto>

#[tauri::command]
async fn skip_phase(state: State<AppState>) -> Result<TimerDto>

#[tauri::command]
async fn get_timer_state(state: State<AppState>) -> Result<TimerDto>

// Task Commands
#[tauri::command]
async fn create_task(request: CreateTaskDto, state: State<AppState>) -> Result<TaskDto>

#[tauri::command]
async fn update_task(id: String, request: UpdateTaskDto, state: State<AppState>) -> Result<TaskDto>

#[tauri::command]
async fn delete_task(id: String, state: State<AppState>) -> Result<()>

#[tauri::command]
async fn get_task(id: String, state: State<AppState>) -> Result<TaskDto>

#[tauri::command]
async fn get_all_tasks(state: State<AppState>) -> Result<Vec<TaskDto>>

#[tauri::command]
async fn search_tasks(query: String, filter: SearchFilterDto, state: State<AppState>) -> Result<Vec<TaskDto>>

// Config Commands
#[tauri::command]
async fn get_config(state: State<AppState>) -> Result<ConfigDto>

#[tauri::command]
async fn update_config(config: ConfigDto, state: State<AppState>) -> Result<ConfigDto>

#[tauri::command]
async fn reset_config(state: State<AppState>) -> Result<ConfigDto>

// Audio Commands
#[tauri::command]
async fn play_audio(asset_name: String, state: State<AppState>) -> Result<String>

#[tauri::command]
async fn stop_audio(handle: String, state: State<AppState>) -> Result<()>

#[tauri::command]
async fn set_volume(handle: String, volume: f32, state: State<AppState>) -> Result<()>
```

### Event Bus API
```rust
// Event Publisher
trait EventPublisher {
    async fn publish(&self, event: Box<dyn Event>) -> Result<()>
    async fn subscribe(&self, handler: Arc<dyn EventHandler>) -> Result<()>
}

// Event Handler
trait EventHandler {
    async fn handle(&self, event: Box<dyn Event>) -> Result<()>
    fn handles_event(&self, event_type: &str) -> bool
}

// Memory Event Bus
MemEventBus::new() -> MemEventBus
MemEventBus::register_handler(&self, event_type: &str, handler: Arc<dyn EventHandler>)
MemEventBus::publish(&self, event: Box<dyn Event>) -> Result<()>
```

### Repository Implementations
```rust
// SQLite Task Repository
SqliteTaskRepository::new(pool: Arc<Mutex<SqliteConnection>>) -> Self
SqliteTaskRepository::from_dto(dto: TaskDto) -> Task
SqliteTaskRepository::to_dto(task: &Task) -> TaskDto

// SQLite Config Repository
SqliteConfigRepository::new(pool: Arc<Mutex<SqliteConnection>>) -> Self
SqliteConfigRepository::load_or_create_default(&self) -> Result<Config>

// SQLite Timer Repository
SqliteTimerRepository::new(pool: Arc<Mutex<SqliteConnection>>) -> Self
SqliteTimerRepository::save_state(&self, state: TimerState) -> Result<()>
SqliteTimerRepository::load_state(&self) -> Result<Option<TimerState>>
```

## UI APIs (Leptos)

### Timer View Model
```rust
TimerViewModel::new() -> Self
TimerViewModel::start_timer(&self)
TimerViewModel::pause_timer(&self)
TimerViewModel::reset_timer(&self)
TimerViewModel::skip_phase(&self)
TimerViewModel::get_current_phase(&self) -> Phase
TimerViewModel::get_remaining_time(&self) -> Duration
```

### Task View Model
```rust
TaskViewModel::new() -> Self
TaskViewModel::create_task(&self, title: String, description: String)
TaskViewModel::update_task(&self, id: String, updates: TaskUpdate)
TaskViewModel::delete_task(&self, id: String)
TaskViewModel::search_tasks(&self, query: String)
TaskViewModel::get_active_task(&self) -> Option<Task>
```

### Settings View Model
```rust
SettingsViewModel::new() -> Self
SettingsViewModel::load_config(&self)
SettingsViewModel::update_config(&self, config: ConfigUpdate)
SettingsViewModel::reset_to_defaults(&self)
SettingsViewModel::export_config(&self) -> String
SettingsViewModel::import_config(&self, data: String)
```

## Event Types

### Timer Events
```rust
TimerStarted { task_id: Option<TaskId>, timestamp: Timestamp }
TimerPaused { remaining: Duration, timestamp: Timestamp }
TimerReset { timestamp: Timestamp }
TimerTick { remaining: Duration, phase: Phase }
PhaseCompleted { phase: Phase, timestamp: Timestamp }
PhaseSkipped { from: Phase, to: Phase, timestamp: Timestamp }
WorkPhaseStarted { task_id: Option<TaskId>, timestamp: Timestamp }
WorkPhaseCompleted { task_id: Option<TaskId>, timestamp: Timestamp }
BreakPhaseStarted { duration: Duration, timestamp: Timestamp }
BreakPhaseCompleted { timestamp: Timestamp }
```

### Task Events
```rust
TaskCreated { task: Task, timestamp: Timestamp }
TaskUpdated { task: Task, changes: Vec<String>, timestamp: Timestamp }
TaskDeleted { task_id: TaskId, timestamp: Timestamp }
TaskStatusChanged { task_id: TaskId, from: TaskStatus, to: TaskStatus, timestamp: Timestamp }
TaskCompleted { task_id: TaskId, timestamp: Timestamp }
TaskSessionCompleted { task_id: TaskId, session_count: u32, timestamp: Timestamp }
```

### System Events
```rust
AppStarted { version: String, timestamp: Timestamp }
AppExited { timestamp: Timestamp }
ConfigUpdated { config: Config, timestamp: Timestamp }
ConfigReset { timestamp: Timestamp }
```

## Utility Functions

### Time Utilities
```rust
Timestamp::now() -> Timestamp
Timestamp::from_unix(secs: i64) -> Timestamp
Duration::from_minutes(mins: u32) -> Duration
Duration::as_seconds(&self) -> u64
```

### ID Generation
```rust
TaskId::new() -> TaskId
EntityId::new() -> EntityId<T>
EntityId::from_string(s: &str) -> Result<EntityId<T>>
```

### Serialization
```rust
// DTOs have From/Into implementations
impl From<Task> for TaskDto
impl From<TaskDto> for Task
impl From<Config> for ConfigDto
impl From<ConfigDto> for Config
```