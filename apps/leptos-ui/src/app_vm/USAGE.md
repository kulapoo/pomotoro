# AppViewModel Usage Guide

The AppViewModel provides global timer state and active task that persist across all pages and components. This is designed for system tray integration, global timer display, and active task management.

## How to Access AppViewModel

The AppViewModel is provided at the app level and can be accessed from any component using Leptos context:

```rust
use leptos::prelude::*;
use crate::app_vm::AppViewModel;

#[component]
pub fn MyComponent() -> impl IntoView {
    // Get the app_vm from context
    let app_vm = expect_context::<StoredValue<AppViewModel>>();

    // Access methods using with_value
    let timer_state = app_vm.with_value(|v| v.timer_state());

    view! {
        <div>
            // Display current timer time
            <p>{move || app_vm.with_value(|v| v.format_time())}</p>

            // Display phase name
            <p>{move || app_vm.with_value(|v| v.phase_name())}</p>

            // Display tray-formatted string
            <p>{move || app_vm.with_value(|v| v.tray_display())}</p>

            // Check timer status
            <p>"Running: "{move || app_vm.with_value(|v| v.is_running())}</p>
            <p>"Paused: "{move || app_vm.with_value(|v| v.is_paused())}</p>
        </div>
    }
}
```

## Available Methods

### State Access (Signal Getters)
- `timer_state()` - Get the `ReadSignal<TimerState>`
- `set_timer_state()` - Get the `WriteSignal<TimerState>` (private, used internally)
- `active_task()` - Get the `ReadSignal<Option<Task>>`
- `set_active_task()` - Get the `WriteSignal<Option<Task>>`
- `error_state()` - Get the `ReadSignal<Option<ErrorInfo>>`

### Timer Status Checks
- `is_running()` - Check if timer is actively counting (returns `bool`)
- `is_paused()` - Check if timer is paused (returns `bool`)
- `is_idle()` - Check if timer is idle/stopped (returns `bool`)

### Timer Display Helpers
- `format_time()` - Returns "MM:SS" formatted time string
- `phase_name()` - Returns current phase name ("Work", "Short Break", "Long Break", or "Idle")
- `current_phase()` - Returns `Option<Phase>` enum (None if idle)

### System Tray Helpers
- `tray_display()` - Returns formatted string for system tray (e.g., "Work: 23:45" or "Work: 23:45 (Paused)")
- `tray_tooltip()` - Returns tooltip text for system tray hover

### Active Task Helpers
- `get_active_task()` - Get the current active task as `Option<Task>`
- `get_active_task_name()` - Get the active task name or "No active task"
- `get_active_entity_id()` - Get the active task ID as `Option<String>`
- `is_active_task_completed()` - Check if the active task is completed

### Timer Operations
- `refresh_timer_state()` - Force refresh timer state from backend
- `update_timer_state(state)` - Manually update timer state (use with caution)
- `clear_error()` - Clear any error state

## Architecture

### Module Structure

The AppViewModel implementation is split across multiple files for better organization:

```
app_vm/
├── mod.rs              # Main struct definition and ViewModel trait impl
├── accessors.rs        # Signal accessors and display helpers
├── initialization.rs   # Setup and event listeners
└── timer_events.rs     # Timer state operations
```

This structure uses Rust's module system where:
- `mod.rs` declares the modules with `mod accessors;`, `mod initialization;`, etc.
- Each file contains `impl AppViewModel` blocks that extend the same type
- All methods are available as if defined in a single file

### Initialization

The AppViewModel is created and provided at the app level in `app.rs`:

```rust
pub fn App() -> impl IntoView {
    // Create app-level ViewModel
    let app_vm = StoredValue::new(AppViewModel::new());

    // Provide to children via context
    provide_context(app_vm);

    view! {
        <Router>
            <AppLayout />
        </Router>
    }
}
```

When `AppViewModel::new()` is called, it:
1. Creates signals for timer state, active task, and error state
2. Calls `initialize()` which:
   - Loads initial timer state from backend
   - Loads initial active task from backend
   - Sets up all event listeners

## Global Event Listeners

The AppViewModel automatically sets up listeners for backend events during initialization:

### Timer Events
- `timer:start` - Updates timer state when timer starts
- `timer:tick` - Updates remaining seconds every tick
- `timer:status_changed` - Updates timer status (running/paused/idle)
- `timer:phase_completed` - Handles phase transitions (work → break, etc.)
- `timer:phase_skipped` - Handles skip events

### Task Events
- `task:active_changed` - Updates the active task when it changes
- `task:task_completed` - Refreshes task data when completed
- `task:progress_updated` - Updates task progress in real-time

These listeners persist for the entire app lifetime and automatically keep the AppViewModel in sync with the backend.

## Example: Adding Timer to Sidebar

```rust
use leptos::prelude::*;
use crate::app_vm::AppViewModel;

#[component]
pub fn Sidebar() -> impl IntoView {
    let app_vm = expect_context::<StoredValue<AppViewModel>>();

    view! {
        <div class="sidebar">
            // ... existing sidebar content ...

            // Add timer display if running
            {move || {
                let is_running = app_vm.with_value(|v| v.is_running() || v.is_paused());
                if is_running {
                    view! {
                        <div class="timer-widget">
                            <span class="timer-phase">
                                {move || app_vm.with_value(|v| v.phase_name())}
                            </span>
                            <span class="timer-time">
                                {move || app_vm.with_value(|v| v.format_time())}
                            </span>
                            {move || {
                                if app_vm.with_value(|v| v.is_paused()) {
                                    view! { <span class="paused">"(Paused)"</span> }
                                } else {
                                    view! { <span></span> }
                                }
                            }}
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }
            }}
        </div>
    }
}
```

## Example: Using AppViewModel in Another ViewModel

The TimerViewModel demonstrates how to use AppViewModel within another ViewModel:

```rust
use leptos::prelude::*;
use crate::app_vm::AppViewModel;

impl ViewModel for TimerViewModel {
    fn new() -> Self {
        // Get the AppViewModel from context
        let app_vm = expect_context::<StoredValue<AppViewModel>>();

        // Extract signals you need
        let timer_state = app_vm.with_value(|v| v.timer_state());
        let set_timer_state = app_vm.with_value(|v| v.set_timer_state);
        let active_task = app_vm.with_value(|v| v.active_task());
        let set_active_task = app_vm.with_value(|v| v.set_active_task());

        // Use these signals in your ViewModel
        Self {
            timer_state,
            set_timer_state,
            active_task,
            set_active_task,
            // ... other fields
        }
    }
}
```

## Best Practices

1. **Single Source of Truth**: The AppViewModel maintains global timer and task state. Don't duplicate this state elsewhere.

2. **Use `with_value`**: Always access methods through `with_value` to avoid unnecessary clones:
   ```rust
   // Good
   app_vm.with_value(|v| v.format_time())
   
   // Avoid (creates unnecessary clone)
   let vm = app_vm.get_value();
   vm.format_time()
   ```

3. **Reactive Updates**: The AppViewModel's signals automatically trigger re-renders. Use them in reactive contexts:
   ```rust
   // This will automatically update when timer state changes
   {move || app_vm.with_value(|v| v.format_time())}
   ```

4. **Error Handling**: The AppViewModel tracks errors via `error_state()`. Check this signal to display global errors.

5. **System Tray Integration**: Use `tray_display()` and `tray_tooltip()` for consistent system tray formatting.

6. **Signal Extraction**: When using AppViewModel in other ViewModels, extract the raw signals once in `new()` rather than repeatedly calling `expect_context`.

## Notes

- The AppViewModel implements the `ViewModel` trait with `TimerState` as its state type
- Event listeners are set up once during initialization and use `callback.forget()` to persist for the app lifetime
- The AppViewModel automatically fetches task details when the active task changes via backend events
- All timer operations should go through backend commands; the AppViewModel listens for the results