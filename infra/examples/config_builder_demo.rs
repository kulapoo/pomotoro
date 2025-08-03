use std::time::Duration;
use infra::adapters::ConfigBuilder;
use domain::{NotificationPosition, TaskCyclingBehavior, Theme};

fn main() {
    // Example of using the ConfigBuilder with flat setters
    let config = ConfigBuilder::new()
        .task_work_duration(Duration::from_secs(30 * 60)) // 30 minutes
        .task_short_break_duration(Duration::from_secs(10 * 60)) // 10 minutes
        .audio_work_notification_sound(Some("bell.mp3".to_string()))
        .audio_volume(0.8)
        .general_task_cycling_behavior(TaskCyclingBehavior::AutoAdvance)
        .general_auto_start_breaks(true)
        .notification_enable_desktop_notifications(true)
        .notification_position(NotificationPosition::TopRight)
        .appearance_theme(Theme::Dark)
        .appearance_compact_mode(true)
        .appearance_always_on_top(false)
        .build();

    println!("Built configuration:");
    println!("Work duration: {:?}", config.task_defaults.work_duration);
    println!("Audio volume: {}", config.audio.volume);
    println!("Theme: {:?}", config.appearance.theme);
    println!("Auto start breaks: {}", config.general.auto_start_breaks);

    // Example of using getters
    let builder = ConfigBuilder::from_config(config.clone());
    println!("\nUsing getters:");
    println!("Task work duration: {:?}", builder.get_task_work_duration());
    println!("Audio volume: {}", builder.get_audio_volume());
    println!("Appearance theme: {:?}", builder.get_appearance_theme());
    println!("General auto start breaks: {}", builder.get_general_auto_start_breaks());
}