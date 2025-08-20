# MVP 2.0 - Pomotoro Task Management Specification

## Overview

MVP 2.0 builds upon the solid foundation established in MVP 1.0, introducing task management capabilities that transform Pomotoro from a simple timer into a comprehensive productivity tool. This version maintains the core pomodoro methodology while adding the flexibility to manage multiple tasks and customize workflows.

## Core Features Evolution

### Building on MVP 1.0 Foundation
- ✅ Robust 25/5/15 minute timer system
- ✅ Session tracking and cycle management  
- ✅ Desktop notifications and persistence
- ✅ Clean Leptos/Tauri architecture

### New in MVP 2.0

## 1. Task Management System 📋

**Default Task**
- Pre-created "Focus Session" task (non-deletable)
- Default max sessions: 4 (follows traditional pomodoro cycle)
- Serves as fallback when no custom tasks exist
- Uses standard 25/5/15 minute configuration

**Custom Tasks**
- Create, read, update, delete operations
- Each task maintains independent session tracking
- Configurable max sessions per task (default: 1)
- Task-specific pomodoro configurations
- Optional task descriptions and notes

**Task Selection**
- Active task selector in main interface
- Current task displayed prominently during sessions
- Task-specific progress tracking
- Automatic task completion when max sessions reached

**Tag System**
- Assign multiple tags to tasks for organization
- Filter and search tasks by tags
- Tag-based task grouping in UI
- Predefined tags: work, personal, learning, health

## 2. Enhanced Configuration System ⚙️

**Per-Task Timing Configuration**
- Custom work session duration (5-60 minutes)
- Custom short break duration (1-30 minutes)
- Custom long break duration (5-60 minutes)
- Override cycles per task (1-8 sessions before long break)

**Global Configuration**
- Default timing settings for new tasks
- Task cycling behavior (auto-advance vs manual selection)
- Max sessions per task default value
- Notification preferences

**Audio Configuration (Per Task)**
- Custom notification sounds for work/break transitions
- Volume control per task
- "Toro audio" - background focus sounds during work sessions
- Mute options for specific tasks

## 3. Advanced Notification System 🔔

**Screen Blocking Feature**
- Optional full-screen overlay during work sessions
- Customizable blocking messages
- Temporary disable for urgent interruptions
- Per-task blocking configuration

**Enhanced System Notifications**
- Rich notifications with task information
- Action buttons (start break, extend session, switch task)
- Progress summaries in notifications
- Task completion celebrations

**Toro Audio System**
- Background soundscape during focus sessions
- Nature sounds, white noise, binaural beats
- Task-specific audio profiles
- Fade in/out with session transitions

## 4. Multi-Task Session Management 📊

**Task Cycling**
- Automatic progression through task queue
- Manual task switching between sessions
- Round-robin cycling through active tasks
- Skip completed tasks in cycle

**Session Tracking Evolution**
- Per-task session counters
- Daily/weekly task completion stats
- Total focused time per task
- Session completion streaks

**Workflow States**
- Active task (currently running)
- Queued tasks (in cycle rotation)
- Completed tasks (max sessions reached)
- Paused tasks (temporarily excluded from cycle)

## User Interface Evolution 🎨

### Task Management Panel
- Collapsible task list sidebar
- Quick task creation inline
- Task status indicators (active, queued, completed)
- Drag-and-drop task reordering

### Enhanced Timer Display
- Current task name prominently displayed
- Task-specific progress indicators
- Session counter: "Task X: Session Y/Z"
- Task completion progress bar

### Configuration Dialogs
- Task settings modal with timing configuration
- Audio settings with preview capability
- Tag management interface
- Global preferences panel

### Quick Actions
- Task switching without stopping timer
- Quick task creation from notification
- Keyboard shortcuts for common actions
- Context menus for task operations

## Technical Architecture Evolution

### Data Model Extensions
```rust
// TODO: Extend TimerState for task management
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub current_sessions: u8,
    pub tags: Vec<String>,
    pub config: TaskConfig,
    pub audio_config: AudioConfig,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub struct TaskConfig {
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub sessions_until_long_break: u8,
    pub enable_screen_blocking: bool,
}
```

### State Management Evolution
- Task repository with SQLite backend
- Task-aware timer state management
- Event system for task lifecycle events
- Configuration persistence per task

### Audio System Integration
- Tauri audio plugin integration
- Asset management for audio files
- Volume and playback control
- Task-specific audio profiles

## User Stories

### As a user, I want to:

1. **Manage multiple work tasks**
   - Create tasks for different projects
   - Set appropriate session counts per task
   - Track progress on each task independently
   - Switch between tasks without losing progress

2. **Customize my workflow**
   - Adjust timer durations for different types of work
   - Set shorter sessions for creative work, longer for deep focus
   - Enable screen blocking for high-concentration tasks
   - Choose appropriate background audio per task

3. **Organize my tasks**
   - Tag tasks by project, priority, or type
   - Filter task lists by tags
   - Archive completed tasks
   - View task completion statistics

4. **Enhanced focus experience**
   - Block distracting applications during work sessions
   - Enjoy background soundscapes while working
   - Receive rich notifications with task context
   - Cycle through my task queue automatically

5. **Track productivity across tasks**
   - See daily/weekly task completion
   - Monitor focused time per project
   - Identify optimal session lengths per task type
   - Celebrate task completion milestones

## Implementation Phases

### Phase 1: Core Task Management (Week 1-2)
1. Task data model and repository
2. Basic task CRUD operations
3. Task selection and switching
4. Enhanced timer state for task awareness

### Phase 2: Configuration System (Week 2-3)
1. Per-task timing configuration
2. Global configuration management
3. Configuration persistence
4. UI for task settings

### Phase 3: Enhanced Notifications (Week 3-4)
1. Screen blocking implementation
2. Rich notification system
3. Task-aware notification content
4. Action buttons in notifications

### Phase 4: Audio System (Week 4-5)
1. Audio plugin integration
2. Background soundscape system
3. Per-task audio configuration
4. Asset management and loading

### Phase 5: UI Polish & Integration (Week 5-6)
1. Task management panel
2. Enhanced timer display
3. Keyboard shortcuts
4. Performance optimization and testing

## Success Criteria

### Functional Requirements
- [ ] Create, edit, delete custom tasks
- [ ] Configure timing per task (5-60 min work sessions)
- [ ] Tag-based task organization and filtering
- [ ] Task-specific session tracking and limits
- [ ] Automatic task cycling with manual override
- [ ] Per-task screen blocking functionality
- [ ] Background audio during work sessions
- [ ] Rich notifications with task context
- [ ] Task completion statistics and tracking

### User Experience
- [ ] Intuitive task management without disrupting timer flow
- [ ] Smooth transitions between tasks and configurations
- [ ] Clear visual hierarchy showing active vs queued tasks
- [ ] Helpful audio cues without being distracting
- [ ] Screen blocking that enhances rather than frustrates focus

### Performance & Reliability
- [ ] Task switching under 500ms
- [ ] Audio loading/playback without UI lag
- [ ] Reliable persistence of task data and configurations
- [ ] Memory usage <75MB with 50+ tasks
- [ ] Backwards compatibility with MVP 1.0 state files

## Migration Strategy

### From MVP 1.0
- Preserve existing timer state and session counts
- Create default task from current session
- Import session history into default task
- Maintain configuration file compatibility

### Data Migration
```rust
// TODO: Migration utility for MVP 1.0 → 2.0
pub fn migrate_from_mvp1(old_state: &TimerState) -> (Vec<Task>, TimerState) {
    let default_task = Task::new_default()
        .with_sessions(old_state.session_count);
    
    let new_state = TimerState::new()
        .with_active_task(default_task.id)
        .with_phase(old_state.phase);
        
    (vec![default_task], new_state)
}
```

## Timeline Estimate

- **Phase 1**: 2 weeks (Task foundation)
- **Phase 2**: 1 week (Configuration system)
- **Phase 3**: 1 week (Enhanced notifications)
- **Phase 4**: 1 week (Audio system)
- **Phase 5**: 1 week (UI polish & testing)

**Total MVP 2.0 Development**: 6 weeks

## Definition of Done

MVP 2.0 is complete when:
1. All success criteria are met
2. Migration from MVP 1.0 works seamlessly
3. Performance benchmarks are achieved
4. User testing validates task management workflow
5. Audio system provides value without distraction
6. Screen blocking enhances focus without frustration
7. Documentation reflects new capabilities
8. Ready for advanced productivity use cases

## Future Evolution Paths

### MVP 2.1 Candidates
- Task templates and quick creation
- Task time estimates vs actual tracking
- Integration with external task managers
- Team/collaborative task management
- Advanced statistics and productivity insights

### Beyond MVP 2.x
- AI-powered task prioritization
- Calendar integration
- Cross-device synchronization
- Pomodoro technique variations (52/17, 90-minute deep work)
- Integration with time tracking services