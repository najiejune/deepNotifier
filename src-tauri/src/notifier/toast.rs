use crate::notifier::dispatcher::Severity;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};

/// What the toast webview renders: the currently visible notification cards
/// plus the card background opacity (shared with the marquee's setting).
/// Emitted as `toast-state` on every change and pulled via `get_toast_state`
/// on page (re)load + once per second — same dual-channel rationale as the
/// marquee (hybrid-GPU webviews can silently stop receiving Tauri events).
#[derive(Debug, Clone, Serialize)]
pub struct ToastSnapshot {
    pub toasts: Vec<ToastItemView>,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToastItemView {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub severity: Severity,
}

#[derive(Debug, Clone)]
struct ToastItem {
    id: u64,
    title: String,
    body: String,
    severity: Severity,
    /// Terminal PID to focus when the card body is clicked.
    pid: Option<u32>,
    /// Auto-dismiss after this long; None = sticky (manual close only).
    dwell: Option<Duration>,
    entered_at: Instant,
}

impl ToastItem {
    fn view(&self) -> ToastItemView {
        ToastItemView {
            id: self.id,
            title: self.title.clone(),
            body: self.body.clone(),
            severity: self.severity.clone(),
        }
    }
}

static LAST_STATE: OnceLock<RwLock<Option<ToastSnapshot>>> = OnceLock::new();

/// Max cards on screen at once; extras wait in the pending queue.
const MAX_VISIBLE: usize = 4;
/// Max queued (not yet displayed) toasts; oldest are dropped beyond this so a
/// notification storm cannot pin cards on screen forever.
const MAX_PENDING: usize = 20;
/// Scheduler tick interval. Dwell times are ≥1s, so 500ms is plenty of
/// resolution and keeps lock hold times negligible.
const TICK_MS: u64 = 500;

/// Per-severity auto-dismiss durations. None = sticky (manual close only),
/// configured in seconds with 0 mapping to None.
#[derive(Debug, Clone, Copy)]
pub struct ToastDurations {
    pub info: Option<Duration>,
    pub warning: Option<Duration>,
    pub critical: Option<Duration>,
}

impl ToastDurations {
    pub fn from_secs(info: u32, warning: u32, critical: u32) -> Self {
        fn secs(s: u32) -> Option<Duration> {
            if s == 0 {
                None
            } else {
                Some(Duration::from_secs(s as u64))
            }
        }
        Self {
            info: secs(info),
            warning: secs(warning),
            critical: secs(critical),
        }
    }

    fn for_severity(&self, severity: &Severity) -> Option<Duration> {
        match severity {
            Severity::Info => self.info,
            Severity::Warning => self.warning,
            Severity::Critical => self.critical,
        }
    }
}
/// Card geometry in logical pixels — must match toast-main.tsx exactly, the
/// backend sizes the window from these numbers.
pub const WIDTH_LOGICAL: f64 = 380.0;
pub const CARD_H_LOGICAL: f64 = 92.0;
pub const GAP_LOGICAL: f64 = 8.0;
/// Transparent padding inside the window on every side. Only needs to keep
/// the card's rounded corners clear of the window edge (Windows rounds
/// transparent window corners). No box-shadow is used on the cards: a shadow
/// reaching the window edge is clipped flat and shows as a faint second
/// rectangle around the card — and a large padded window would swallow
/// clicks meant for the windows beneath it.
pub const PAD_LOGICAL: f64 = 8.0;
const MARGIN_LOGICAL: f64 = 12.0;

/// Strip residual frame styles from the toast window. Undecorated windows
/// keep WS_CAPTION-adjacent styles; dropping the style bits removes the
/// invisible resize borders, so force_geometry's side-border compensation
/// simply measures 0. (The DWM undecorated shadow — a hard-edged rectangle
/// on old Windows builds — is disabled at creation via `.shadow(false)`;
/// DWMWA_NCRENDERING_POLICY was tried and rejected: it kills the glass frame
/// Tauri's transparent windows composite through, leaving an opaque window.)
#[cfg(target_os = "windows")]
pub fn strip_window_frame(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::*;
    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    let hwnd = HWND(hwnd.0 as *mut core::ffi::c_void);
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        let new_style = style & !((WS_BORDER.0 | WS_DLGFRAME.0 | WS_THICKFRAME.0) as i32);
        SetWindowLongW(hwnd, GWL_STYLE, new_style);
        let exstyle = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let new_exstyle = exstyle
            & !((WS_EX_WINDOWEDGE.0
                | WS_EX_DLGMODALFRAME.0
                | WS_EX_STATICEDGE.0
                | WS_EX_CLIENTEDGE.0) as i32);
        SetWindowLongW(hwnd, GWL_EXSTYLE, new_exstyle);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

struct Queue {
    visible: VecDeque<ToastItem>,
    pending: VecDeque<ToastItem>,
    next_id: u64,
    /// Card background opacity, mirroring the marquee config.
    opacity: f32,
}

impl Default for Queue {
    fn default() -> Self {
        Self {
            visible: VecDeque::new(),
            pending: VecDeque::new(),
            next_id: 0,
            opacity: 0.9,
        }
    }
}

impl Queue {
    fn snapshot(&self) -> ToastSnapshot {
        ToastSnapshot {
            toasts: self.visible.iter().map(|t| t.view()).collect(),
            opacity: self.opacity,
        }
    }

    /// Move pending items into freed visible slots. Returns true if changed.
    fn fill_visible(&mut self) -> bool {
        let mut changed = false;
        while self.visible.len() < MAX_VISIBLE {
            match self.pending.pop_front() {
                Some(item) => {
                    self.visible.push_back(item);
                    changed = true;
                }
                None => break,
            }
        }
        changed
    }

    /// Core queue mutation, kept app-handle-free so it is unit-testable.
    fn enqueue_item(
        &mut self,
        title: String,
        body: String,
        severity: Severity,
        pid: Option<u32>,
        dwell: Option<Duration>,
    ) {
        self.next_id += 1;
        let item = ToastItem {
            id: self.next_id,
            title,
            body,
            severity,
            pid,
            dwell,
            entered_at: Instant::now(),
        };
        if self.visible.len() < MAX_VISIBLE {
            self.visible.push_back(item);
        } else {
            self.pending.push_back(item);
            while self.pending.len() > MAX_PENDING {
                if let Some(dropped) = self.pending.pop_front() {
                    tracing::warn!(
                        "Toast queue full ({}), dropped oldest: {}",
                        MAX_PENDING,
                        dropped.title
                    );
                }
            }
        }
    }

    /// Remove a toast by id (visible or pending), refilling visible slots.
    /// Returns the removed item's focus PID, if any.
    fn dismiss_item(&mut self, id: u64) -> Option<Option<u32>> {
        if let Some(pos) = self.visible.iter().position(|t| t.id == id) {
            let item = self.visible.remove(pos).expect("position checked");
            self.fill_visible();
            Some(item.pid)
        } else if let Some(pos) = self.pending.iter().position(|t| t.id == id) {
            let item = self.pending.remove(pos).expect("position checked");
            Some(item.pid)
        } else {
            None
        }
    }

    /// Restart the auto-dismiss timer for a visible toast (hover pause).
    fn touch_item(&mut self, id: u64) {
        if let Some(item) = self.visible.iter_mut().find(|t| t.id == id) {
            item.entered_at = Instant::now();
        }
    }

    /// Drop expired toasts and refill from pending. Sticky items (dwell None)
    /// never expire. Returns true if the visible set changed.
    fn expire(&mut self, now: Instant) -> bool {
        let before: Vec<u64> = self.visible.iter().map(|t| t.id).collect();
        self.visible.retain(|t| match t.dwell {
            None => true,
            Some(d) => now.duration_since(t.entered_at) < d,
        });
        let mut changed = before.len() != self.visible.len()
            || before.iter().zip(self.visible.iter()).any(|(a, b)| *a != b.id);
        changed |= self.fill_visible();
        changed
    }

    fn is_idle(&self) -> bool {
        self.visible.is_empty() && self.pending.is_empty()
    }
}

static QUEUE: OnceLock<Mutex<Queue>> = OnceLock::new();
static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);

fn queue_lock() -> &'static Mutex<Queue> {
    QUEUE.get_or_init(|| Mutex::new(Queue::default()))
}

fn store_snapshot(snapshot: &ToastSnapshot) {
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

/// Push the snapshot to the toast window, placing it at the primary monitor's
/// bottom-right corner when there are cards, parking it off-screen when empty.
fn broadcast(app_handle: &tauri::AppHandle, snapshot: &ToastSnapshot) {
    store_snapshot(snapshot);
    let Some(window) = app_handle.get_webview_window("toast") else {
        tracing::warn!("Toast window not found, dropping broadcast");
        return;
    };
    let count = snapshot.toasts.len();
    if count > 0 {
        place_bottom_right(&window, count);
        let _ = window.show();
    }
    let _ = window.emit("toast-state", snapshot);
    if count == 0 {
        crate::notifier::marquee::park_marquee_window(&window);
    }
}

/// Enqueue a notification for display as a toast card.
pub fn enqueue(
    app_handle: &tauri::AppHandle,
    title: &str,
    body: &str,
    severity: Severity,
    pid: Option<u32>,
    opacity: f32,
    durations: ToastDurations,
) {
    let snapshot = {
        let mut q = queue_lock().lock().unwrap();
        q.opacity = opacity;
        let dwell = durations.for_severity(&severity);
        q.enqueue_item(title.to_string(), body.to_string(), severity.clone(), pid, dwell);
        tracing::info!(
            "Toast enqueue [{:?}]: '{}', visible={}, pending={}",
            severity,
            title,
            q.visible.len(),
            q.pending.len()
        );
        q.snapshot()
    };
    broadcast(app_handle, &snapshot);
    ensure_scheduler(app_handle);
}

fn dismiss_and_broadcast(app_handle: &tauri::AppHandle, id: u64) -> Option<Option<u32>> {
    let (removed, snapshot) = {
        let mut q = queue_lock().lock().unwrap();
        let removed = q.dismiss_item(id);
        (removed, q.snapshot())
    };
    if removed.is_some() {
        broadcast(app_handle, &snapshot);
    }
    removed
}

#[tauri::command]
pub fn toast_dismiss(app_handle: tauri::AppHandle, id: u64) {
    dismiss_and_broadcast(&app_handle, id);
}

/// Card body clicked: focus the source terminal window, then dismiss.
#[tauri::command]
pub fn toast_activate(app_handle: tauri::AppHandle, id: u64) {
    if let Some(Some(pid)) = dismiss_and_broadcast(&app_handle, id) {
        crate::notifier::window_focus::bring_pid_to_front(pid);
    }
}

/// Hover pause: restart the auto-dismiss timer for a visible card.
#[tauri::command]
pub fn toast_keepalive(id: u64) {
    queue_lock().lock().unwrap().touch_item(id);
}

#[tauri::command]
pub fn get_toast_state(window: tauri::WebviewWindow) -> Option<ToastSnapshot> {
    let state = LAST_STATE
        .get()
        .and_then(|l| l.read().ok())
        .and_then(|g| g.clone());
    tracing::info!(
        "Toast state pulled by '{}': has_state={}",
        window.label(),
        state.is_some()
    );
    state
}

/// Preview entry point for the settings page: enqueues a sample Info card.
#[tauri::command]
pub async fn toast_preview(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<(), String> {
    let (opacity, durations) = {
        let cfg = state.config.read().await;
        (
            cfg.marquee.opacity,
            ToastDurations::from_secs(
                cfg.notification.toast_info_secs,
                cfg.notification.toast_warning_secs,
                cfg.notification.toast_critical_secs,
            ),
        )
    };
    enqueue(
        &app_handle,
        "deepNotifier",
        "通知弹窗预览 — This is a notification preview.",
        Severity::Info,
        None,
        opacity,
        durations,
    );
    Ok(())
}

/// Start the expiry scheduler if it is not running. The task drops expired
/// non-critical toasts, refills visible slots from the pending queue, and
/// parks the window (then exits) once everything has drained.
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
                let changed = q.expire(Instant::now());
                let idle = q.is_idle();
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
                let has_work = !queue_lock().lock().unwrap().is_idle();
                if has_work {
                    if !SCHEDULER_RUNNING.swap(true, Ordering::SeqCst) {
                        continue;
                    }
                    return; // a freshly spawned task took over
                }
                if let Some(window) = handle.get_webview_window("toast") {
                    crate::notifier::marquee::park_marquee_window(&window);
                }
                clear_snapshot();
                return;
            }
        }
    });
}

/// Primary monitor work area (screen minus taskbar) in physical pixels, as
/// (left, top, right, bottom). Using the work area keeps the toast clear of
/// the taskbar no matter which screen edge it sits on.
#[cfg(target_os = "windows")]
fn primary_work_area() -> Option<(i32, i32, i32, i32)> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };
    let mut rect = RECT::default();
    unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut rect as *mut RECT as *mut core::ffi::c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .ok()?;
    }
    Some((rect.left, rect.top, rect.right, rect.bottom))
}

/// Resize and move the toast window so its client area sits at the primary
/// monitor's bottom-right corner (above the taskbar), sized to exactly fit
/// `count` cards. Physical pixels, DPI aware — reuses the marquee's
/// force_geometry, which already compensates mixed-DPI scale mismatches and
/// the invisible resize borders of undecorated windows.
fn place_bottom_right(window: &tauri::WebviewWindow, count: usize) {
    let mon = window
        .primary_monitor()
        .ok()
        .flatten()
        .or_else(|| window.current_monitor().ok().flatten());
    let Some(mon) = mon else {
        tracing::warn!("No monitor geometry for toast placement");
        return;
    };
    let scale = mon.scale_factor();
    let w = ((WIDTH_LOGICAL + 2.0 * PAD_LOGICAL) * scale).round() as i32;
    let h = ((CARD_H_LOGICAL * count as f64
        + GAP_LOGICAL * (count - 1) as f64
        + 2.0 * PAD_LOGICAL)
        * scale)
        .round() as i32;
    let margin = (MARGIN_LOGICAL * scale).round() as i32;
    let pad = (PAD_LOGICAL * scale).round() as i32;
    // Bottom-right of the work area (taskbar excluded) when available,
    // falling back to raw monitor bounds on other platforms.
    #[cfg(target_os = "windows")]
    let area = primary_work_area();
    #[cfg(not(target_os = "windows"))]
    let area: Option<(i32, i32, i32, i32)> = None;
    let (right, bottom) = match area {
        Some((_, _, r, b)) => (r, b),
        None => (
            mon.position().x + mon.size().width as i32,
            mon.position().y + mon.size().height as i32,
        ),
    };
    // The window is larger than the cards by PAD on every side; shift the
    // origin up-left so the cards (not the window) keep the visual margin
    // from the work-area corner.
    let x = right - w - margin + pad;
    let y = bottom - h - margin + pad;
    crate::notifier::marquee::force_geometry(window, x, y, w as u32, h as u32);
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEN_S: Option<Duration> = Some(Duration::from_secs(10));

    fn enqueue_n(q: &mut Queue, n: usize) {
        for i in 0..n {
            q.enqueue_item(format!("t{i}"), String::new(), Severity::Info, None, None);
        }
    }

    fn visible_titles(q: &Queue) -> Vec<&str> {
        q.visible.iter().map(|t| t.title.as_str()).collect()
    }

    #[test]
    fn fills_visible_then_queues() {
        let mut q = Queue::default();
        enqueue_n(&mut q, MAX_VISIBLE + 2);
        assert_eq!(q.visible.len(), MAX_VISIBLE);
        assert_eq!(q.pending.len(), 2);
        assert_eq!(visible_titles(&q), vec!["t0", "t1", "t2", "t3"]);
    }

    #[test]
    fn pending_cap_drops_oldest() {
        let mut q = Queue::default();
        enqueue_n(&mut q, MAX_VISIBLE + MAX_PENDING + 3);
        assert_eq!(q.pending.len(), MAX_PENDING);
        assert_eq!(q.pending[0].title, format!("t{}", MAX_VISIBLE + 3));
    }

    #[test]
    fn dismiss_refills_from_pending() {
        let mut q = Queue::default();
        enqueue_n(&mut q, MAX_VISIBLE + 1);
        let id = q.visible[1].id;
        let removed = q.dismiss_item(id);
        assert_eq!(removed, Some(None)); // pid was None
        assert_eq!(visible_titles(&q), vec!["t0", "t2", "t3", "t4"]);
        assert!(q.pending.is_empty());
    }

    #[test]
    fn dismiss_pending_works() {
        let mut q = Queue::default();
        enqueue_n(&mut q, MAX_VISIBLE + 2);
        let id = q.pending[0].id;
        assert!(q.dismiss_item(id).is_some());
        assert_eq!(q.pending.len(), 1);
    }

    #[test]
    fn dismiss_unknown_id_is_noop() {
        let mut q = Queue::default();
        enqueue_n(&mut q, 1);
        assert_eq!(q.dismiss_item(9999), None);
        assert_eq!(q.visible.len(), 1);
    }

    #[test]
    fn expire_drops_old_timed_keeps_sticky() {
        let mut q = Queue::default();
        q.enqueue_item("old".into(), String::new(), Severity::Info, None, TEN_S);
        q.enqueue_item("sticky".into(), String::new(), Severity::Critical, None, None);
        // Age the timed toast past its dwell time.
        q.visible[0].entered_at = Instant::now() - Duration::from_secs(11);
        let now = Instant::now();
        assert!(q.expire(now));
        assert_eq!(visible_titles(&q), vec!["sticky"]);
        // Sticky never expires.
        assert!(!q.expire(now + Duration::from_secs(3600)));
        assert_eq!(visible_titles(&q), vec!["sticky"]);
    }

    #[test]
    fn severity_durations_map_correctly() {
        let d = ToastDurations::from_secs(5, 15, 0);
        assert_eq!(d.for_severity(&Severity::Info), Some(Duration::from_secs(5)));
        assert_eq!(d.for_severity(&Severity::Warning), Some(Duration::from_secs(15)));
        assert_eq!(d.for_severity(&Severity::Critical), None);
    }

    #[test]
    fn expire_refills_visible() {
        let mut q = Queue::default();
        for i in 0..MAX_VISIBLE + 1 {
            q.enqueue_item(format!("t{i}"), String::new(), Severity::Info, None, TEN_S);
        }
        for t in q.visible.iter_mut() {
            t.entered_at = Instant::now() - Duration::from_secs(11);
        }
        assert!(q.expire(Instant::now()));
        assert_eq!(visible_titles(&q), vec!["t4"]);
        assert!(!q.is_idle());
    }

    #[test]
    fn touch_restarts_timer() {
        let mut q = Queue::default();
        q.enqueue_item("a".into(), String::new(), Severity::Info, None, TEN_S);
        q.visible[0].entered_at = Instant::now() - Duration::from_secs(10);
        q.touch_item(q.visible[0].id);
        assert!(!q.expire(Instant::now()));
        assert_eq!(q.visible.len(), 1);
    }
}
