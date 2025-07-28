# 🧠 Pomotoro Domain Layer Learning Guide

Welcome to the heart of Pomotoro! This guide will take you on a comprehensive journey through the domain layer, where all the core business logic lives. By the end, you'll understand how domain-driven design (DDD) principles create a robust, maintainable foundation for the entire application.

## 🎯 Learning Objectives

After completing this guide, you will:
- ✅ Understand the bounded contexts and their interactions
- ✅ Master the event-driven architecture patterns
- ✅ Recognize DDD tactical patterns in practice
- ✅ Know how to test domain logic effectively
- ✅ Be able to extend domain functionality following established patterns

## 📋 Prerequisites

- Basic Rust knowledge (structs, traits, enums, Result types)
- Understanding of async/await patterns
- Familiarity with testing in Rust (`cargo test`)

**Estimated Learning Time: 6-8 hours**

---

## 🏗️ Domain Architecture Overview

The Pomotoro domain follows **Clean Architecture + DDD** principles with strict separation of concerns:

```
pomotoro-domain/
├── shared_kernel/    # Common building blocks across all domains
├── task/             # Task management bounded context
├── timer/            # Timer execution bounded context  
├── config/           # Configuration management bounded context
├── audio/            # Audio system bounded context
└── events.rs         # Domain event coordination
```

### Core Principles

1. **Pure Business Logic** - No I/O operations, no side effects
2. **Event-Driven Design** - Domains communicate via events
3. **Bounded Contexts** - Clear domain boundaries
4. **Rich Domain Models** - Entities contain business behavior

---

## 🚀 Guided Learning Path

### Phase 1: Shared Kernel Fundamentals (90 minutes)

The shared kernel contains the building blocks used across all domains. Start here to understand the foundation.

#### 1.1 Value Objects (`shared_kernel/value_objects/`)

**Start with:** `identifier.rs`
```bash
# Examine the base ID pattern
cat src/shared_kernel/value_objects/identifier.rs
```

**Key Concepts:**
- `EntityId<T>` - Type-safe entity identifiers  
- `EntityMarker` - Phantom types for ID safety
- UUID-based IDs with display formatting

**Next:** `tag.rs` - Simple value object for categorization
```rust
// Example: Task tags like "work", "personal", "urgent"
let tag = Tag::new("work".to_string())?;
```

**Then:** `timer_configuration.rs` - Configuration value objects
```rust
// Work/break durations with validation
let config = TimerConfiguration::new(
    Duration::minutes(25), // work
    Duration::minutes(5),  // short break  
    Duration::minutes(15), // long break
    4                      // cycles
)?;
```

#### 1.2 Domain Traits (`shared_kernel/traits/`)

**Essential Reading Order:**
1. `readable.rs` - Query abstractions
2. `writable.rs` - Command abstractions  
3. `searchable.rs` - Search capabilities

```bash
# Study the repository patterns
cat src/shared_kernel/traits/readable.rs
cat src/shared_kernel/traits/writable.rs
```

**Key Pattern:** Repository trait design for persistence abstraction

#### 1.3 Event System (`shared_kernel/events/`)

```bash
# Understand the event architecture
cat src/shared_kernel/events/domain_event.rs
cat src/shared_kernel/events/event_publisher.rs
```

**Practice Exercise:**
```bash
# Run shared kernel tests to see patterns in action
cargo test -p pomotoro-domain shared_kernel
```

---

### Phase 2: Task Domain Deep Dive (2-3 hours)

The task domain manages work items and their lifecycle. It's the most complex domain with rich business rules.

#### 2.1 Core Entity (`task/task.rs`)

```bash
# Study the main Task entity
cat src/task/task.rs | head -100
```

**Key Business Rules:**
- Tasks have max session limits
- Session counting and completion logic
- Status transitions (Active → Completed)
- Tags for organization

**Critical Methods to Understand:**
- `is_completed()` - Completion criteria
- `increment_session()` - Session tracking
- `can_start_session()` - Validation logic

#### 2.2 Task Builder Pattern (`task/builder.rs`)

```bash
# Learn the creation patterns
cat src/task/builder.rs
```

**Why Builder Pattern?**
- Complex object construction
- Default value application
- Validation during build process

**Practice:**
```rust
// Try building different task types
TaskBuilder::default_task().build()?
TaskBuilder::with_name_and_sessions("Focus Work".to_string(), 2).build()?
```

#### 2.3 Task Lifecycle (`task/status.rs`)

```bash
cat src/task/status.rs
```

**Status Flow:**
```
Active → InProgress → Completed
   ↓         ↓           ↓
 Paused → Paused → (terminal)
```

#### 2.4 Task Events (`task/events/`)

**Essential Events to Study:**
```bash
# Core lifecycle events
cat src/task/events/task_created.rs
cat src/task/events/task_session_completed.rs  
cat src/task/events/task_completed.rs

# Workflow events
cat src/task/events/session_transition_completed.rs
cat src/task/events/task_switch_workflow_completed.rs
```

**Event Flow Example:**
```
Task Session Started → Timer Started → Work Session Completed → 
Task Session Completed → Task Status Changed → (Optional) Task Completed
```

#### 2.5 Domain Services (`task/cycling_srv.rs`)

```bash
# Study task cycling strategies  
cat src/task/cycling_srv.rs
```

**Key Concepts:**
- Round-robin task cycling
- Automatic vs manual task switching
- Queue management for active tasks

**Practice Exercise:**
```bash
# Run task domain tests
cargo test -p pomotoro-domain task::test
cargo test -p pomotoro-domain task::builder
```

---

### Phase 3: Timer Domain Exploration (2 hours)

The timer domain handles time tracking and phase transitions.

#### 3.1 Timer State Management (`timer/state.rs`)

```bash
# Core timer state
cat src/timer/state.rs | head -150
```

**State Components:**
- Current phase (Work/ShortBreak/LongBreak)
- Remaining time tracking
- Session counting  
- Timer status (Running/Paused/Stopped)

#### 3.2 Phase Management (`timer/phase.rs`)

```bash
cat src/timer/phase.rs
```

**Phase Types:**
- `Work` - 25-minute focus sessions
- `ShortBreak` - 5-minute breaks
- `LongBreak` - 15-minute breaks after 4 cycles

#### 3.3 Timer Entity (`timer/timer.rs`)

```bash
# Study the timer entity
cat src/timer/timer.rs
```

**Key Operations:**
- `start()` - Begin timing
- `pause()` - Suspend timer
- `reset()` - Return to phase start
- `complete_phase()` - Finish current phase

#### 3.4 Phase Transitions (`timer/phase_transition_srv.rs`)

```bash
# Complex business logic here
cat src/timer/phase_transition_srv.rs
```

**Transition Rules:**
- Work → Short Break (normal cycle)
- Work → Long Break (after 4th session)
- Break → Work (continue cycle)

#### 3.5 Timer Events (`timer/events/`)

**Study Order:**
```bash
# State changes
cat src/timer/events/timer_started.rs
cat src/timer/events/timer_paused.rs
cat src/timer/events/phase_completed.rs

# Session events  
cat src/timer/events/work_session_started.rs
cat src/timer/events/work_session_completed.rs
cat src/timer/events/break_session_completed.rs
```

**Practice Exercise:**
```bash
# Test timer domain logic
cargo test -p pomotoro-domain timer::test
cargo test -p pomotoro-domain timer::phase
```

---

### Phase 4: Configuration Domain (1 hour)

Configuration management for customizable behavior.

#### 4.1 Master Configuration (`config/config.rs`)

```bash
cat src/config/config.rs
```

**Configuration Areas:**
- Timer defaults (work/break durations)
- Audio preferences
- Notification settings
- Appearance options

#### 4.2 Task Defaults (`config/task_defaults.rs`)

```bash
cat src/config/task_defaults.rs
```

**Default Value Management:**
- New task creation defaults
- Timer configuration inheritance
- Audio configuration defaults

#### 4.3 Specialized Configs

```bash
# Study each configuration domain
cat src/config/audio.rs         # Audio preferences
cat src/config/notification.rs  # Notification settings  
cat src/config/appearance.rs    # UI appearance
```

**Practice:**
```bash
cargo test -p pomotoro-domain config
```

---

### Phase 5: Audio Domain (45 minutes)

Audio system abstractions for focus and notification sounds.

#### 5.1 Audio Service Interface (`audio/audio_srv.rs`)

```bash
cat src/audio/audio_srv.rs
```

**Key Abstractions:**
- Playback control (play/pause/stop)
- Volume management
- Asset loading

#### 5.2 Audio Assets (`audio/asset.rs`)

```bash
cat src/audio/asset.rs
```

**Asset Types:**
- Background focus sounds
- Notification alerts
- Custom user audio

#### 5.3 Audio Library (`audio/library.rs`)

```bash
cat src/audio/library.rs
```

**Practice:**
```bash
cargo test -p pomotoro-domain audio
```

---

### Phase 6: Event System Integration (1 hour)

Understanding how events coordinate between domains.

#### 6.1 Event Coordination (`events.rs`)

```bash
cat src/events.rs
```

#### 6.2 Cross-Domain Event Flows

**Key Integration Points:**

1. **Timer → Task Integration:**
   ```
   WorkSessionCompleted → TaskSessionCompleted → TaskStatusChanged
   ```

2. **Task → Timer Integration:**
   ```
   TaskSwitchWorkflowCompleted → ActiveTaskSwitched
   ```

3. **Configuration Changes:**
   ```
   ConfigUpdated → (Domain reconfiguration)
   ```

**Trace Exercise:**
```bash
# Find all events that cross domain boundaries
grep -r "SessionCompleted" src/
grep -r "TaskSwitched" src/
```

---

## 🧪 Testing Your Understanding

### Domain Test Execution

```bash
# Run all domain tests
cargo test -p pomotoro-domain

# Test specific domains
cargo test -p pomotoro-domain task
cargo test -p pomotoro-domain timer  
cargo test -p pomotoro-domain shared_kernel

# Verbose output for understanding
cargo test -p pomotoro-domain -- --nocapture

# Test specific functionality
cargo test -p pomotoro-domain task::test_task_completion
cargo test -p pomotoro-domain timer::test_phase_transitions
```

### Understanding Test Patterns

**Unit Tests Pattern:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_rule() {
        // Arrange: Set up domain objects
        let mut task = Task::new("Test".to_string(), 2)?;
        
        // Act: Execute business logic
        let result = task.increment_session();
        
        // Assert: Verify business rules
        assert!(result.is_ok());
        assert_eq!(task.current_sessions, 1);
    }
}
```

### Domain Logic Validation Exercises

1. **Task Completion Rules:**
   ```bash
   cargo test -p pomotoro-domain task::test_task_completion_logic
   ```

2. **Timer Phase Transitions:**
   ```bash
   cargo test -p pomotoro-domain timer::test_phase_transitions
   ```

3. **Event Generation:**
   ```bash
   cargo test -p pomotoro-domain event_generation
   ```

---

## 🔍 Key Patterns & Concepts Reference

### Domain Entity Pattern
```rust
// Entities have identity and business behavior
impl Task {
    pub fn complete_session(&mut self) -> Result<Vec<DomainEvent>, Error> {
        // Business rule validation
        if self.is_completed() {
            return Err(Error::TaskAlreadyCompleted);
        }
        
        // State change
        self.current_sessions += 1;
        
        // Event generation
        let mut events = vec![TaskSessionCompleted::new(self.id)];
        
        if self.is_completed() {
            events.push(TaskCompleted::new(self.id));
        }
        
        Ok(events)
    }
}
```

### Value Object Pattern
```rust
// Value objects are immutable and equality-based
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Duration {
    seconds: u32,
}

impl Duration {
    pub fn minutes(m: u32) -> Self {
        Self { seconds: m * 60 }
    }
    
    // Rich behavior
    pub fn is_valid_work_duration(&self) -> bool {
        self.seconds >= 5 * 60 && self.seconds <= 60 * 60
    }
}
```

### Repository Pattern
```rust
// Abstract data access
#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn save(&self, task: &Task) -> Result<(), Error>;
    async fn find_by_id(&self, id: TaskId) -> Result<Option<Task>, Error>;
    async fn get_active_tasks(&self) -> Result<Vec<Task>, Error>;
}
```

### Domain Service Pattern
```rust
// Domain service for complex business logic
pub trait TaskCyclerService {
    fn get_next_task(&self, tasks: &[Task], current: Option<TaskId>) -> Option<TaskId>;
}
```

---

## 💡 Learning Tips & Best Practices

### Reading Code Effectively

1. **Start with tests** - They show intended behavior
2. **Follow the data flow** - Trace how data transforms
3. **Understand the invariants** - What rules must always be true
4. **Look for error handling** - What can go wrong and how it's handled

### Common Pitfalls to Avoid

❌ **Don't skip the shared kernel** - It's the foundation
❌ **Don't ignore the tests** - They document expected behavior  
❌ **Don't focus only on happy paths** - Study error scenarios
❌ **Don't try to understand everything at once** - Follow the phases

### Debugging Domain Logic

```bash
# Add debug output to tests
cargo test -p pomotoro-domain -- --nocapture

# Run specific failing tests
cargo test -p pomotoro-domain test_name -- --exact

# Check for compilation issues
cargo check -p pomotoro-domain
```

---

## 🎯 Learning Milestones

Mark your progress as you master each area:

- [ ] **Shared Kernel Mastery** - Understand value objects, traits, and events
- [ ] **Task Domain Expertise** - Can explain task lifecycle and business rules  
- [ ] **Timer Domain Knowledge** - Understand phase transitions and timing logic
- [ ] **Configuration Understanding** - Know how defaults and customization work
- [ ] **Audio Domain Basics** - Understand audio abstractions
- [ ] **Event System Mastery** - Can trace cross-domain event flows
- [ ] **Testing Proficiency** - Can run and understand domain tests
- [ ] **Architecture Comprehension** - See how DDD principles are applied

---

## 🚀 Next Steps

Once you've mastered the domain layer:

1. **Explore Application Layer** - See how use cases orchestrate domain logic
2. **Study Infrastructure** - Understand how domain abstractions are implemented  
3. **Examine UI Integration** - Learn how the frontend uses domain concepts
4. **Contribute** - Apply your domain knowledge to add new features

---

## 📚 Additional Resources

- [Domain-Driven Design by Eric Evans](https://domainlanguage.com/ddd/)
- [Clean Architecture by Robert Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Rust for Rustaceans](https://rust-for-rustaceans.com/) - Advanced Rust patterns

Happy learning! 🦀✨

---

*This guide is a living document. As the domain evolves, so will this guide. Contributions and improvements are welcome!*