use crate::config::schema::{MarqueeConfig, MarqueePosition};
use crate::notifier::dispatcher::Severity;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize};

/// What the marquee webviews render: one text slot per track plus the display
/// config. Emitted as `marquee-state` on every change and pulled via
/// `get_marquee_state` on page (re)load + once per second — polling is the
/// reliable channel on hybrid-GPU machines where the dGPU-hosted webview can
/// silently stop receiving Tauri events.
#[derive(Debug, Clone, Serialize)]
pub struct MarqueeSnapshot {
    pub tracks: Vec<Option<String>>,
    pub config: MarqueeConfig,
}

static LAST_STATE: OnceLock<RwLock<Option<MarqueeSnapshot>>> = OnceLock::new();

/// Parked position fallback, far off every screen. Marquee windows are NEVER
/// hidden: hiding lets WebView2 suspend/reclaim the page, which then misses
/// Tauri events and renders stale text or nothing at all. Parking the window
/// off-screen keeps it "visible" to the OS, so the page stays fully alive.
pub const PARK_POS: (i32, i32) = (-32000, -32000);

/// Max queued (not yet displayed) notifications; oldest are dropped beyond
/// this so a notification storm cannot pin the bar on screen forever.
const MAX_PENDING: usize = 20;
/// Scheduler tick interval. Track dwell time is `duration_secs`, so a 500ms
/// tick is plenty of resolution and keeps lock hold times negligible.
const TICK_MS: u64 = 500;

struct TrackSlot {
    text: String,
    severity: Severity,
    entered_at: Instant,
}

struct Queue {
    tracks: Vec<Option<TrackSlot>>,
    pending: VecDeque<(String, Severity)>,
    config: MarqueeConfig,
}

impl Queue {
    fn new() -> Self {
        Self {
            tracks: Vec::new(),
            pending: VecDeque::new(),
            config: MarqueeConfig::default(),
        }
    }

    fn snapshot(&self) -> MarqueeSnapshot {
        MarqueeSnapshot {
            tracks: self
                .tracks
                .iter()
                .map(|l| l.as_ref().map(|s| s.text.clone()))
                .collect(),
            config: self.config.clone(),
        }
    }

    /// Move queued items into empty tracks. Returns true if any track changed.
    fn fill_empty_tracks(&mut self) -> bool {
        let mut changed = false;
        // Split the borrow so tracks and pending can be mutated together.
        let Queue { tracks, pending, .. } = self;
        for track in tracks.iter_mut() {
            if track.is_none() {
                if let Some((text, severity)) = pending.pop_front() {
                    *track = Some(TrackSlot {
                        text,
                        severity,
                        entered_at: Instant::now(),
                    });
                    changed = true;
                }
            }
        }
        changed
    }

    /// Resize the track array to `n`. Items on removed tracks go back to the
    /// queue head, keeping their original track order.
    fn set_track_count(&mut self, n: usize) {
        while self.tracks.len() > n {
            if let Some(slot) = self.tracks.pop().flatten() {
                self.pending.push_front((slot.text, slot.severity));
            }
        }
        self.tracks.resize_with(n, || None);
    }

    /// Core queue mutation, kept app-handle-free so it is unit-testable.
    fn enqueue_item(&mut self, text: String, severity: Severity) {
        if matches!(severity, Severity::Critical) {
            if let Some(old) = self.tracks[0].take() {
                if !matches!(old.severity, Severity::Critical) {
                    self.pending.push_front((old.text, old.severity));
                }
            }
            self.tracks[0] = Some(TrackSlot {
                text,
                severity,
                entered_at: Instant::now(),
            });
        } else {
            self.pending.push_back((text, severity));
            while self.pending.len() > MAX_PENDING {
                if let Some((dropped, _)) = self.pending.pop_front() {
                    tracing::warn!(
                        "Marquee queue full ({}), dropped oldest: {}",
                        MAX_PENDING,
                        dropped
                    );
                }
            }
            // Fill free tracks right away so the first of a burst shows without
            // waiting for a scheduler tick.
            self.fill_empty_tracks();
        }
    }
}

static QUEUE: OnceLock<Mutex<Queue>> = OnceLock::new();
static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);

fn queue_lock() -> &'static Mutex<Queue> {
    QUEUE.get_or_init(|| Mutex::new(Queue::new()))
}

fn store_snapshot(snapshot: &MarqueeSnapshot) {
    let lock = LAST_STATE.get_or_init(|| RwLock::new(None));
    if let Ok(mut guard) = lock.write() {
        *guard = Some(snapshot.clone());
    }
}

fn clear_snapshot() {
    if let Some(lock) = LAST_STATE.get() {
        if let Ok(mut guard) = lock.write() {
            *guard = None;
        }
    }
}

/// Push the current snapshot to all marquee windows, showing and placing them
/// when there is content to display.
fn broadcast(app_handle: &tauri::AppHandle, snapshot: &MarqueeSnapshot) {
    store_snapshot(snapshot);
    let active = snapshot.tracks.iter().filter(|t| t.is_some()).count();
    for (window, mon) in window_monitor_pairs(app_handle) {
        if active > 0 {
            if let Some((origin, screen, scale)) = &mon {
                place_on_monitor(&window, origin, screen, *scale, &snapshot.config, active);
            }
        }
        let _ = window.emit("marquee-state", snapshot);
        if active > 0 {
            let _ = window.show();
            let _ = window.set_ignore_cursor_events(true);
        }
    }
}

/// Enqueue a notification for display.
///
/// - Critical preempts track 0 immediately: a displaced non-critical item goes
///   back to the front of the pending queue; a displaced critical one is
///   replaced outright.
/// - Info/Warning are appended to the pending queue (capped at MAX_PENDING).
pub fn enqueue(
    app_handle: &tauri::AppHandle,
    text: &str,
    severity: Severity,
    marquee_cfg: &MarqueeConfig,
) {
    let snapshot = {
        let mut q = queue_lock().lock().unwrap();
        q.config = marquee_cfg.clone();
        q.set_track_count(marquee_cfg.track_count());
        q.enqueue_item(text.to_string(), severity.clone());
        tracing::info!(
            "Marquee enqueue [{:?}]: '{}', tracks={:?}, pending={}",
            severity,
            text,
            q.tracks
                .iter()
                .map(|l| l.as_ref().map(|s| s.text.as_str()))
                .collect::<Vec<_>>(),
            q.pending.len()
        );
        q.snapshot()
    };
    broadcast(app_handle, &snapshot);
    ensure_scheduler(app_handle);
}

/// Preview entry point shared with the notification dispatcher path: the
/// preview text is enqueued like any Info notification.
pub fn show(app_handle: &tauri::AppHandle, text: &str, marquee_cfg: &MarqueeConfig) {
    enqueue(app_handle, text, Severity::Info, marquee_cfg);
}

/// Live-apply a new config to whatever is currently on screen (settings
/// changed while bars are visible, e.g. dragging the transparency slider
/// with the preview on). Updates the stored config and re-broadcasts the
/// current snapshot without discarding queued items: tracks displaced by a
/// track-count reduction are re-queued, freed tracks are refilled.
pub fn refresh_config(app_handle: &tauri::AppHandle, marquee_cfg: &MarqueeConfig) {
    let snapshot = {
        let mut q = queue_lock().lock().unwrap();
        q.config = marquee_cfg.clone();
        q.set_track_count(marquee_cfg.track_count());
        q.fill_empty_tracks();
        q.snapshot()
    };
    broadcast(app_handle, &snapshot);
    ensure_scheduler(app_handle);
}

/// Clear queue and tracks, then park all windows (used by the hide command so
/// a cancelled preview cannot leave queued items resurfacing later).
pub fn clear_and_park(app_handle: &tauri::AppHandle) {
    let snapshot = {
        let mut q = queue_lock().lock().unwrap();
        q.pending.clear();
        for track in q.tracks.iter_mut() {
            *track = None;
        }
        q.snapshot()
    };
    for w in sorted_marquee_windows(app_handle) {
        let _ = w.emit("marquee-state", &snapshot);
    }
    clear_snapshot();
    park_all(app_handle);
}

/// Start the track scheduler if it is not running. The task expires tracks
/// after `duration_secs`, refills them from the pending queue, and parks all
/// windows (then exits) once everything has drained.
fn ensure_scheduler(app_handle: &tauri::AppHandle) {
    if SCHEDULER_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(TICK_MS)).await;
            let idle = {
                let mut q = queue_lock().lock().unwrap();
                let dwell = Duration::from_secs(q.config.duration_secs.max(1) as u64);
                let now = Instant::now();
                let mut changed = false;
                // Split the borrow so tracks and pending can be mutated together.
                let Queue { tracks, pending, .. } = &mut *q;
                for track in tracks.iter_mut() {
                    let expired = match track {
                        Some(slot) => now.duration_since(slot.entered_at) >= dwell,
                        None => true,
                    };
                    if expired {
                        let next = pending
                            .pop_front()
                            .map(|(text, severity)| TrackSlot {
                                text,
                                severity,
                                entered_at: now,
                            });
                        if next.as_ref().map(|s| &s.text) != track.as_ref().map(|s| &s.text) {
                            changed = true;
                        }
                        *track = next;
                    }
                }
                let idle = q.pending.is_empty() && q.tracks.iter().all(|l| l.is_none());
                if changed {
                    let snapshot = q.snapshot();
                    // Drop the lock before touching windows/events.
                    drop(q);
                    broadcast(&handle, &snapshot);
                }
                idle
            };
            if idle {
                // Reset the flag first, then re-check: an enqueue that slipped
                // in between the idle check and the flag reset must not be
                // stranded with no running scheduler.
                SCHEDULER_RUNNING.store(false, Ordering::SeqCst);
                let has_work = {
                    let q = queue_lock().lock().unwrap();
                    !q.pending.is_empty() || q.tracks.iter().any(|l| l.is_some())
                };
                if has_work {
                    if !SCHEDULER_RUNNING.swap(true, Ordering::SeqCst) {
                        continue;
                    }
                    return; // a freshly spawned task took over
                }
                park_all(&handle);
                clear_snapshot();
                return;
            }
        }
    });
}

#[tauri::command]
pub fn get_marquee_state(
    window: tauri::WebviewWindow,
    dpr: f64,
    w: f64,
    h: f64,
) -> Option<MarqueeSnapshot> {
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
pub(crate) fn force_geometry(window: &tauri::WebviewWindow, x: i32, y: i32, w: u32, h: u32) {
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
/// The strip is only as tall as the ACTIVE tracks (`active_tracks` rows of
/// `cfg.height` logical pixels), so a single notification does not reserve
/// screen space for idle tracks.
fn place_on_monitor(
    window: &tauri::WebviewWindow,
    origin: &PhysicalPosition<i32>,
    screen: &PhysicalSize<u32>,
    scale: f64,
    cfg: &MarqueeConfig,
    active_tracks: usize,
) {
    // cfg.height is logical; convert to physical pixels on this monitor
    let bar_h = (cfg.height as f64 * active_tracks.max(1) as f64 * scale).round() as i32;
    let y = match cfg.position {
        MarqueePosition::Top => origin.y + bar_h,
        MarqueePosition::Bottom => origin.y + screen.height as i32 - bar_h,
    };
    // The static window from tauri.conf.json is fixed at 1920 logical px,
    // which overflows onto the adjacent monitor on scaled (e.g. 125%)
    // displays; always force the exact client geometry (DPI-context safe).
    force_geometry(window, origin.x, y, screen.width, bar_h as u32);
}


#[cfg(test)]
mod tests {
    use super::*;

    fn queue_with_tracks(n: usize) -> Queue {
        let mut q = Queue::new();
        q.set_track_count(n);
        q
    }

    fn track_text(q: &Queue, i: usize) -> Option<&str> {
        q.tracks[i].as_ref().map(|s| s.text.as_str())
    }

    #[test]
    fn info_fills_free_tracks_then_queues() {
        let mut q = queue_with_tracks(2);
        q.enqueue_item("a".into(), Severity::Info);
        q.enqueue_item("b".into(), Severity::Info);
        q.enqueue_item("c".into(), Severity::Info);
        assert_eq!(track_text(&q, 0), Some("a"));
        assert_eq!(track_text(&q, 1), Some("b"));
        assert_eq!(q.pending.len(), 1);
        assert_eq!(q.pending[0].0, "c");
    }

    #[test]
    fn critical_preempts_track0_and_requeues_displaced() {
        let mut q = queue_with_tracks(2);
        q.enqueue_item("a".into(), Severity::Info);
        q.enqueue_item("b".into(), Severity::Info);
        q.enqueue_item("c".into(), Severity::Info);
        q.enqueue_item("URGENT".into(), Severity::Critical);
        assert_eq!(track_text(&q, 0), Some("URGENT"));
        assert_eq!(track_text(&q, 1), Some("b"));
        // Displaced "a" goes to the queue front, ahead of "c".
        let texts: Vec<&str> = q.pending.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(texts, vec!["a", "c"]);
    }

    #[test]
    fn critical_replaces_critical_without_requeue() {
        let mut q = queue_with_tracks(2);
        q.enqueue_item("first".into(), Severity::Critical);
        q.enqueue_item("second".into(), Severity::Critical);
        assert_eq!(track_text(&q, 0), Some("second"));
        assert!(q.pending.is_empty());
    }

    #[test]
    fn shrink_returns_track_items_to_queue_head_in_order() {
        let mut q = queue_with_tracks(3);
        q.enqueue_item("a".into(), Severity::Info);
        q.enqueue_item("b".into(), Severity::Info);
        q.enqueue_item("c".into(), Severity::Info);
        q.enqueue_item("d".into(), Severity::Info); // queued
        q.set_track_count(1);
        let texts: Vec<&str> = q.pending.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(texts, vec!["b", "c", "d"]);
        assert_eq!(track_text(&q, 0), Some("a"));
    }

    #[test]
    fn queue_cap_drops_oldest() {
        let mut q = queue_with_tracks(1);
        q.enqueue_item("shown".into(), Severity::Info);
        for i in 0..(MAX_PENDING + 5) {
            q.enqueue_item(format!("n{i}"), Severity::Info);
        }
        assert_eq!(q.pending.len(), MAX_PENDING);
        assert_eq!(q.pending[0].0, "n5"); // n0..n4 dropped
        assert_eq!(track_text(&q, 0), Some("shown"));
    }

    #[test]
    fn track_count_clamped() {
        let mut cfg = MarqueeConfig::default();
        cfg.tracks = 0;
        assert_eq!(cfg.track_count(), 1);
        cfg.tracks = 9;
        assert_eq!(cfg.track_count(), 3);
    }
}
