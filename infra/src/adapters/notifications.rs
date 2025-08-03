use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use domain::Phase;

pub fn send_phase_notification(app_handle: &AppHandle, from_phase: &Phase, to_phase: &Phase) {
    let (title, body) = match (from_phase, to_phase) {
        (Phase::Work, Phase::ShortBreak) => (
            "Great work! 🎉",
            "Time for a 5-minute break. Rest your eyes and stretch!"
        ),
        (Phase::Work, Phase::LongBreak) => (
            "Excellent! 🏆",
            "You've completed 4 focus sessions! Take a 15-minute break."
        ),
        (Phase::ShortBreak, Phase::Work) | (Phase::LongBreak, Phase::Work) => (
            "Break's over! 💪",
            "Ready to focus? Let's get back to work!"
        ),
        _ => return,
    };

    let _ = app_handle
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}