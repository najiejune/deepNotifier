use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub id: String,
    pub source: NotificationSource,
    pub event_type: String,
    pub title: String,
    pub body: String,
    pub severity: Severity,
    pub timestamp: DateTime<Local>,
    pub raw_payload: Option<serde_json::Value>,
    pub url: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum NotificationSource {
    GitHub,
    GitLab,
    Bitbucket,
    Custom,
    Poll { endpoint_name: String },
    Timer,
    Pomodoro,
    System,
    Hook { cli_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

pub fn start(app_handle: tauri::AppHandle, state: crate::state::AppState) {
    let rx = state.notification_rx.clone();
    let dnd_active = state.dnd_active.clone();
    let history = state.history.clone();
    let config = state.config.clone();
    let pending_pid = state.pending_pid.clone();

    tauri::async_runtime::spawn(async move {
        // Take the receiver out
        let receiver = {
            let mut guard = rx.write().await;
            guard.take()
        };

        if let Some(mut rx) = receiver {
            while let Some(event) = rx.recv().await {
                // Always store in history
                {
                    let mut hist = history.write().await;
                    hist.push(event.clone());
                }

                let cfg = config.read().await;
                let dnd = *dnd_active.read().await;

                tracing::info!("Dispatching notification: '{}', DND={}", event.title, dnd);

                // Emit to frontend always (so user can see notifications in dashboard)
                let _ = app_handle.emit("notification", &event);

                if dnd {
                    tracing::info!("DND active, suppressing all notifications: {}", event.title);
                    continue;
                }

                // pretooluse is a timer-starter only — no notification effects.
                // But we still resolve & store the terminal PID now while the CLI process
                // is alive, so that the subsequent stop/posttooluse notification can focus
                // the correct window even after the process has exited.
                if let NotificationSource::Hook { .. } = &event.source {
                    if event.event_type == "pretooluse" {
                        if let Some(pid) = event.pid {
                            let resolved = crate::notifier::window_focus::resolve_terminal_pid(pid);
                            let stored = if resolved != pid {
                                resolved
                            } else {
                                crate::notifier::window_focus::capture_foreground_pid().unwrap_or(pid)
                            };
                            tracing::info!(pid, resolved=stored, "pretooluse: pre-resolved terminal PID while process alive");
                            *pending_pid.write().await = Some(stored);
                            *crate::notifier::tray::NOTIFICATION_CLICK_PID.lock().unwrap() = Some(stored);
                        }
                        continue;
                    }
                }

                // Normal (non-DND) path: all notification methods enabled
                // For Hook events, use shared hook notification settings
                let (sound_enabled, sound_file, marquee_enabled) = match &event.source {
                    NotificationSource::Hook { .. } => {
                        let (sound, file, marq) = match event.event_type.as_str() {
                            "stop" => (cfg.hook.on_stop_sound, cfg.hook.stop_sound_file.clone(), cfg.hook.on_stop_marquee),
                            "notification" => (cfg.hook.on_notification_sound, cfg.hook.notification_sound_file.clone(), cfg.hook.on_notification_marquee),
                            "posttooluse" => (cfg.hook.approval_timeout_sound_enabled, cfg.hook.approval_timeout_sound_file.clone(), false),
                            _ => (false, String::new(), false),
                        };
                        (sound, file, marq)
                    }
                    _ => (
                        cfg.notification.sound_enabled,
                        cfg.notification.sound_file.clone(),
                        cfg.notification.marquee_enabled,
                    ),
                };

                if sound_enabled {
                    crate::notifier::sound::play(&sound_file, &state.config_dir.join("sounds"), cfg.notification.sound_volume);
                }

                if marquee_enabled {
                    let text = format!("{} — {}", event.title, event.body);
                    crate::notifier::marquee::show(&app_handle, &text, &cfg.marquee);
                }

                // Resolve the notification PID to the terminal emulator PID that owns
                // the visible window. Walk the process tree if the CLI process is still alive.
                // If resolution fails, keep the pretooluse-stored PID (which was resolved
                // while the process was alive) instead of falling back to foreground.
                let focus_pid = if let Some(pid) = event.pid {
                    let resolved = crate::notifier::window_focus::resolve_terminal_pid(pid);
                    if resolved != pid {
                        Some(resolved)
                    } else {
                        // Re-resolution failed (process likely exited).
                        // Keep the pretooluse-stored PID if available.
                        let stored = *pending_pid.read().await;
                        if stored.is_some() {
                            tracing::info!(pid, ?stored, "re-resolution failed, keeping pretooluse-stored PID");
                            stored
                        } else {
                            crate::notifier::window_focus::capture_foreground_pid().or(Some(pid))
                        }
                    }
                } else {
                    None
                };
                tracing::info!(original=?event.pid, resolved=?focus_pid, "Stored focus PID");
                *pending_pid.write().await = focus_pid;
                *crate::notifier::tray::NOTIFICATION_CLICK_PID.lock().unwrap() = focus_pid;

                crate::notifier::tray::update_tooltip(&app_handle, &event.title);

                if cfg.notification.tray_enabled {
                    crate::notifier::tray::notify(&event.title, &event.body);
                }
            }
        }
    });
}
