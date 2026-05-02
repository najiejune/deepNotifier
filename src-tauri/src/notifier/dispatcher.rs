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

                // Normal (non-DND) path: all notification methods enabled
                if cfg.notification.sound_enabled {
                    crate::notifier::sound::play(&cfg.notification.sound_file, &state.config_dir.join("sounds"), cfg.notification.sound_volume);
                }

                if cfg.notification.marquee_enabled {
                    crate::notifier::marquee::show(&app_handle, &event.title, &event.body, &cfg.marquee);
                }

                if cfg.notification.tray_enabled {
                    crate::notifier::tray::notify(&app_handle, &event.title, &event.body);
                }
            }
        }
    });
}
