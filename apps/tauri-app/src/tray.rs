//! System tray integration.
//!
//! Builds the tray icon + context menu, keeps the tooltip/icon in sync with
//! the live timer state, and honors the `minimize_to_tray` / `start_minimized`
//! / `show_countdown_in_tray` general config flags.
//!
//! The context menu mirrors the in-app actions from the React `TimerPage`:
//! play/pause, restart phase, skip phase, reset task, and complete task.

use std::sync::Arc;

use domain::{
    ConfigRepository, GeneralConfig, Phase, Task, TaskId, TaskRepository,
    TaskStatus, Timer, TimerStatus, event_names::ui_listeners,
};
use infra::adapters::{
    TimerRepositoryArc, TimerTickService,
    events::mem_event_bus::EventPublisherArc,
};
use serde_json;
use tauri::{
    AppHandle, Emitter, Listener, Manager, Wry,
    image::Image,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{
        MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    },
};
use usecases::task::reset_task as reset_task_uc;
use usecases::timer::{
    StartTimerPhaseCmd, pause_timer_phase, reset_timer_phase,
    resume_timer_phase, skip_timer_phase, start_timer_phase,
};

use crate::commands::task_cmd::complete_task_flow;

/// Stable id for our tray icon so it can be retrieved later.
pub const TRAY_ID: &str = "pomotoro-tray";

/// Cached decoded toro tray icon (the brand bull-head on an indigo tile).
static TORO_ICON: std::sync::OnceLock<Image<'static>> =
    std::sync::OnceLock::new();

/// The toro brand icon used as the tray's base image, decoded once from the
/// embedded PNG. The countdown overlay is drawn on top of this on Linux.
fn toro_base_icon() -> &'static Image<'static> {
    TORO_ICON.get_or_init(|| {
        match Image::from_bytes(include_bytes!("../icons/toro-128.png")) {
            Ok(img) => img,
            Err(e) => {
                log::error!("Failed to decode toro tray icon: {e}");
                Image::new_owned(vec![0; 4], 1, 1)
            }
        }
    })
}

/// Whether the current platform needs countdown text baked into icon pixels.
/// Linux tray backends ignore `set_title`, so the only reliable way to show
/// live text is to render it directly into the icon.
fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

/// 5x7 bitmap font for digits + colon, used to render the countdown directly
/// onto the tray icon on Linux where `set_title` is ignored.
fn glyph(c: char) -> Option<&'static [&'static str]> {
    match c {
        '0' => Some(&[
            "01110", "10001", "10001", "10001", "10001", "10001", "01110",
        ]),
        '1' => Some(&[
            "00100", "01100", "00100", "00100", "00100", "00100", "01110",
        ]),
        '2' => Some(&[
            "01110", "10001", "00001", "00010", "00100", "01000", "11111",
        ]),
        '3' => Some(&[
            "01110", "10001", "00001", "00110", "00001", "10001", "01110",
        ]),
        '4' => Some(&[
            "00010", "00110", "01010", "10010", "11111", "00010", "00010",
        ]),
        '5' => Some(&[
            "11111", "10000", "11110", "00001", "00001", "10001", "01110",
        ]),
        '6' => Some(&[
            "00110", "01000", "10000", "11110", "10001", "10001", "01110",
        ]),
        '7' => Some(&[
            "11111", "00001", "00010", "00100", "01000", "01000", "01000",
        ]),
        '8' => Some(&[
            "01110", "10001", "10001", "01110", "10001", "10001", "01110",
        ]),
        '9' => Some(&[
            "01110", "10001", "10001", "01111", "00001", "00010", "01100",
        ]),
        ':' => Some(&["0", "1", "0", "0", "0", "1", "0"]),
        _ => None,
    }
}

/// Render the tray icon showing just the countdown text in red on a
/// transparent background. Linux tray backends ignore `set_title`, so the
/// only reliable way to show live text is to bake it into the icon pixels.
///
/// "MM:SS" countdowns are **stacked vertically** (minutes above seconds) so
/// the digits can be much larger inside the square tray icon — a side-by-side
/// layout would force the block size down to ~1 px on typical panels, making
/// the text unreadable.
///
/// When `text` is `None` the base icon is returned unchanged.
fn overlay_countdown(base: &Image, text: Option<&str>) -> Image<'static> {
    let w = base.width() as usize;
    let h = base.height() as usize;

    if let Some(text) = text {
        // ── stacked "MM:SS" ────────────────────────────────────────────
        if text.len() == 5 && text.as_bytes()[2] == b':' {
            let top_glyphs: Vec<_> =
                text[0..2].chars().filter_map(glyph).collect();
            let bot_glyphs: Vec<_> =
                text[3..5].chars().filter_map(glyph).collect();

            if top_glyphs.len() == 2 && bot_glyphs.len() == 2 && h > 0 {
                // 2 digits per line (10 font-px) + 1 gap = 11 block units wide;
                // 2 rows of 7 + 1 gap = 15 block units tall.
                let block =
                    ((h as f32) * 0.90 / 15.0).round().max(1.0) as usize;
                let font_h = block * 7;
                let gap = block;
                let line_w = 2 * 5 * block + gap;

                let canvas = h;
                let mut rgba = vec![0u8; canvas * canvas * 4];

                // helpers
                let draw_line =
                    |rgba: &mut [u8],
                     x_off: usize,
                     y_off: usize,
                     glyphs: &[&[&str]]| {
                        let mut x = x_off;
                        for g in glyphs {
                            for (ry, row) in g.iter().enumerate() {
                                for (rx, ch) in row.chars().enumerate() {
                                    if ch == '1' {
                                        for dy in 0..block {
                                            for dx in 0..block {
                                                let px = x + rx * block + dx;
                                                let py =
                                                    y_off + ry * block + dy;
                                                if px < canvas && py < canvas {
                                                    let i =
                                                        (py * canvas + px) * 4;
                                                    rgba[i] = 255;
                                                    rgba[i + 1] = 0;
                                                    rgba[i + 2] = 0;
                                                    rgba[i + 3] = 255;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            x += 5 * block + gap;
                        }
                    };

                let total_h = font_h * 2 + gap;
                let x_off = (canvas.saturating_sub(line_w)) / 2;
                let y_off = (canvas.saturating_sub(total_h)) / 2;

                draw_line(&mut rgba, x_off, y_off, &top_glyphs);
                draw_line(&mut rgba, x_off, y_off + font_h + gap, &bot_glyphs);

                return Image::new_owned(rgba, canvas as u32, canvas as u32);
            }
            // fall through to single-line if glyph extraction failed
        }

        // ── single-line fallback (generic text, e.g. "25:00" as one line) ──
        let glyphs: Vec<&'static [&'static str]> =
            text.chars().filter_map(glyph).collect();
        if !glyphs.is_empty() && w > 0 && h > 0 {
            // Fit "MM:SS" horizontally inside a square canvas so the
            // tray doesn't squish it.
            let block = ((w as f32) * 0.88 / 25.0).round().max(1.0) as usize;
            let font_h = block * 7;
            let gap = block;
            let font_widths: Vec<usize> =
                glyphs.iter().map(|g| g[0].len() * block).collect();
            let total_w: usize =
                font_widths.iter().sum::<usize>() + gap * (glyphs.len() - 1);

            let canvas = w.max(h);
            let mut rgba = vec![0u8; canvas * canvas * 4];

            let mut x_off = (canvas.saturating_sub(total_w)) / 2;
            let y_off = (canvas.saturating_sub(font_h)) / 2;
            for (gi, g) in glyphs.iter().enumerate() {
                for (ry, row) in g.iter().enumerate() {
                    for (rx, ch) in row.chars().enumerate() {
                        if ch == '1' {
                            for dy in 0..block {
                                for dx in 0..block {
                                    let pxx = x_off + rx * block + dx;
                                    let pxy = y_off + ry * block + dy;
                                    if pxx < canvas && pxy < canvas {
                                        let i = (pxy * canvas + pxx) * 4;
                                        rgba[i] = 255; // R
                                        rgba[i + 1] = 0; // G
                                        rgba[i + 2] = 0; // B
                                        rgba[i + 3] = 255; // A
                                    }
                                }
                            }
                        }
                    }
                }
                x_off += font_widths[gi] + gap;
            }

            return Image::new_owned(rgba, canvas as u32, canvas as u32);
        }
    }

    // No countdown: return the base icon unchanged.
    let rgba = base.rgba().to_vec();
    Image::new_owned(rgba, w as u32, h as u32)
}

/// Menu item ids.
const MI_TOGGLE: &str = "toggle";
const MI_PLAY_PAUSE: &str = "play_pause";
const MI_RESET_PHASE: &str = "reset_phase";
const MI_SKIP: &str = "skip";
const MI_RESET_TASK: &str = "reset_task";
const MI_COMPLETE: &str = "complete";
const MI_QUIT: &str = "quit";

/// Held in Tauri state so refresh handlers can mutate menu item labels/state.
#[derive(Clone)]
pub struct TrayMenuHandles {
    pub toggle: Arc<MenuItem<Wry>>,
    pub play_pause: Arc<MenuItem<Wry>>,
    pub reset_phase: Arc<MenuItem<Wry>>,
    pub skip: Arc<MenuItem<Wry>>,
    pub reset_task: Arc<MenuItem<Wry>>,
    pub complete: Arc<MenuItem<Wry>>,
}

/// Build the tray icon and wire its menu / icon events. Stores menu handles in
/// Tauri state and registers event listeners that keep the tray in sync.
pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let (menu, handles) = build_menu(app)?;
    app.manage(handles);

    let initial_icon = overlay_countdown(toro_base_icon(), None);
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(initial_icon)
        .tooltip("Pomotoro")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_icon_event)
        .build(app)?;

    register_event_listeners(app);

    // Initial paint.
    let _ = refresh(app, None);

    Ok(())
}

fn build_menu(app: &AppHandle) -> tauri::Result<(Menu<Wry>, TrayMenuHandles)> {
    let toggle =
        MenuItem::with_id(app, MI_TOGGLE, "Show Pomotoro", true, None::<&str>)?;
    let play_pause =
        MenuItem::with_id(app, MI_PLAY_PAUSE, "Start", true, None::<&str>)?;
    let reset_phase = MenuItem::with_id(
        app,
        MI_RESET_PHASE,
        "Restart Phase",
        true,
        None::<&str>,
    )?;
    let skip =
        MenuItem::with_id(app, MI_SKIP, "Skip Phase", true, None::<&str>)?;
    let reset_task = MenuItem::with_id(
        app,
        MI_RESET_TASK,
        "Reset Task",
        true,
        None::<&str>,
    )?;
    let complete = MenuItem::with_id(
        app,
        MI_COMPLETE,
        "Complete Task",
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, MI_QUIT, "Quit", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &toggle,
            &sep1,
            &play_pause,
            &reset_phase,
            &skip,
            &sep2,
            &reset_task,
            &complete,
            &sep3,
            &quit,
        ],
    )?;

    let handles = TrayMenuHandles {
        toggle: Arc::new(toggle),
        play_pause: Arc::new(play_pause),
        reset_phase: Arc::new(reset_phase),
        skip: Arc::new(skip),
        reset_task: Arc::new(reset_task),
        complete: Arc::new(complete),
    };

    Ok((menu, handles))
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        MI_TOGGLE => toggle_window(app),
        MI_PLAY_PAUSE => menu_play_pause(app),
        MI_RESET_PHASE => menu_reset_phase(app),
        MI_SKIP => menu_skip(app),
        MI_RESET_TASK => menu_reset_task(app),
        MI_COMPLETE => menu_complete(app),
        MI_QUIT => quit_app(app),
        _ => {}
    }
}

// ── managed-state accessor helpers ──────────────────────────────────────────

fn task_repo(app: &AppHandle) -> Option<Arc<dyn TaskRepository + Send + Sync>> {
    app.try_state::<Arc<dyn TaskRepository + Send + Sync>>()
        .map(|s| s.inner().clone())
}

fn timer_repo(app: &AppHandle) -> Option<TimerRepositoryArc> {
    app.try_state::<TimerRepositoryArc>()
        .map(|s| s.inner().clone())
}

fn config_repo(
    app: &AppHandle,
) -> Option<Arc<dyn ConfigRepository + Send + Sync>> {
    app.try_state::<Arc<dyn ConfigRepository + Send + Sync>>()
        .map(|s| s.inner().clone())
}

fn event_publisher(app: &AppHandle) -> Option<EventPublisherArc> {
    app.try_state::<EventPublisherArc>()
        .map(|s| s.inner().clone())
}

fn tick_service(app: &AppHandle) -> Option<Arc<TimerTickService>> {
    app.try_state::<Arc<TimerTickService>>()
        .map(|s| s.inner().clone())
}

// ── menu action handlers ────────────────────────────────────────────────────

/// Play / Pause / Resume — a single toggle mirroring the React play-pause
/// button. Running → pause, Paused → resume, otherwise → start.
fn menu_play_pause(app: &AppHandle) {
    let Some((timer, _, _)) = load_state(app) else {
        return;
    };
    let status = timer.status();
    let Some(task_id) = timer.task_id() else {
        return;
    };

    let Some(task_repo) = task_repo(app) else {
        return;
    };
    let Some(timer_repo) = timer_repo(app) else {
        return;
    };
    let Some(event_publisher) = event_publisher(app) else {
        return;
    };

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let res = match status {
            TimerStatus::Running => {
                let remaining = timer.remaining_seconds(None);
                pause_timer_phase(
                    task_id,
                    remaining,
                    task_repo,
                    timer_repo,
                    event_publisher,
                )
                .await
                .map(|_| ())
            }
            TimerStatus::Paused => resume_timer_phase(
                task_id,
                task_repo,
                timer_repo,
                event_publisher,
            )
            .await
            .map(|_| ()),
            TimerStatus::Idle | TimerStatus::Stopped => {
                start_timer_phase(
                    task_repo,
                    timer_repo,
                    event_publisher,
                    StartTimerPhaseCmd {
                        task_id: Some(task_id),
                    },
                )
                .await
            }
        };
        if let Err(e) = res {
            log::error!("Tray play/pause failed: {}", e);
        }
        let _ = refresh(&app, None);
    });
}

/// Restart the current phase's countdown, mirroring the React "Restart Phase"
/// button (`reset_timer_phase`). After the reset, a running phase must have its
/// tick loop restarted (the Reset event handler stops it); a paused phase is
/// left paused.
fn menu_reset_phase(app: &AppHandle) {
    let Some(task_id) = task_id_for_action(app) else {
        return;
    };
    let Some(task_repo) = task_repo(app) else {
        return;
    };
    let Some(timer_repo) = timer_repo(app) else {
        return;
    };
    let Some(event_publisher) = event_publisher(app) else {
        return;
    };
    let Some(tick_service) = tick_service(app) else {
        return;
    };

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        // Task's timer configuration is needed to restart the tick loop.
        let task = match task_repo.get_by_id(task_id).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                log::error!("Tray reset phase: task {} not found", task_id);
                let _ = refresh(&app, None);
                return;
            }
            Err(e) => {
                log::error!("Tray reset phase: failed to load task: {}", e);
                let _ = refresh(&app, None);
                return;
            }
        };

        if let Err(e) = reset_timer_phase(
            task_id,
            task_repo.clone(),
            timer_repo.clone(),
            event_publisher.clone(),
        )
        .await
        {
            log::error!("Tray reset phase failed: {}", e);
        }

        // Drain the async Reset event handler (stop_timer_tick_loop + load_state).
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Restart the tick loop so a running phase keeps counting down from the
        // full duration. A paused timer's loop would no-op, so skip it.
        if let Ok(updated) = timer_repo.get().await {
            if updated.is_running() {
                if let Err(e) = tick_service
                    .start_timer_tick_loop(
                        Some(task.config().timer.clone()),
                        None,
                    )
                    .await
                {
                    log::error!(
                        "Tray reset phase: failed to restart tick loop: {}",
                        e
                    );
                }
            }
        }
        let _ = refresh(&app, None);
    });
}

/// Skip to the next phase, mirroring the React "Skip Phase" button.
fn menu_skip(app: &AppHandle) {
    let Some(task_id) = task_id_for_action(app) else {
        return;
    };
    let Some(task_repo) = task_repo(app) else {
        return;
    };
    let Some(timer_repo) = timer_repo(app) else {
        return;
    };
    let Some(event_publisher) = event_publisher(app) else {
        return;
    };

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) =
            skip_timer_phase(task_repo, timer_repo, event_publisher, task_id)
                .await
        {
            log::error!("Tray skip phase failed: {}", e);
        }
        let _ = refresh(&app, None);
    });
}

/// Reset the active task's progress (completed sessions), mirroring the React
/// "Reset Task" button. Also resets the timer to idle; the Reset event handler
/// stops the tick loop.
fn menu_reset_task(app: &AppHandle) {
    let Some(task_id) = task_id_for_action(app) else {
        return;
    };
    let Some(task_repo) = task_repo(app) else {
        return;
    };
    let Some(timer_repo) = timer_repo(app) else {
        return;
    };
    let Some(event_publisher) = event_publisher(app) else {
        return;
    };

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) =
            reset_task_uc(task_repo, timer_repo, event_publisher, task_id).await
        {
            log::error!("Tray reset task failed: {}", e);
        }
        let _ = refresh(&app, None);
    });
}

/// Complete the active task (force-complete all sessions), mirroring the React
/// "Complete Task" button. Delegates to the shared `complete_task_flow` used by
/// the Tauri command so behavior is identical (stop + reset timer, optional
/// auto-advance).
fn menu_complete(app: &AppHandle) {
    let Some(task_id) = task_id_for_action(app) else {
        return;
    };
    let Some(task_repo) = task_repo(app) else {
        return;
    };
    let Some(timer_repo) = timer_repo(app) else {
        return;
    };
    let Some(config_repo) = config_repo(app) else {
        return;
    };
    let Some(event_publisher) = event_publisher(app) else {
        return;
    };
    let Some(tick_service) = tick_service(app) else {
        return;
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = complete_task_flow(
            task_id,
            task_repo,
            timer_repo,
            config_repo,
            event_publisher,
            tick_service,
            app_handle.clone(),
        )
        .await
        {
            log::error!("Tray complete task failed: {}", e);
        }
        let _ = refresh(&app_handle, None);
    });
}

/// Extract the bound task_id for menu actions.
fn task_id_for_action(app: &AppHandle) -> Option<TaskId> {
    load_state(app).and_then(|(timer, _, _)| timer.task_id())
}

fn on_tray_icon_event(tray: &TrayIcon<Wry>, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        toggle_window(tray.app_handle());
    }
}

/// Show/hide the main window, then refresh tray UI so the toggle label
/// reflects the new visibility.
pub fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
    let _ = refresh(app, None);
}

fn quit_app(app: &AppHandle) {
    let _ = app.emit("app:exited", ());
    app.exit(0);
}

/// Subscribe to timer + task + config events so the tray stays in sync.
fn register_event_listeners(app: &AppHandle) {
    let app_tick = app.clone();
    app.listen(ui_listeners::timer::TICK, move |event| {
        let remaining =
            serde_json::from_str::<serde_json::Value>(event.payload())
                .ok()
                .and_then(|v| {
                    v.get("remaining_seconds")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32)
                });
        let _ = refresh(&app_tick, remaining);
    });

    for evt in [
        ui_listeners::timer::STATUS_CHANGED,
        ui_listeners::timer::PHASE_EVENT,
        ui_listeners::timer::PHASE_COMPLETED,
        ui_listeners::timer::PHASE_SKIPPED,
        ui_listeners::timer::PAUSE,
        ui_listeners::timer::RESUME,
        ui_listeners::timer::START,
        ui_listeners::timer::RESET,
        ui_listeners::task::TASK_COMPLETED,
        ui_listeners::task::LIST_UPDATED,
        ui_listeners::task::ACTIVE_CHANGED,
        ui_listeners::task::AUTO_ADVANCED,
    ] {
        let app_status = app.clone();
        app.listen(evt, move |_| {
            let _ = refresh(&app_status, None);
        });
    }

    let app_cfg = app.clone();
    app.listen(ui_listeners::config::CONFIG_UPDATED, move |_| {
        let _ = refresh(&app_cfg, None);
    });
}

/// Re-read timer + general config + active task, then update the tray
/// icon/tooltip/title and the menu item labels/enabled-state.
///
/// When `remaining_secs_override` is `Some`, it is used for the countdown
/// display instead of reading `remaining_seconds` from the persisted timer
/// (which is only saved periodically and may be stale between ticks).
pub fn refresh(
    app: &AppHandle,
    remaining_secs_override: Option<u32>,
) -> tauri::Result<()> {
    let Some((timer, general, task)) = load_state(app) else {
        return Ok(());
    };

    let status = timer.status();
    let phase = timer.get_current_phase();
    let running = matches!(status, TimerStatus::Running);
    let paused = matches!(status, TimerStatus::Paused);
    let idle = matches!(status, TimerStatus::Idle | TimerStatus::Stopped);
    // A "session" exists whenever a task is attached — even if the timer is
    // currently stopped. This is what gates the action availability.
    let has_task = timer.task_id().is_some();

    // Mirror the React `useTimerSession` gating flags.
    let is_task_completed = task.as_ref().is_some_and(|t| {
        matches!(t.status(), TaskStatus::Completed)
            && t.completed_at().is_some()
    });
    let is_break = matches!(phase, Phase::ShortBreak | Phase::LongBreak);
    let is_last_break = task.as_ref().is_some_and(|t| {
        t.completed_at().is_none()
            && matches!(t.status(), TaskStatus::Completed)
            && is_break
    });

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_tooltip(Some(tooltip_text(status, phase, has_task)))?;

        if is_linux() {
            // Linux tray backends ignore `set_title`, so we bake the
            // countdown text directly into the icon pixels. When a task is
            // attached we always show the countdown — even for Idle/Stopped
            // — so the user sees the full phase duration at a glance.
            let countdown = if general.show_countdown_in_tray && has_task {
                let secs = remaining_secs_override
                    .unwrap_or_else(|| timer.remaining_seconds(None));
                Some(format!("{:02}:{:02}", secs / 60, secs % 60))
            } else {
                None
            };
            let icon =
                overlay_countdown(toro_base_icon(), countdown.as_deref());
            tray.set_icon(Some(icon))?;
            let _ = tray.set_title::<&str>(None);
        } else {
            // macOS / Windows: `set_title` renders native crisp text
            // next to the tray icon. Only show the countdown while the
            // timer is actively counting (Running or Paused).
            let title = if general.show_countdown_in_tray {
                match status {
                    TimerStatus::Running | TimerStatus::Paused => {
                        let secs = remaining_secs_override
                            .unwrap_or_else(|| timer.remaining_seconds(None));
                        Some(format!("{:02}:{:02}", secs / 60, secs % 60))
                    }
                    TimerStatus::Idle | TimerStatus::Stopped => None,
                }
            } else {
                None
            };
            tray.set_title(title.as_deref())?;
            tray.set_icon(Some(toro_base_icon().clone()))?;
        }
    }

    if let Some(h) = app.try_state::<TrayMenuHandles>() {
        // Reflect window visibility in the toggle label.
        let visible = app
            .get_webview_window("main")
            .map(|w| w.is_visible().unwrap_or(false))
            .unwrap_or(true);
        let _ = h.toggle.set_text(if visible {
            "Hide Pomotoro"
        } else {
            "Show Pomotoro"
        });
        let _ = h.toggle.set_enabled(true);

        // Play/Pause toggles between Start / Pause / Resume.
        let _ = h.play_pause.set_enabled(has_task && !is_task_completed);
        let _ = h.play_pause.set_text(if running {
            "Pause"
        } else if paused {
            "Resume"
        } else {
            "Start"
        });

        let _ = h
            .reset_phase
            .set_enabled(has_task && !idle && !is_task_completed);
        let _ = h.skip.set_enabled(
            has_task && !idle && !is_last_break && !is_task_completed,
        );
        let _ = h.reset_task.set_enabled(has_task);
        let _ = h.complete.set_enabled(has_task && !is_task_completed);
    }

    Ok(())
}

/// Hover tooltip — short status line (the live countdown lives in the icon).
fn tooltip_text(status: TimerStatus, phase: Phase, has_task: bool) -> String {
    if !has_task {
        return "Pomotoro — No active task".to_string();
    }
    let status_word = match status {
        TimerStatus::Running => "Running",
        TimerStatus::Paused => "Paused",
        TimerStatus::Idle | TimerStatus::Stopped => "Stopped",
    };
    match status {
        TimerStatus::Running | TimerStatus::Paused => {
            format!("Pomotoro — {} ({})", phase.name(), status_word)
        }
        TimerStatus::Idle | TimerStatus::Stopped => {
            format!("Pomotoro — {}", status_word)
        }
    }
}

/// Run a future to completion in a way that is safe from *any* thread.
///
/// `tauri::async_runtime::block_on` panics with "Cannot start a runtime
/// from within a runtime" when the caller is already on a tokio worker
/// thread — which is exactly where our `app.listen` callbacks fire (timer
/// ticks and similar events are emitted from async background tasks). When
/// we detect an active runtime we hop onto a bare OS thread so `block_on`
/// is driven from outside the worker pool; otherwise we call it directly
/// (e.g. from the main thread handling a menu/tray click).
fn block_on_safe<F>(fut: F) -> F::Output
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        std::thread::scope(|s| {
            s.spawn(|| tauri::async_runtime::block_on(fut))
                .join()
                .expect("tray state load task panicked")
        })
    } else {
        tauri::async_runtime::block_on(fut)
    }
}

/// Load the current timer, general config, and active task from managed Tauri
/// state. The task is `None` when no task is bound to the timer.
fn load_state(app: &AppHandle) -> Option<(Timer, GeneralConfig, Option<Task>)> {
    let timer_repo = app.try_state::<TimerRepositoryArc>()?.inner().clone();
    let config_repo = app
        .try_state::<Arc<dyn ConfigRepository + Send + Sync>>()?
        .inner()
        .clone();
    let task_repo = app
        .try_state::<Arc<dyn TaskRepository + Send + Sync>>()?
        .inner()
        .clone();

    block_on_safe(async move {
        let timer = timer_repo.get().await.ok()?;
        let general = config_repo.get_config().await.ok()?.general;
        let task = match timer.task_id() {
            Some(tid) => task_repo.get_by_id(tid).await.ok().flatten(),
            None => None,
        };
        Some((timer, general, task))
    })
}

/// Read the current `GeneralConfig` (best-effort; falls back to defaults if
/// the repositories aren't ready yet).
pub fn current_general(app: &AppHandle) -> GeneralConfig {
    match load_state(app) {
        Some((_, g, _)) => g,
        None => GeneralConfig::default(),
    }
}
