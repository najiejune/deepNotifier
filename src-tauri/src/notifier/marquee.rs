use crate::config::schema::{MarqueeConfig, MarqueePosition};
use std::sync::{OnceLock, RwLock};
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize};

/// Last (text, config) emitted to marquee windows. The webview pulls this on
/// (re)load and polls it once per second, so it can recover when it misses
/// push events — e.g. after WebView2 reclaims a long-hidden window and
/// reloads the page mid-show, or when the event channel to a webview hosted
/// on a dGPU-driven monitor silently stops delivering.
static LAST_STATE: OnceLock<RwLock<Option<(String, MarqueeConfig)>>> = OnceLock::new();

/// Parked position fallback, far off every screen. Marquee windows are NEVER
/// hidden: hiding lets WebView2 suspend/reclaim the page, which then misses
/// Tauri events and renders stale text or nothing at all. Parking the window
/// off-screen keeps it "visible" to the OS, so the page stays fully alive.
pub const PARK_POS: (i32, i32) = (-32000, -32000);

pub fn store_state(text: &str, cfg: &MarqueeConfig) {
    let lock = LAST_STATE.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = lock.write() {
        *guard = Some((text.to_string(), cfg.clone()));
    }
}

#[tauri::command]
pub fn get_marquee_state(
    window: tauri::WebviewWindow,
    dpr: f64,
    w: f64,
    h: f64,
) -> Option<(String, MarqueeConfig)> {
    let state = LAST_STATE
        .get()
        .and_then(|l| l.read().ok())
        .and_then(|g| g.clone());
    // Diagnostic: every (re)loaded marquee page pulls state on mount, reporting
    // its CSS viewport. If a window never pulls, its webview failed to load
    // the page at all; wrong dpr/viewport means a DPI-scale mismatch.
    tracing::info!(
        "Marquee state pulled by '{}': has_state={} dpr={} viewport={}x{}",
        window.label(),
        state.is_some(),
        dpr,
        w,
        h
    );
    state
}

/// Monitor geometry as (origin, size, scale_factor).
type MonitorGeom = (PhysicalPosition<i32>, PhysicalSize<u32>, f64);

/// All marquee windows (static "marquee" + dynamic "marquee-N"), sorted by
/// label so the window<->monitor assignment is deterministic.
pub(crate) fn sorted_marquee_windows(app: &tauri::AppHandle) -> Vec<tauri::WebviewWindow> {
    let mut wins: Vec<tauri::WebviewWindow> = app
        .webview_windows()
        .into_iter()
        .filter(|(label, _)| label == "marquee" || label.starts_with("marquee-"))
        .map(|(_, w)| w)
        .collect();
    wins.sort_by(|a, b| a.label().cmp(b.label()));
    wins
}

/// Monitor geometries sorted by position so the window<->monitor assignment
/// is deterministic.
fn sorted_monitor_geometries(app: &tauri::AppHandle) -> Vec<MonitorGeom> {
    let mut mons: Vec<MonitorGeom> = app
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| (*m.position(), *m.size(), m.scale_factor()))
        .collect();
    mons.sort_by_key(|(pos, _, _)| (pos.x, pos.y));
    mons
}

/// Marquee windows paired with their assigned monitor (by deterministic
/// sorted order on both sides).
pub(crate) fn window_monitor_pairs(
    app: &tauri::AppHandle,
) -> Vec<(tauri::WebviewWindow, Option<MonitorGeom>)> {
    let monitors = sorted_monitor_geometries(app);
    sorted_marquee_windows(app)
        .into_iter()
        .enumerate()
        .map(|(i, w)| (w, monitors.get(i).cloned()))
        .collect()
}

/// Move a marquee window just below its ASSIGNED monitor instead of hiding it
/// (see PARK_POS). Parking below its own monitor keeps the window's
/// nearest-monitor DPI context pinned to that monitor. Never use
/// `current_monitor()` for this: right after a move the position may not have
/// settled, and Windows ties break toward the *primary* display, so the
/// window would get parked under the wrong monitor and adopt the wrong DPI
/// context (which then corrupts every size/position conversion at show time).
fn park_window_on(window: &tauri::WebviewWindow, origin: &PhysicalPosition<i32>, size: &PhysicalSize<u32>) {
    let win_w = window.outer_size().map(|s| s.width).unwrap_or(0) as i32;
    let x = origin.x + (size.width as i32 - win_w) / 2;
    let y = origin.y + size.height as i32 + 200;
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

/// Fallback parking when no monitor geometry is known.
pub fn park_marquee_window(window: &tauri::WebviewWindow) {
    if let Ok(Some(mon)) = window.current_monitor() {
        park_window_on(window, mon.position(), mon.size());
    } else {
        let _ = window.set_position(PhysicalPosition::new(PARK_POS.0, PARK_POS.1));
    }
}

/// Park all marquee windows, each below its assigned monitor.
pub fn park_all(app_handle: &tauri::AppHandle) {
    for (w, mon) in window_monitor_pairs(app_handle) {
        match &mon {
            Some((origin, size, _)) => park_window_on(&w, origin, size),
            None => park_marquee_window(&w),
        }
    }
}

/// Force a window's *client area* (the webview) to exactly cover the given
/// physical rect. Two platform quirks are compensated:
///
/// 1. On mixed-DPI multi-monitor setups Tauri converts physical sizes to
///    logical units using the window's *cached* scale factor, which can
///    disagree with the DPI context Windows actually assigns (window moves
///    settle asynchronously, so the context can lag behind). The distortion
///    is linear, so an additive fixed-point iteration on the REQUEST side
///    (request += target - measured) converges geometrically for any scale
///    mismatch — unlike ratio corrections, which oscillate.
/// 2. Undecorated windows carry ~9px invisible resize borders: the outer
///    rect is wider than the client area, so a window positioned at x=0
///    actually renders its content at x=9..1929 — visibly spilling ~9px onto
///    the adjacent monitor. The position target is shifted left by the
///    measured side-border width so the CLIENT edges land exactly on
///    (x, x+w). (The border pixels themselves never render content.)
///
/// Corrections run in an async retry loop (250ms apart) because SetWindowPos
/// results are not observable synchronously; the pass that measures
/// everything already on target ends the loop.
fn force_geometry(window: &tauri::WebviewWindow, x: i32, y: i32, w: u32, h: u32) {
    let win = window.clone();
    tauri::async_runtime::spawn(async move {
        let (mut req_x, mut req_y) = (x, y);
        let (mut req_w, mut req_h) = (w as i32, h as i32);
        for attempt in 0..6 {
            let _ = win.set_size(PhysicalSize::new(req_w.max(1) as u32, req_h.max(1) as u32));
            let _ = win.set_position(PhysicalPosition::new(req_x, req_y));
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            let (Ok(p), Ok(outer), Ok(inner)) =
                (win.outer_position(), win.outer_size(), win.inner_size())
            else {
                break;
            };
            // Invisible side borders (symmetric); top border is 0 on
            // undecorated windows (the height slack sits at the bottom).
            let side = (outer.width as i32 - inner.width as i32).max(0) / 2;
            let (tx, ty) = (x - side, y);
            let (ex, ey) = (tx - p.x, ty - p.y);
            let (ew, eh) = (w as i32 - inner.width as i32, h as i32 - inner.height as i32);
            if ex == 0 && ey == 0 && ew == 0 && eh == 0 {
                return; // measured exactly on target
            }
            req_x += ex;
            req_y += ey;
            req_w += ew;
            req_h += eh;
            if attempt == 5 {
                tracing::warn!(
                    "Marquee '{}' geometry off: got pos ({}, {}) inner {}x{}, \
                     want client at ({}, {}) {}x{}",
                    win.label(), p.x, p.y, inner.width, inner.height, x, y, w, h
                );
            }
        }
    });
}

/// Resize and move a marquee window so its client area exactly covers the top
/// (or bottom) strip of the given monitor, in physical pixels (DPI aware).
fn place_on_monitor(
    window: &tauri::WebviewWindow,
    origin: &PhysicalPosition<i32>,
    screen: &PhysicalSize<u32>,
    scale: f64,
    cfg: &MarqueeConfig,
) {
    // cfg.height is logical; convert to physical pixels on this monitor
    let bar_h = (cfg.height as f64 * scale).round() as i32;
    let y = match cfg.position {
        MarqueePosition::Top => origin.y + bar_h,
        MarqueePosition::Bottom => origin.y + screen.height as i32 - bar_h,
    };
    // The static window from tauri.conf.json is fixed at 1920 logical px,
    // which overflows onto the adjacent monitor on scaled (e.g. 125%)
    // displays; always force the exact client geometry (DPI-context safe).
    force_geometry(window, origin.x, y, screen.width, bar_h as u32);
}

/// Show all marquee windows with the given text, one per monitor.
pub fn show(app_handle: &tauri::AppHandle, text: &str, marquee_cfg: &MarqueeConfig) {
    store_state(text, marquee_cfg);

    let pairs = window_monitor_pairs(app_handle);
    let labels: Vec<String> = pairs.iter().map(|(w, _)| w.label().to_string()).collect();
    tracing::info!("Marquee show on windows: {:?}", labels);

    for (window, mon) in &pairs {
        if let Some((origin, screen, scale)) = mon {
            place_on_monitor(window, origin, screen, *scale, marquee_cfg);
        }
        let _ = window.emit("marquee-text", &text);
        let _ = window.emit("marquee-config", marquee_cfg);
        let _ = window.show();
        let _ = window.set_ignore_cursor_events(true);
    }

    // Park (not hide!) after the configured duration.
    let duration = marquee_cfg.duration_secs as u64;
    let handle = app_handle.clone();
    let retry_text = text.to_string();
    let retry_cfg = marquee_cfg.clone();
    tauri::async_runtime::spawn(async move {
        // Re-emit shortly after show: if a webview was still (re)loading its
        // page when the first emit fired, this retry reaches it once its
        // listeners are registered.
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        for w in sorted_marquee_windows(&handle) {
            let _ = w.emit("marquee-text", &retry_text);
            let _ = w.emit("marquee-config", &retry_cfg);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(
            duration
                .saturating_mul(1000)
                .saturating_sub(1500)
                .max(500),
        ))
        .await;
        park_all(&handle);
    });
}
