# AppViewModel Usage Guide

The AppViewModel provides global timer state that persists across all pages and components. This is designed for system tray integration and global timer display.

## How to Access AppViewModel

From any component in your app:

```rust
use leptos::prelude::*;
use crate::app_vm::AppViewModel;

#[component]
pub fn MyComponent() -> impl IntoView {
    // Get the app_vm from context
    let app_vm = expect_context::<StoredValue<AppViewModel>>();

    // Extract signals you need
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

### Timer State Access
- `timer_state()` - Get the timer state signal
- `error_state()` - Get error state signal

### Timer Status Checks
- `is_running()` - Check if timer is actively counting
- `is_paused()` - Check if timer is paused
- `is_idle()` - Check if timer is idle/stopped

### Display Helpers
- `format_time()` - Returns "MM:SS" formatted time
- `phase_name()` - Returns current phase name ("Focus Time", "Short Break", etc.)
- `current_phase()` - Returns Option<Phase> enum
- `tray_display()` - Returns system tray formatted string (e.g., "Focus Time: 23:45")
- `tray_tooltip()` - Returns tooltip text for system tray

### Timer Operations
- `refresh_timer_state()` - Force refresh from backend
- `update_timer_state(state)` - Manually update timer state
- `clear_error()` - Clear any error state

## Example: Adding Timer to Sidebar

```rust
// In components/sidebar.rs
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

## Global Event Listeners

The AppViewModel automatically sets up listeners for these timer events:
- `timer:tick` - Updates remaining seconds every tick
- `timer:status_changed` - Updates timer status
- `timer:phase_completed` - Handles phase transitions
- `timer:phase_skipped` - Handles skip events

These listeners are initialized once when the app starts and persist for the entire app lifetime.

## Notes

1. **Single Source of Truth**: The AppViewModel maintains a single global timer state that updates automatically from backend events.

2. **Performance**: Use `with_value` to access methods to avoid unnecessary clones of the StoredValue.

3. **System Tray Ready**: The `tray_display()` and `tray_tooltip()` methods are specifically designed for system tray integration.

4. **No Task Management**: This AppViewModel focuses only on timer state. Task switching remains in the timer page's ViewModel for now.