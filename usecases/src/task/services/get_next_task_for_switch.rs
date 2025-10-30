use domain::{Error, Result, Task, TaskId, task::{TaskCycling, PureTaskCycling}};

/// Pure function to determine next task for switching
pub fn get_next_task_for_switch(
    tasks: &[Task],
    current_task_id: Option<&TaskId>,
    timer_is_running: bool,
) -> Result<Option<Task>> {
    // Check if timer is running
    if timer_is_running {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "NextTaskSwitch".to_string(),
        });
    }

    let cycling = PureTaskCycling;
    Ok(cycling.get_next_task(tasks, current_task_id))
}