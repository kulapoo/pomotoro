# UI Simulator Guide

## Overview

The UI Simulator is a comprehensive testing utility for simulating user interface interactions in integration tests. It provides a complete mock of the Tauri AppHandle and simulates all UI actions like clicking buttons, managing tasks, and handling events.

## Architecture

The UI Simulator consists of several key components:

### 1. MockAppHandle
- Mimics Tauri's AppHandle for IPC communication
- Tracks emitted events and handles event listeners
- Provides methods to verify event emissions

### 2. UI Action Modules
Organized into domain-specific modules:

- **timer_actions**: Timer controls (start, pause, reset, skip phase)
- **task_actions**: Task management (create, update, delete, complete)
- **config_actions**: Configuration management (themes, settings, audio)
- **audio_actions**: Audio controls (notifications, background music)

### 3. UiSimulator
The main simulator that combines all action modules and provides:
- Event bus integration
- Auto-responder functionality
- Complete workflow simulations

### 4. UiSimulatorHandle
Control handle for the running simulator:
- Trigger UI responses
- Check emitted events
- Simulate disconnections

## Usage Examples

### Basic Timer Operations

```rust
use std::sync::Arc;
use infra::tests::core::mocks::{MockEventBus, UiSimulator};

#[tokio::test]
async fn test_timer_operations() {
    let event_bus = Arc::new(MockEventBus::new());
    let simulator = UiSimulator::new(event_bus);
    
    // Start the timer
    let result = simulator.timer.click_start().await;
    assert_eq!(result["status"], "started");
    
    // Pause the timer
    let result = simulator.timer.click_pause().await;
    assert_eq!(result["status"], "paused");
    
    // Reset the timer
    let result = simulator.timer.click_reset().await;
    assert_eq!(result["status"], "idle");
}
```

### Task Management

```rust
#[tokio::test]
async fn test_task_management() {
    let event_bus = Arc::new(MockEventBus::new());
    let simulator = UiSimulator::new(event_bus);
    
    // Create a task
    let task = simulator.task.create_task(
        "Write tests",
        Some("Complete unit test coverage")
    ).await;
    
    let task_id = task["id"].as_str().unwrap();
    
    // Update the task
    let update = simulator.task.update_task(task_id, json!({
        "title": "Write integration tests"
    })).await;
    
    // Complete a session
    let session = simulator.task.complete_session(task_id).await;
    
    // Delete the task
    let deleted = simulator.task.delete_task(task_id).await;
}
```

### Auto-Responder Mode

```rust
#[tokio::test]
async fn test_auto_responder() {
    let event_bus = Arc::new(MockEventBus::new());
    let simulator = UiSimulator::new(event_bus);
    
    // Start auto-responder
    let handle = simulator.start_auto_responder();
    
    // Simulate timer ticks
    handle.acknowledge_ticks(5).await;
    
    // Respond to specific events
    handle.respond_to_event("timer:tick", json!({
        "remaining_seconds": 1200
    }));
    
    // Check if events were emitted
    assert!(handle.was_event_emitted("start_timer"));
}
```

### Complete Workflow Simulation

```rust
#[tokio::test]
async fn test_pomodoro_workflow() {
    let event_bus = Arc::new(MockEventBus::new());
    let simulator = UiSimulator::new(event_bus);
    
    // Run predefined workflow
    let responses = simulator.simulate_pomodoro_session().await;
    
    // Or create custom workflow
    let task = simulator.task.create_task("Focus work", None).await;
    let task_id = task["id"].as_str().unwrap();
    
    simulator.timer.switch_task(task_id).await;
    simulator.timer.click_start().await;
    
    // Simulate work session...
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    simulator.timer.click_pause().await;
    simulator.task.complete_session(task_id).await;
    simulator.timer.click_reset().await;
}
```

### Using the Builder Pattern

```rust
use infra::tests::core::mocks::UiSimulatorBuilder;

#[tokio::test]
async fn test_with_custom_config() {
    let event_bus = Arc::new(MockEventBus::new());
    
    let simulator = UiSimulatorBuilder::new()
        .with_auto_acknowledge_ticks(false)
        .with_response_delay(50)
        .with_initial_config(json!({
            "theme": "dark",
            "work_duration": 30
        }))
        .build(event_bus);
    
    // Use the configured simulator...
}
```

### Event Verification

```rust
#[tokio::test]
async fn test_event_verification() {
    let event_bus = Arc::new(MockEventBus::new());
    let simulator = UiSimulator::new(event_bus);
    
    // Perform actions
    simulator.timer.click_start().await;
    simulator.timer.click_pause().await;
    
    // Verify events were emitted
    let app_handle = simulator.app_handle();
    assert!(app_handle.was_event_emitted("start_timer"));
    assert!(app_handle.was_event_emitted("pause_timer"));
    
    // Get specific events
    let start_events = app_handle.events_of_type("start_timer");
    assert_eq!(start_events.len(), 1);
    
    // Clear events for next test phase
    app_handle.clear_events();
}
```

## Key Features

### 1. Modular Design
Each UI domain (timer, task, config, audio) has its own action module, making it easy to test specific functionality.

### 2. JSON Responses
All UI actions return JSON values using `serde_json`, matching real Tauri IPC responses.

### 3. Event Tracking
The MockAppHandle tracks all emitted events with timestamps, allowing verification of event sequences.

### 4. Auto-Responder
Automatically processes UI events in the background, simulating real UI behavior.

### 5. Workflow Simulations
Predefined workflows for common scenarios (Pomodoro sessions, task management).

## Testing Best Practices

1. **Start with Basic Actions**: Test individual UI actions before complex workflows
2. **Verify Events**: Always check that expected events were emitted
3. **Use Auto-Responder**: For integration tests that need UI acknowledgment
4. **Clean State**: Clear events between test phases if needed
5. **Custom Workflows**: Create reusable workflow functions for common scenarios

## API Reference

### UiSimulator
- `new(event_bus)`: Create new simulator
- `start_auto_responder()`: Start background event processing
- `simulate_pomodoro_session()`: Run predefined Pomodoro workflow
- `simulate_task_workflow()`: Run predefined task workflow
- `app_handle()`: Get the MockAppHandle

### MockAppHandle
- `emit(event, payload)`: Emit an event
- `listen(event, handler)`: Register event listener
- `emitted_events()`: Get all emitted events
- `events_of_type(type)`: Get events of specific type
- `was_event_emitted(type)`: Check if event was emitted
- `clear_events()`: Clear event history

### UiSimulatorHandle
- `trigger_response(response)`: Manually trigger UI response
- `acknowledge_ticks(count)`: Simulate timer tick acknowledgments
- `respond_to_event(type, payload)`: Respond to specific event
- `was_event_emitted(type)`: Check event emission
- `simulate_disconnect()`: Simulate UI disconnect

## Integration with Test Infrastructure

The UI Simulator integrates seamlessly with the existing test infrastructure:

```rust
use infra::tests::{
    AppContext,
    AppContextBuilder,
    MockEventBus,
    UiSimulator,
};

#[tokio::test]
async fn test_with_full_context() {
    // Setup app context
    let ctx = AppContextBuilder::new()
        .with_default_task()
        .with_default_config()
        .build()
        .await
        .unwrap();
    
    // Create UI simulator
    let event_bus = Arc::new(MockEventBus::new());
    let simulator = UiSimulator::new(event_bus);
    
    // Test with both backend and UI simulation...
}
```