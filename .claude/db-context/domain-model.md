# Domain Model Reference

## Core Entities

### Timer
**Location**: `domain/src/timer/`

**Key Components**:
- `Timer` - Main timer entity
- `TimerState` - Current state (duration, phase, status)
- `Phase` - Work or Break
- `TimerStatus` - Idle, Running, Paused
- `StateTransitions` - State machine logic

**Key Methods**:
- `start()` - Begin timer session
- `pause()` - Pause current session
- `reset()` - Reset to initial state
- `skip_phase()` - Move to next phase
- `tick()` - Update timer (called every second)

**Events Emitted**:
- `TimerStarted`
- `TimerPaused`
- `TimerReset`
- `PhaseCompleted`
- `PhaseSkipped`
- `WorkSessionStarted/Completed`
- `BreakSessionStarted/Completed`

### Task
**Location**: `domain/src/task/`

**Key Components**:
- `Task` - Task entity with title, description, status
- `TaskId` - Unique identifier (UUID)
- `TaskStatus` - Pending, InProgress, Completed
- `TaskSettings` - Per-task timer configuration
- `EffectiveSettings` - Resolved settings (task + defaults)

**Key Methods**:
- `Task::builder()` - Create new task
- `complete_session()` - Increment session count
- `update_status()` - Change task status
- `apply_settings()` - Set custom timer config

**Repository Interface**:
```rust
trait TaskRepository {
    async fn create(&self, task: Task) -> Result<Task>
    async fn update(&self, task: Task) -> Result<Task>
    async fn delete(&self, id: TaskId) -> Result<()>
    async fn find_by_id(&self, id: TaskId) -> Result<Option<Task>>
    async fn find_all(&self) -> Result<Vec<Task>>
    async fn search(&self, query: &str, status: Option<TaskStatus>) -> Result<Vec<Task>>
}
```

### Config
**Location**: `domain/src/config/`

**Components**:
- `Config` - Root configuration object
- `GeneralConfig` - Theme, auto-start, etc.
- `AudioConfig` - Volume, enabled sounds
- `NotificationConfig` - Position, enabled
- `TaskDefaults` - Default timer durations
- `AppearanceConfig` - UI preferences

**Repository Interface**:
```rust
trait ConfigRepository {
    async fn get(&self) -> Result<Config>
    async fn update(&self, config: Config) -> Result<Config>
    async fn reset(&self) -> Result<Config>
}
```

### Audio
**Location**: `domain/src/audio/`

**Components**:
- `AudioAsset` - Individual sound file
- `AudioCategory` - Background or Notification
- `AudioLibrary` - Collection of assets
- `PlaybackRequest` - Play audio command
- `AudioService` - Audio playback interface

**Service Interface**:
```rust
trait AudioService {
    async fn play(&self, request: PlaybackRequest) -> Result<PlaybackHandle>
    async fn stop(&self, handle: PlaybackHandle) -> Result<()>
    async fn set_volume(&self, handle: PlaybackHandle, volume: f32) -> Result<()>
}
```

## Value Objects

### Shared Kernel
**Location**: `domain/src/shared_kernel/`

- `EntityId<T>` - Type-safe entity identifiers
- `Timestamp` - Time tracking
- `TimerConfiguration` - Work/break durations
- `Tag` - Categorization tags
- `Event` - Base event trait
- `EventPublisher` - Event publishing interface

### Timer Value Objects
- `Duration` - Time duration wrapper
- `Phase` - Work | Break enum
- `TransitionType` - Manual | Automatic
- `TransitionResult` - State change outcome

### Task Value Objects
- `TaskCyclingStrategy` - How to select next task
- `SessionCount` - Number of completed sessions
- `Priority` - Task priority level

## Event Flow

### Timer Events
```
User Action → Timer Domain → Event Published → Handlers
                 ↓
          State Change
                 ↓
          Return Result
```

### Task Events
```
Task Operation → Repository → Event Published → Side Effects
                    ↓
              Database Update
                    ↓
              Return Updated Entity
```

## Domain Rules

### Timer Rules
1. Timer must be idle to start
2. Can only pause when running
3. Phase skip advances to next phase
4. Work sessions followed by breaks
5. Automatic transitions configurable

### Task Rules
1. Only one task active at a time
2. Tasks track session completion
3. Status changes trigger events
4. Deleted tasks cannot be recovered
5. Task settings override defaults

### Configuration Rules
1. Config always has valid defaults
2. Import merges with existing config
3. Reset restores factory defaults
4. Audio volume: 0.0 to 1.0 range

## Aggregate Boundaries

### Timer Aggregate
- Root: Timer
- Includes: TimerState, Phase, Status
- Boundary: Timer operations only

### Task Aggregate
- Root: Task
- Includes: TaskSettings, Status
- Boundary: Individual task operations

### Config Aggregate
- Root: Config
- Includes: All config sections
- Boundary: Configuration as a whole

## Domain Services

### TaskCyclerService
- Manages task cycling strategies
- Selects next task based on rules
- Handles exhausted queue scenarios

### TimerService (Infrastructure)
- Manages timer tick intervals
- Handles system timer integration
- Provides timer state persistence

## Error Handling

### Domain Errors
```rust
pub enum Error {
    NotFound(String),
    ValidationError(String),
    InvalidState(String),
    OperationFailed(String),
}
```

### Timer Errors
- InvalidTransition
- AlreadyRunning
- NotRunning
- InvalidDuration

### Task Errors
- TaskNotFound
- DuplicateTask
- InvalidStatus
- CyclingExhausted