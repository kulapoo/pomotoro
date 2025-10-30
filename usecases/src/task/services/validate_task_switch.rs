use domain::{Error, Result, Task, Timer};

/// Pure validation function for task switching
pub fn validate_task_switch(
    task: &Task,
    timer: &Timer,
) -> Result<()> {
    // Check if task is completed
    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // Check if timer is running
    if timer.is_running() {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "TaskSwitch".to_string(),
        });
    }

    Ok(())
}