//! System tray integration.
//!
//! Builds the tray icon + context menu, keeps the tooltip/icon in sync with
//! the live timer state, and honors the `minimize_to_tray` / `start_minimized`
//! / `show_countdown_in_tray` general config flags.
//!
//! The context menu mirrors the in-app actions from the React `TimerPage`:
//! play/pause, restart phase, skip phase, reset task, and complete task.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

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

/// Tracks the *intended* visibility of the main window. `window.is_visible()`
/// does not update synchronously right after `hide()`/`show()` on Linux/GTK,
/// so reading it immediately after toggling yields stale data and the Show/Hide
/// menu label lags by one click. This atomic is the authoritative source for
/// `refresh()` and is updated wherever the window is shown/hidden.
static INTENDED_VISIBLE: AtomicBool = AtomicBool::new(true);

/// Read the intended main-window visibility.
pub fn intended_visible() -> bool {
    INTENDED_VISIBLE.load(Ordering::Relaxed)
}

/// Update the intended main-window visibility. Call this whenever the main
/// window is shown or hidden from anywhere (tray toggle, close-to-tray,
/// start-minimized, etc.).
pub fn set_intended_visible(v: bool) {
    INTENDED_VISIBLE.store(v, Ordering::Relaxed);
}

// ── refresh coalescing ───────────────────────────────────────────────────────
//
// Tray event listeners fire synchronously on the thread that *emits* the event
// — for the per-second timer TICK and most status events that is a tokio worker
// thread. Doing the repo reads + icon re-render inline there (and worse,
// `block_on_safe`-ing them, which spawns a fresh OS thread per call because we
// are already inside a runtime) starves the very runtime that produces the data
// and visibly "chokes" the client.
//
// Instead every listener/menu handler just pushes a cheap signal onto an
// unbounded channel; a single background task drains it, coalesces bursts, and
// performs one async `refresh`. That keeps the work off the emitter thread and
// collapses the half-dozen events a single user action can fan out into one
// re-render.

/// A request to re-sync the tray UI.
enum RefreshSignal {
    /// Timer tick carrying the live remaining-seconds override (avoids reading
    /// the periodically-persisted timer, which is stale between ticks).
    Tick(u32),
    /// Any other change (status / phase / task / config) — re-read everything.
    Dirty,
}

static REFRESH_TX: std::sync::OnceLock<
    tokio::sync::mpsc::UnboundedSender<RefreshSignal>,
> = std::sync::OnceLock::new();

/// Enqueue a refresh signal. Non-blocking; safe to call from any thread
/// (listener callbacks, menu handlers, window events).
fn schedule_refresh(signal: RefreshSignal) {
    if let Some(tx) = REFRESH_TX.get() {
        let _ = tx.send(signal);
    }
}

/// Last countdown label painted into the tray. Used to skip the (relatively
/// expensive) icon pixel rebuild + `set_icon`/`set_title` calls when the
/// displayed text has not actually changed.
static LAST_LABEL: Mutex<Option<String>> = Mutex::new(None);

/// Background task owning all tray re-renders. Drains the signal channel,
/// coalesces a burst into a single refresh, and forwards the most recent tick
/// override (if any) so the countdown stays live.
async fn refresh_loop(
    app: AppHandle,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<RefreshSignal>,
) {
    while let Some(first) = rx.recv().await {
        // Coalesce everything already queued by the time we get here. Keep the
        // latest tick override so the countdown reflects the freshest second.
        let mut override_secs = match first {
            RefreshSignal::Tick(s) => Some(s),
            RefreshSignal::Dirty => None,
        };
        while let Ok(next) = rx.try_recv() {
            if let RefreshSignal::Tick(s) = next {
                override_secs = Some(s);
            }
        }
        let _ = refresh_async(&app, override_secs).await;
    }
}

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

    // Spawn the single coalescing refresh loop before registering listeners so
    // the very first signal has somewhere to go.
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<RefreshSignal>();
    let _ = REFRESH_TX.set(tx);
    let app_loop = app.clone();
    tauri::async_runtime::spawn(async move {
        refresh_loop(app_loop, rx).await;
    });

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

    // Initial paint: try the synchronous path first (blocking, most likely to
    // work immediately), then schedule an async retry so a transient failure
    // (e.g. D-Bus not ready on Linux) doesn't leave the tray stuck on the
    // base icon.  The LAST_LABEL cache is only committed *after* a successful
    // icon update, so a failed sync attempt won't poison subsequent refreshes.
    let _ = refresh(app, None);
    schedule_refresh(RefreshSignal::Dirty);

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

// ── menu action context ─────────────────────────────────────────────────────
//
// Every tray menu handler follows the identical skeleton: extract repos from
// Tauri state, load the bound task_id, spawn async work, and schedule a
// refresh on completion. `TrayCtx` bundles all of that extraction + spawning
// into a single call so each handler becomes a 3–8 line function.

/// All managed state needed by tray menu actions, extracted once per action.
struct TrayCtx {
    app: AppHandle,
    task_id: TaskId,
    timer: Timer,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: TimerRepositoryArc,
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: EventPublisherArc,
    tick_service: Arc<TimerTickService>,
}

impl TrayCtx {
    fn try_load(app: &AppHandle) -> Option<Self> {
        let (timer, _, _) = load_state(app)?;
        let task_id = timer.task_id()?;
        Some(Self {
            app: app.clone(),
            task_id,
            timer,
            task_repo: app
                .try_state::<Arc<dyn TaskRepository + Send + Sync>>()?
                .inner()
                .clone(),
            timer_repo: app.try_state::<TimerRepositoryArc>()?.inner().clone(),
            config_repo: app
                .try_state::<Arc<dyn ConfigRepository + Send + Sync>>()?
                .inner()
                .clone(),
            event_publisher: app
                .try_state::<EventPublisherArc>()?
                .inner()
                .clone(),
            tick_service: app
                .try_state::<Arc<TimerTickService>>()?
                .inner()
                .clone(),
        })
    }

    /// Spawn async work on the Tauri runtime; schedules a tray refresh
    /// (Dirty) once the future completes, regardless of success or error.
    fn spawn<F, Fut>(self, f: F)
    where
        F: FnOnce(Self) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tauri::async_runtime::spawn(async move {
            f(self).await;
            schedule_refresh(RefreshSignal::Dirty);
        });
    }
}

// ── menu action handlers ────────────────────────────────────────────────────

async fn start_loop_for_task(ctx: &TrayCtx) -> domain::Result<()> {
    let task =
        ctx.task_repo.get_by_id(ctx.task_id).await?.ok_or_else(|| {
            domain::Error::TaskNotFound {
                id: ctx.task_id.to_string(),
            }
        })?;
    ctx.tick_service
        .start_timer_tick_loop(Some(task.config().timer.clone()))
        .await
        .map_err(|e| domain::Error::RepositoryError { message: e })
}

async fn pause_running_timer(ctx: &TrayCtx) -> domain::Result<()> {
    let live = ctx.tick_service.get_current_timer().await;
    let remaining = live.remaining_seconds(None);
    pause_timer_phase(
        ctx.task_id,
        remaining,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_publisher.clone(),
    )
    .await?;
    // Drive the stop directly. The TimerPaused event handler is a UI-only
    // emitter and no longer stops the loop.
    ctx.tick_service.load_state().await?;
    ctx.tick_service.stop_timer_tick_loop().await?;
    Ok(())
}

async fn resume_paused_timer(ctx: &TrayCtx) -> domain::Result<()> {
    resume_timer_phase(
        ctx.task_id,
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_publisher.clone(),
    )
    .await?;
    start_loop_for_task(ctx).await
}

async fn start_idle_timer(ctx: &TrayCtx) -> domain::Result<()> {
    start_timer_phase(
        ctx.task_repo.clone(),
        ctx.timer_repo.clone(),
        ctx.event_publisher.clone(),
        StartTimerPhaseCmd {
            task_id: Some(ctx.task_id),
        },
    )
    .await?;
    start_loop_for_task(ctx).await
}

/// Play / Pause / Resume — a single toggle mirroring the React play-pause
/// button. Running → pause, Paused → resume, otherwise → start. Per the
/// tick-loop ownership contract, this handler drives the loop directly in
/// each branch.
fn menu_play_pause(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let _orchestration_lock = ctx.tick_service.orchestration_lock().await;
        let res: domain::Result<()> = match ctx.timer.status() {
            TimerStatus::Running => pause_running_timer(&ctx).await,
            TimerStatus::Paused => resume_paused_timer(&ctx).await,
            TimerStatus::Idle | TimerStatus::Stopped => {
                start_idle_timer(&ctx).await
            }
        };
        if let Err(e) = res {
            log::error!("Tray play/pause failed: {}", e);
        }
    });
}

/// Restart the current phase's countdown, mirroring the React "Restart Phase"
/// button (`reset_timer_phase`). Per the tick-loop ownership contract, this
/// handler drives stop/load/start directly — no sleep, no reliance on the
/// Reset event handler.
fn menu_reset_phase(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let _orchestration_lock = ctx.tick_service.orchestration_lock().await;
        let task = match ctx.task_repo.get_by_id(ctx.task_id).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                log::error!("Tray reset phase: task {} not found", ctx.task_id);
                return;
            }
            Err(e) => {
                log::error!("Tray reset phase: failed to load task: {}", e);
                return;
            }
        };

        if let Err(e) = reset_timer_phase(
            ctx.task_id,
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_publisher.clone(),
        )
        .await
        {
            log::error!("Tray reset phase failed: {}", e);
            return;
        }

        if let Err(e) = ctx.tick_service.stop_timer_tick_loop().await {
            log::error!("Tray reset phase: failed to stop tick loop: {}", e);
            return;
        }
        if let Err(e) = ctx.tick_service.load_state().await {
            log::error!("Tray reset phase: failed to load timer state: {}", e);
            return;
        }

        match ctx.timer_repo.get().await {
            Ok(updated) if updated.is_running() => {
                if let Err(e) = ctx
                    .tick_service
                    .start_timer_tick_loop(Some(task.config().timer.clone()))
                    .await
                {
                    log::error!(
                        "Tray reset phase: failed to restart tick loop: {}",
                        e
                    );
                }
            }
            Ok(_) => { /* paused: leave the loop stopped */ }
            Err(e) => {
                log::error!("Tray reset phase: failed to read timer: {}", e)
            }
        }
    });
}

/// Skip to the next phase, mirroring the React "Skip Phase" button. Per the
/// tick-loop ownership contract, this handler drives stop/load/start directly.
/// Previously skip-from-tray never restarted the loop (no handler did), so the
/// timer appeared stuck after a skip.
fn menu_skip(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let _orchestration_lock = ctx.tick_service.orchestration_lock().await;
        let task = match ctx.task_repo.get_by_id(ctx.task_id).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                log::error!("Tray skip: task {} not found", ctx.task_id);
                return;
            }
            Err(e) => {
                log::error!("Tray skip: failed to load task: {}", e);
                return;
            }
        };

        if let Err(e) = skip_timer_phase(
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_publisher.clone(),
            ctx.task_id,
        )
        .await
        {
            log::error!("Tray skip phase failed: {}", e);
            return;
        }

        if let Err(e) = ctx.tick_service.stop_timer_tick_loop().await {
            log::error!("Tray skip: failed to stop tick loop: {}", e);
            return;
        }
        if let Err(e) = ctx.tick_service.load_state().await {
            log::error!("Tray skip: failed to load timer state: {}", e);
            return;
        }

        match ctx.timer_repo.get().await {
            Ok(updated) if updated.is_running() => {
                if let Err(e) = ctx
                    .tick_service
                    .start_timer_tick_loop(Some(task.config().timer.clone()))
                    .await
                {
                    log::error!(
                        "Tray skip: failed to restart tick loop: {}",
                        e
                    );
                }
            }
            Ok(_) => { /* paused: leave the loop stopped */ }
            Err(e) => log::error!("Tray skip: failed to read timer: {}", e),
        }
    });
}

/// Reset the active task's progress (completed sessions), mirroring the React
/// "Reset Task" button. Also resets the timer to idle.
fn menu_reset_task(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let _orchestration_lock = ctx.tick_service.orchestration_lock().await;
        if let Err(e) = reset_task_uc(
            ctx.task_repo.clone(),
            ctx.timer_repo.clone(),
            ctx.event_publisher.clone(),
            ctx.task_id,
        )
        .await
        {
            log::error!("Tray reset task failed: {}", e);
        }

        // Per the tick-loop ownership contract, drive the stop directly. No
        // sleep — the TaskReset event handler is a UI-only emitter now.
        if let Err(e) = ctx.tick_service.stop_timer_tick_loop().await {
            log::error!("Tray reset task: failed to stop tick loop: {}", e);
        }
        if let Err(e) = ctx.tick_service.load_state().await {
            log::error!("Tray reset task: failed to load timer state: {}", e);
        }
    });
}

/// Complete the active task (force-complete all sessions), mirroring the React
/// "Complete Task" button. Delegates to the shared `complete_task_flow` used by
/// the Tauri command so behavior is identical (stop + reset timer, optional
/// auto-advance).
fn menu_complete(app: &AppHandle) {
    let Some(ctx) = TrayCtx::try_load(app) else {
        return;
    };
    ctx.spawn(|ctx| async move {
        let _orchestration_lock = ctx.tick_service.orchestration_lock().await;
        if let Err(e) = complete_task_flow(
            ctx.task_id,
            ctx.task_repo,
            ctx.timer_repo,
            ctx.config_repo,
            ctx.event_publisher,
            ctx.tick_service.clone(),
            ctx.app,
        )
        .await
        {
            log::error!("Tray complete task failed: {}", e);
        }
    });
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
        let was_visible =
            window.is_visible().unwrap_or_else(|_| intended_visible());
        if was_visible {
            let _ = window.hide();
            set_intended_visible(false);
        } else {
            let _ = window.show();
            let _ = window.set_focus();
            set_intended_visible(true);
        }
    }
    let _ = refresh(app, None);
}

fn quit_app(app: &AppHandle) {
    let _ = app.emit("app:exited", ());
    app.exit(0);
}

/// Subscribe to timer + task + config events so the tray stays in sync.
///
/// These callbacks run on the emitter's thread (a tokio worker for ticks), so
/// they must stay cheap: they only push a signal onto the coalescing channel
/// and let the background `refresh_loop` do the actual work.
fn register_event_listeners(app: &AppHandle) {
    app.listen(ui_listeners::timer::TICK, move |event| {
        let remaining =
            serde_json::from_str::<serde_json::Value>(event.payload())
                .ok()
                .and_then(|v| {
                    v.get("remaining_seconds")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32)
                });
        match remaining {
            Some(s) => schedule_refresh(RefreshSignal::Tick(s)),
            None => schedule_refresh(RefreshSignal::Dirty),
        }
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
        app.listen(evt, move |_| {
            schedule_refresh(RefreshSignal::Dirty);
        });
    }

    app.listen(ui_listeners::config::CONFIG_UPDATED, move |_| {
        schedule_refresh(RefreshSignal::Dirty);
    });
}

/// Re-read timer + general config + active task, then update the tray
/// icon/tooltip/title and the menu item labels/enabled-state.
///
/// Synchronous entry point used by the rare callers that are not on the async
/// runtime path (initial paint in `build_tray`, window close-to-tray). The
/// listener/menu-handler path goes through the background `refresh_loop`
/// instead — see `schedule_refresh`.
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
    render_refresh(
        app,
        &timer,
        &general,
        task.as_ref(),
        remaining_secs_override,
    )
}

/// Async counterpart of [`refresh`] used by the coalescing `refresh_loop`.
/// Reads repositories directly with `.await` (no `block_on`, no extra OS
/// thread) so it never stalls the tokio worker that emitted the event.
async fn refresh_async(
    app: &AppHandle,
    remaining_secs_override: Option<u32>,
) -> tauri::Result<()> {
    let Some((timer, general, task)) = load_state_async(app).await else {
        return Ok(());
    };
    render_refresh(
        app,
        &timer,
        &general,
        task.as_ref(),
        remaining_secs_override,
    )
}

/// Apply the loaded state to the tray icon/tooltip/title and the menu items.
/// Pure UI mutation — no async, no blocking. Shared by the sync and async
/// refresh entry points.
///
/// The countdown label is diffed against [`LAST_LABEL`]; the (relatively
/// expensive) icon pixel rebuild + `set_icon`/`set_title` calls are skipped
/// when the displayed text has not changed. For a per-second tick this still
/// repaints once per second (the label changes each second), but it eliminates
/// the redundant re-renders from the many status/task events that fan out per
/// user action, and avoids any work while idle.
fn render_refresh(
    app: &AppHandle,
    timer: &Timer,
    general: &GeneralConfig,
    task: Option<&Task>,
    remaining_secs_override: Option<u32>,
) -> tauri::Result<()> {
    let status = timer.status();
    let phase = timer.get_current_phase();
    let running = matches!(status, TimerStatus::Running);
    let paused = matches!(status, TimerStatus::Paused);
    let idle = matches!(status, TimerStatus::Idle | TimerStatus::Stopped);
    // A "session" exists whenever a task is attached — even if the timer is
    // currently stopped. This is what gates the action availability.
    let has_task = timer.task_id().is_some();

    // Mirror the React `useTimerSession` gating flags.
    let is_task_completed = task.is_some_and(|t| {
        matches!(t.status(), TaskStatus::Completed)
            && t.completed_at().is_some()
    });
    let is_break = matches!(phase, Phase::ShortBreak | Phase::LongBreak);
    let is_last_break = task.is_some_and(|t| {
        t.completed_at().is_none()
            && matches!(t.status(), TaskStatus::Completed)
            && is_break
    });

    // Compute the single countdown label to display. When
    // `show_countdown_in_tray` is on and a task is bound, the remaining time
    // is always shown — baked into the icon on Linux, as title text beside
    // the icon on macOS/Windows.
    let label = if general.show_countdown_in_tray && has_task {
        let config = task.map(|t| &t.config().timer);
        let secs = remaining_secs_override
            .unwrap_or_else(|| timer.remaining_seconds(config));
        Some(format!("{:02}:{:02}", secs / 60, secs % 60))
    } else {
        None
    };

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_tooltip(Some(tooltip_text(status, phase, has_task)))?;

        // Only repaint the icon/title when the displayed text actually
        // changes — coalesced bursts that don't alter the label (e.g. several
        // task events arriving within the same second) become no-ops.
        //
        // NOTE: we only *commit* LAST_LABEL after the platform update
        // succeeds, so a transient failure (e.g. D-Bus not ready on Linux)
        // doesn't poison the cache and cause all subsequent refreshes with
        // the same label to be skipped — which would permanently hide the
        // countdown.
        let label_changed = match LAST_LABEL.lock() {
            Ok(g) => *g != label,
            Err(_) => true,
        };

        if is_linux() {
            // Linux tray backends ignore `set_title`, so we bake the
            // countdown text directly into the icon pixels.
            if label_changed {
                let icon =
                    overlay_countdown(toro_base_icon(), label.as_deref());
                tray.set_icon(Some(icon))?;
                // Commit only after successful set_icon.
                if let Ok(mut g) = LAST_LABEL.lock() {
                    *g = label.clone();
                }
            }
            let _ = tray.set_title::<&str>(None);
        } else {
            // macOS / Windows: `set_title` renders native crisp text.
            // NOTE: macOS's tray backend ignores `set_title(None)` — the old
            // title persists. To actually clear it we must pass `Some("")`.
            if label_changed {
                let title = label.as_deref().unwrap_or("");
                tray.set_title(Some(title))?;
                tray.set_icon(Some(toro_base_icon().clone()))?;
                // Commit only after successful updates.
                if let Ok(mut g) = LAST_LABEL.lock() {
                    *g = label.clone();
                }
            }
        }
    }

    if let Some(h) = app.try_state::<TrayMenuHandles>() {
        // Reflect window visibility in the toggle label. Use the tracked
        // intended visibility rather than `window.is_visible()`, which lags
        // behind hide()/show() on Linux/GTK and would leave the label stale
        // for one click.
        let visible = intended_visible();
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
/// thread. The only remaining callers of [`load_state`] are synchronous setup
/// (`build_tray` initial paint) and window event handlers, which usually run
/// on the main thread — but defensively, when an active runtime is detected we
/// hop onto a bare OS thread so `block_on` is driven from outside the worker
/// pool. The hot listener path no longer touches this; it goes through
/// [`refresh_async`] on the runtime directly.
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
///
/// Synchronous wrapper used by the non-async refresh path (`build_tray` initial
/// paint, window close-to-tray, [`current_general`]). It borrows the repos and
/// drives the inner future via [`block_on_safe`].
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

    block_on_safe(load_state_inner(timer_repo, config_repo, task_repo))
}

/// Async load used by the coalescing `refresh_loop`; identical to
/// [`load_state`] but without the `block_on` hop so it never stalls a tokio
/// worker.
async fn load_state_async(
    app: &AppHandle,
) -> Option<(Timer, GeneralConfig, Option<Task>)> {
    let timer_repo = app.try_state::<TimerRepositoryArc>()?.inner().clone();
    let config_repo = app
        .try_state::<Arc<dyn ConfigRepository + Send + Sync>>()?
        .inner()
        .clone();
    let task_repo = app
        .try_state::<Arc<dyn TaskRepository + Send + Sync>>()?
        .inner()
        .clone();
    load_state_inner(timer_repo, config_repo, task_repo).await
}

/// Shared repository reads backing both the sync and async load paths.
async fn load_state_inner(
    timer_repo: TimerRepositoryArc,
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
) -> Option<(Timer, GeneralConfig, Option<Task>)> {
    let timer = timer_repo.get().await.ok()?;
    let general = config_repo.get_config().await.ok()?.general;
    let task = match timer.task_id() {
        Some(tid) => task_repo.get_by_id(tid).await.ok().flatten(),
        None => None,
    };
    Some((timer, general, task))
}

/// Read the current `GeneralConfig` (best-effort; falls back to defaults if
/// the repositories aren't ready yet).
pub fn current_general(app: &AppHandle) -> GeneralConfig {
    match load_state(app) {
        Some((_, g, _)) => g,
        None => GeneralConfig::default(),
    }
}
