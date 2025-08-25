use domain::{Result, TaskConfig, TimerConfiguration};

/// Convert TaskConfig to TimerConfiguration
pub fn task_config_to_timer_config(
    task_config: &TaskConfig,
) -> Result<TimerConfiguration> {
    TimerConfiguration::new(
        task_config.work_duration(),
        task_config.short_break_duration(),
        task_config.long_break_duration(),
        task_config.sessions_until_long_break(),
    )
}

/// Convert TimerConfiguration to TaskConfig with default screen blocking
pub fn timer_config_to_task_config(
    timer_config: &TimerConfiguration,
    enable_screen_blocking: bool,
) -> Result<TaskConfig> {
    TaskConfig::new(
        timer_config.work_duration,
        timer_config.short_break_duration,
        timer_config.long_break_duration,
        timer_config.sessions_until_long_break,
        enable_screen_blocking,
    )
}
