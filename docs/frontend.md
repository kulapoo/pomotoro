# Frontend Architecture - Pomotoro

## Overview

Pomotoro's frontend is built with **Leptos 0.8.3** using Client-Side Rendering (CSR), integrated with **Tauri** for native desktop functionality. The architecture follows Domain-Driven Design principles with clean separation between UI components and business logic.

## Technology Stack

- **UI Framework**: Leptos (Rust-based reactive UI)
- **Runtime**: WebAssembly (WASM) 
- **Native Integration**: Tauri (Rust-based desktop app framework)
- **State Management**: Leptos Signals + Resources
- **Domain Layer**: `pomotoro-domain` crate for business logic

## Architecture Principles

1. **Domain-First Design**: UI components depend on domain types and contracts
2. **Reactive State Management**: Leptos signals for fine-grained reactivity  
3. **Component Composition**: Reusable components with clear boundaries
4. **Async-First**: All domain interactions are async via Tauri commands
5. **Type Safety**: Full type safety from domain to UI layer

## Directory Structure

```
src/
├── app.rs                  # Root application component
├── main.rs                 # WASM entry point and setup
├── pages/                  # Page-level components and routing
│   ├── timer/         
│   │   ├── timer_page.rs       # Main timer page component
│   │   ├── timer_state.rs      # Page-specific state management
│   │   ├── timer_display.rs    # Timer display component
│   │   ├── timer_controls.rs   # Timer control buttons
│   │   ├── session_counter.rs  # Session progress display
│   │   └── phase_indicator.rs  # Work/break phase indicator
│   └── task/          
│       ├── task_page.rs        # Main task management page
│       ├── task_list.rs        # Task list component
│       ├── task_item.rs        # Individual task item
│       ├── task_form.rs        # Task creation/edit form
│       └── task_filters.rs     # Task filtering and search
├── components/             # Reusable UI components (domain-agnostic)
│   ├── circular_progress.rs   # Generic progress indicator
│   ├── modal.rs               # Generic modal component
│   ├── button.rs              # Styled button variants
│   ├── screen_blocker.rs      # Full-screen overlay
│   └── form_controls/         # Input, select, checkbox components
│       ├── mod.rs
│       ├── input.rs
│       ├── select.rs
│       └── checkbox.rs
├── domain-components/      # Shared domain-aware components
│   └── settings/           # Settings components (shared across pages)
│       ├── mod.rs
│       ├── timer_config.rs     # Timer configuration UI
│       ├── audio_config.rs     # Audio settings UI
│       ├── global_settings_panel.rs  # Global app settings
│       └── task_settings_modal.rs    # Task-specific settings
└── store/                  # State management and events
    ├── mod.rs
    ├── timer_state.rs      # Timer state management
    ├── config.rs           # Configuration state
    └── events.rs           # Event system and Tauri integration
```

## Component Categories

### Pages (`pages/`)

Page-level components that compose entire application views. Components within each page directory are specific to that page and handle page-level state and composition.

**Design Principles:**
- Each page has its own directory (`timer/`, `task/`)
- Page-specific components stay within their page directory
- Components can be promoted to `domain-components/` when they become reusable across pages
- Handle page-level routing, state management, and composition

**Current Structure:**

#### Timer Page (`pages/timer/`)
```rust
// timer_page.rs - Main timer page composition
#[component]
pub fn TimerPage() -> impl IntoView {
    view! {
        <div class="timer-page">
            <TimerDisplay />
            <TimerControls />
            <SessionCounter />
        </div>
    }
}

// timer_display.rs - Timer display specific to timer page
#[component]
pub fn TimerDisplay(
    timer_state: ReadSignal<TimerState>,
    timer_with_task: ReadSignal<TimerStateWithTask>
) -> impl IntoView {
    // Timer display logic
}
```

#### Task Page (`pages/task/`)
```rust
// task_page.rs - Main task management page
#[component] 
pub fn TaskPage() -> impl IntoView {
    view! {
        <div class="task-page">
            <TaskFilters />
            <TaskList />
            <TaskForm />
        </div>
    }
}
```

### Components (`components/`)

Reusable, domain-agnostic UI components that can be used across different pages and contexts. These are "dumb" components focused purely on presentation.

**Design Principles:**
- No direct domain logic or Tauri command invocation
- Accept data via props, emit events via callbacks
- Focused on presentation and user interaction
- Completely reusable across different contexts
- No knowledge of business rules

**Current Components:**
- `circular_progress.rs` - Generic circular progress indicator
- `screen_blocker.rs` - Full-screen overlay component
- `modal.rs` - Generic modal wrapper
- `button.rs` - Styled button variants
- `form_controls/` - Generic form inputs

**Example:**
```rust
// components/button.rs
#[derive(Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
}

#[component]
pub fn Button(
    #[prop(into)] text: String,
    #[prop(optional)] variant: Option<ButtonVariant>,
    #[prop(optional)] disabled: Option<bool>,
    #[prop(optional)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    let variant = variant.unwrap_or(ButtonVariant::Primary);
    let disabled = disabled.unwrap_or(false);
    
    view! {
        <button 
            class=format!("btn btn-{:?}", variant).to_lowercase()
            disabled=disabled
            on:click=move |_| {
                if let Some(callback) = on_click {
                    callback(());
                }
            }
        >
            {text}
        </button>
    }
}
```

### Domain Components (`domain-components/`)

Shared domain-aware components that understand business logic and can be reused across multiple pages. These components have knowledge of domain types and operations.

**Design Principles:**
- Can invoke Tauri commands and handle domain operations
- Work directly with domain types
- Reusable across multiple pages
- Encapsulate domain-specific UI logic
- Handle domain validation and business rules

**Current Structure:**

#### Settings (`domain-components/settings/`)
Settings components are shared across pages since timer settings, task settings, and global settings can be accessed from different parts of the application.

```rust
// settings/timer_config.rs
#[component]
pub fn TimerConfig(
    #[prop(optional)] task_id: Option<TaskId>,
) -> impl IntoView {
    // Timer configuration component that can be used in:
    // - Global settings page
    // - Task-specific settings modal
    // - Timer page settings panel
}

// settings/global_settings_panel.rs  
#[component]
pub fn GlobalSettingsPanel() -> impl IntoView {
    // Global settings that can be opened from any page
}
```

**Component Promotion:**
Components start in their respective page directories. When a component proves useful across multiple pages, it can be promoted to `domain-components/`:

```rust
// Before: pages/timer/session_counter.rs
// After promotion: domain-components/timer/session_counter.rs
// Can now be used in timer page, task page, and dashboard
```

## State Management

### Signal-Based Reactivity

Leptos signals provide fine-grained reactivity without unnecessary re-renders:

```rust
// Reactive state
let (timer_state, set_timer_state) = signal(TimerState::default());
let (tasks, set_tasks) = signal(Vec::<Task>::new());

// Derived signals
let progress_percentage = Signal::derive(move || {
    timer_state.get().get_progress_percentage()
});

// Effects for side effects
Effect::new(move |_| {
    let state = timer_state.get();
    // React to state changes
});
```

### Resource Pattern

For async data loading and caching:

```rust
// store/config.rs - Configuration resource
#[derive(Clone)]
pub struct ConfigResource {
    pub timer_config: Resource<(), TimerConfig>,
    pub audio_config: Resource<(), AudioConfig>,
}

impl ConfigResource {
    pub fn new() -> Self {
        let timer_config = Resource::new(|| (), |_| async {
            invoke_command(events::config::GET_TIMER_CONFIG, JsValue::NULL).await
        });
        
        let audio_config = Resource::new(|| (), |_| async {
            invoke_command(events::config::GET_AUDIO_CONFIG, JsValue::NULL).await  
        });
        
        Self { timer_config, audio_config }
    }
}
```

### Page-Specific State

Each page manages its own state within its directory:

```rust
// pages/timer/timer_state.rs
pub struct TimerPageState {
    pub timer_state: ReadSignal<TimerState>,
    pub timer_with_task: ReadSignal<TimerStateWithTask>,
    pub is_settings_open: ReadSignal<bool>,
}

impl TimerPageState {
    pub fn new() -> Self {
        let (timer_state, set_timer_state) = signal(TimerState::default());
        let (timer_with_task, set_timer_with_task) = signal(TimerStateWithTask::new());
        let (is_settings_open, set_is_settings_open) = signal(false);
        
        // Setup event listeners
        setup_timer_events(set_timer_state);
        
        Self {
            timer_state,
            timer_with_task,
            is_settings_open,
        }
    }
}
```

## Domain Integration

### Tauri Command Integration

All domain operations go through Tauri commands:

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

// Standardized command invocation
pub async fn invoke_command<T>(command: &str, args: JsValue) -> Result<T, String> 
where 
    T: serde::de::DeserializeOwned 
{
    let result = invoke(command, args).await;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| e.to_string())
}

// Usage in components
pub async fn start_timer() -> Result<TimerState, String> {
    invoke_command(events::timer::START, JsValue::NULL).await
}
```

### Domain Type Usage

Frontend components work directly with domain types:

```rust
use pomotoro_domain::{
    TimerState, TimerStateWithTask, Task, TaskId, 
    TaskStatus, Phase, TimerStatus
};

// Components accept domain types as props
#[component]
pub fn TimerDisplay(
    timer_state: ReadSignal<TimerState>,
    timer_with_task: ReadSignal<TimerStateWithTask>
) -> impl IntoView {
    // Use domain methods directly
    view! {
        <div class="timer-display">
            <span class="phase">{move || timer_state.get().get_phase_name()}</span>
            <span class="time">{move || timer_with_task.get().format_time()}</span>
            <span class="task">{move || timer_with_task.get().get_active_task_name()}</span>
        </div>
    }
}
```

## Development Workflow

### Adding New Components

1. **Determine Component Type:**
   - **UI Component**: Goes in `components/` if domain-agnostic
   - **Page Component**: Goes in `pages/[page]/` if page-specific
   - **Domain Component**: Goes in `domain-components/` if shared and domain-aware

2. **Start Page-Specific:**
   - New components usually start in their page directory
   - Promote to `domain-components/` when reused across pages
   - Move to `components/` if they become domain-agnostic

3. **Component Structure:**
```rust
// Standard component template
#[component]
pub fn ComponentName(
    // Required props first
    #[prop(into)] required_prop: String,
    
    // Optional props with defaults
    #[prop(optional)] optional_prop: Option<bool>,
    
    // Callbacks for events
    #[prop(optional)] on_event: Option<Callback<EventData>>,
) -> impl IntoView {
    // Component logic
    view! {
        // Component template
    }
}
```

### Adding New Pages

1. **Create Page Directory:**
```bash
mkdir src/pages/new_page
```

2. **Create Page Structure:**
```rust
// pages/new_page/mod.rs
mod new_page_component;
mod page_state;

pub use new_page_component::NewPage;
pub use page_state::NewPageState;

// pages/new_page/new_page_component.rs
#[component]
pub fn NewPage() -> impl IntoView {
    let page_state = NewPageState::new();
    
    view! {
        <div class="new-page">
            // Page content
        </div>
    }
}
```

3. **Add Page-Specific State:**
```rust
// pages/new_page/page_state.rs
pub struct NewPageState {
    // Page-specific state
}

impl NewPageState {
    pub fn new() -> Self {
        // Initialize state
        Self {}
    }
}
```

### Promoting Components

When a component becomes reusable across pages:

1. **Move from page to domain-components:**
```bash
# Move the component
mv src/pages/timer/session_counter.rs src/domain-components/timer/
```

2. **Update imports:**
```rust
// Before
use crate::pages::timer::SessionCounter;

// After  
use crate::domain_components::timer::SessionCounter;
```

3. **Update the component to be more generic:**
```rust
// Make props more flexible for different contexts
#[component]
pub fn SessionCounter(
    current_sessions: ReadSignal<u8>,
    max_sessions: ReadSignal<u8>,
    #[prop(optional)] show_labels: Option<bool>,
) -> impl IntoView {
    // More generic implementation
}
```

## Error Handling

### Error Boundaries

Use Leptos error boundaries for graceful error handling:

```rust
#[component]
pub fn App() -> impl IntoView {
    view! {
        <ErrorBoundary
            fallback=|errors| view! {
                <div class="error-container">
                    <h2>"Something went wrong"</h2>
                    <ul>
                        {errors.get().into_iter().map(|(_, error)| {
                            view! { <li>{error.to_string()}</li> }
                        }).collect::<Vec<_>>()}
                    </ul>
                </div>
            }
        >
            <Router>
                <Routes>
                    <Route path="/timer" view=TimerPage />
                    <Route path="/tasks" view=TaskPage />
                </Routes>
            </Router>
        </ErrorBoundary>
    }
}
```

### Command Error Handling

Standardized error handling for Tauri commands:

```rust
// store/events.rs
pub async fn handle_command_error<T>(
    command: &str,
    result: Result<T, String>
) -> Result<T, AppError> {
    match result {
        Ok(data) => Ok(data),
        Err(e) => {
            web_sys::console::error_1(&format!("Command {} failed: {}", command, e).into());
            Err(AppError::CommandFailed(command.to_string(), e))
        }
    }
}
```

## Testing Strategy

### Component Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use leptos::*;
    
    #[test]
    fn test_timer_display_renders() {
        let timer_state = create_signal(TimerState::default()).0;
        let timer_with_task = create_signal(TimerStateWithTask::new()).0;
        
        let view = TimerDisplay { timer_state, timer_with_task };
        // Test rendering logic
    }
}
```

### Integration Testing
Test component interaction with domain layer through Tauri commands.

## Future Enhancements

### Planned Features (MVP 2.0)
- **Multi-page routing** with leptos-router
- **Advanced task management** page with filtering and search
- **Enhanced settings** with per-task configuration
- **Rich notifications** with action buttons
- **Audio system integration** with background soundscapes

### Architecture Evolution
- **Component library expansion** in `components/`
- **More domain components** as features become reusable
- **Page-specific optimizations** for performance
- **Theme system** for customization
- **Accessibility enhancements** following WCAG guidelines

## Migration from Current Structure

The current codebase has most components in `src/components/`. The migration path:

1. **Phase 1**: Move page-specific components to appropriate page directories
2. **Phase 2**: Keep truly reusable components in `components/`  
3. **Phase 3**: Move settings components to `domain-components/settings/`
4. **Phase 4**: Create new pages and promote components as they become reusable

This structure provides clear boundaries while allowing natural evolution of component reusability.