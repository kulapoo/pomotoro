# Patterns and Conventions

## Architecture Patterns

### Clean Architecture Layers
```
UI → Use Cases → Domain ← Infrastructure
```

**Rules**:
- Domain has no external dependencies
- Use cases orchestrate domain logic
- Infrastructure implements domain interfaces
- UI communicates through use cases
- Dependencies point inward

### Repository Pattern
```rust
// Domain trait
pub trait TaskRepository: Send + Sync {
    async fn create(&self, task: Task) -> Result<Task>;
    async fn find_by_id(&self, id: TaskId) -> Result<Option<Task>>;
}

// Infrastructure implementation
pub struct SqliteTaskRepository {
    pool: Arc<Mutex<SqliteConnection>>,
}

impl TaskRepository for SqliteTaskRepository {
    // Implementation
}
```

### Service Pattern
```rust
// Domain service
pub trait TimerService: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}

// Use case using service
pub struct StartTimerSession {
    timer_service: Arc<dyn TimerService>,
    event_publisher: Arc<dyn EventPublisher>,
}
```

### Builder Pattern
```rust
Task::builder()
    .title("My Task")
    .description("Description")
    .status(TaskStatus::Pending)
    .build()
```

## Event-Driven Patterns

### Event Definition
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreated {
    pub task_id: TaskId,
    pub title: String,
    pub timestamp: Timestamp,
}

impl Event for TaskCreated {
    fn event_type(&self) -> &'static str {
        "task.created"
    }
}
```

### Event Publishing
```rust
// In domain operation
self.event_publisher.publish(Box::new(TaskCreated {
    task_id: task.id,
    title: task.title.clone(),
    timestamp: Timestamp::now(),
})).await?;
```

### Event Handling
```rust
pub struct TaskCreatedHandler {
    notification_service: Arc<dyn NotificationService>,
}

impl EventHandler for TaskCreatedHandler {
    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(created) = event.downcast_ref::<TaskCreated>() {
            // Handle event
        }
        Ok(())
    }
}
```

## Error Handling

### Domain Errors
```rust
#[derive(Debug, thiserror::Error)]
pub enum TimerError {
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),
    
    #[error("Timer not running")]
    NotRunning,
}
```

### Result Type Alias
```rust
pub type Result<T> = std::result::Result<T, Error>;
pub type TimerResult<T> = std::result::Result<T, TimerError>;
```

### Error Propagation
```rust
// Use ? operator
let task = self.repository.find_by_id(id).await?;

// Map errors
task.ok_or_else(|| Error::NotFound("Task not found".into()))?
```

## Testing Patterns

### Test Builders
```rust
#[cfg(test)]
pub struct TaskBuilder {
    title: String,
    status: TaskStatus,
}

impl TaskBuilder {
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
}
```

### Mock Implementations
```rust
#[cfg(test)]
pub struct MockTaskRepository {
    tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
}

impl TaskRepository for MockTaskRepository {
    // Mock implementation
}
```

### Test Fixtures
```rust
pub fn create_test_task() -> Task {
    Task::builder()
        .title("Test Task")
        .build()
}

pub fn create_test_config() -> Config {
    Config::default()
}
```

### Integration Test Structure
```rust
#[tokio::test]
async fn test_complete_workflow() {
    // Arrange
    let context = TestContext::new().await;
    
    // Act
    let result = context.create_task("Test").await;
    
    // Assert
    assert!(result.is_ok());
}
```

## Async Patterns

### Async Trait Methods
```rust
#[async_trait]
pub trait TaskRepository {
    async fn find_all(&self) -> Result<Vec<Task>>;
}
```

### Tokio Usage
```rust
// Spawning tasks
tokio::spawn(async move {
    // Async work
});

// Timeouts
tokio::time::timeout(Duration::from_secs(5), async_operation).await?;
```

### Arc for Shared State
```rust
pub struct AppState {
    timer: Arc<Mutex<Timer>>,
    repository: Arc<dyn TaskRepository>,
}
```

## Type Safety Patterns

### NewType Pattern
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskId(Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### Phantom Types
```rust
pub struct EntityId<T> {
    id: Uuid,
    _marker: PhantomData<T>,
}
```

### Type State Pattern
```rust
pub struct Timer<S> {
    state: S,
}

pub struct Idle;
pub struct Running;

impl Timer<Idle> {
    pub fn start(self) -> Timer<Running> {
        // Transition
    }
}
```

## Naming Conventions

### Files
- Snake case: `timer_service.rs`
- Module folders: `timer/mod.rs`
- Tests: `timer_tests.rs`

### Types
```rust
// Structs: PascalCase
pub struct TaskRepository;

// Enums: PascalCase
pub enum TaskStatus {
    Pending,
    InProgress,
}

// Traits: PascalCase
pub trait EventPublisher;

// Functions: snake_case
pub fn create_task() {}

// Constants: SCREAMING_SNAKE_CASE
pub const MAX_RETRIES: u32 = 3;
```

### Modules
```rust
// Module declaration
pub mod timer;
pub mod task;

// Use statements
use domain::timer::Timer;
use crate::adapters::task;
```

## Project-Specific Conventions

### Event Naming
- Format: `entity.action`
- Examples: `task.created`, `timer.started`

### DTO Pattern
```rust
// Infrastructure DTOs for serialization
#[derive(Serialize, Deserialize)]
pub struct TaskDto {
    pub id: String,
    pub title: String,
}

// Conversion
impl From<Task> for TaskDto {
    fn from(task: Task) -> Self {
        Self {
            id: task.id.to_string(),
            title: task.title,
        }
    }
}
```

### Command Pattern (Tauri)
```rust
#[tauri::command]
pub async fn create_task(
    title: String,
    state: State<'_, AppState>,
) -> Result<TaskDto> {
    // Implementation
}
```

### Repository Method Naming
- `create` - Create new entity
- `update` - Update existing
- `delete` - Remove entity
- `find_by_id` - Single lookup
- `find_all` - Get all
- `search` - Query with filters

## Configuration Management

### Default Implementation
```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            audio: AudioConfig::default(),
        }
    }
}
```

### Settings Override Pattern
```rust
pub struct EffectiveSettings {
    work_duration: Duration,
    break_duration: Duration,
}

impl EffectiveSettings {
    pub fn resolve(task_settings: Option<TaskSettings>, defaults: TaskDefaults) -> Self {
        // Merge logic
    }
}
```

## Dependency Injection

### Constructor Injection
```rust
pub struct CreateTaskUseCase {
    repository: Arc<dyn TaskRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl CreateTaskUseCase {
    pub fn new(
        repository: Arc<dyn TaskRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self { repository, event_publisher }
    }
}
```

### Bootstrap Pattern
```rust
// infra/src/bootstrap.rs
pub async fn create_app_state() -> AppState {
    let repository = Arc::new(SqliteTaskRepository::new(pool));
    let event_publisher = Arc::new(MemEventBus::new());
    
    AppState {
        repository,
        event_publisher,
    }
}
```