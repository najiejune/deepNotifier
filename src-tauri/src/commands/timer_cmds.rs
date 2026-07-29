use crate::state::AppState;
use crate::timer::engine::{TimerMode, TimerState, TimerStatus};
use crate::notifier::dispatcher::{NotificationEvent, NotificationSource, Severity};
use tauri::{Emitter, State};

async fn cancel_existing_timer(state: &AppState) {
    if let Some(cancel) = state.timer_cancel.write().await.take() {
        let _ = cancel.send(());
    }
}

fn spawn_timer_tick(app_handle: tauri::AppHandle, state: AppState) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        // skip the first immediate tick
        interval.tick().await;

        loop {
            interval.tick().await;

            let should_break = {
                let mut ts = state.timer_state.write().await;
                if ts.status != TimerStatus::Running {
                    // Check if cancelled
                    let cancelled = state.timer_cancel.read().await.is_none();
                    if cancelled && ts.status == TimerStatus::Idle {
                        break;
                    }
                    continue;
                }
                if ts.remaining_secs > 0 {
                    ts.remaining_secs -= 1;
                    let _ = app_handle.emit("timer-tick", ts.remaining_secs);

                    if ts.remaining_secs == 0 {
                        ts.status = TimerStatus::Completed;
                        let _mode = ts.mode.clone();
                        true // should send notification
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if should_break {
                // Play sound directly for immediate feedback
                let cfg = state.config.read().await;
                let sound_file = cfg.notification.sound_file.clone();
                let sounds_dir = state.config_dir.join("sounds");
                let volume = cfg.notification.sound_volume;
                drop(cfg);
                crate::notifier::sound::play(&sound_file, &sounds_dir, volume);

                // Send completion notification via dispatcher
                let mode = state.timer_state.read().await.mode.clone();
                let label = match mode {
                    TimerMode::Pomodoro => "Pomodoro",
                    TimerMode::Countdown => "Timer",
                };
                let source = match mode {
                    TimerMode::Pomodoro => NotificationSource::Pomodoro,
                    TimerMode::Countdown => NotificationSource::Timer,
                };
                let event = NotificationEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    source,
                    event_type: "timer.completed".into(),
                    title: format!("{} Completed", label),
                    body: "Time's up!".into(),
                    severity: Severity::Warning,
                    timestamp: chrono::Local::now(),
                    raw_payload: None,
                    url: None,
                    pid: None,
                };
                let _ = state.notification_tx.send(event).await;
                let _ = app_handle.emit("timer-completed", ());
                break;
            }
        }
    });
}

#[tauri::command]
pub async fn stop_timer(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(cancel) = state.timer_cancel.write().await.take() {
        let _ = cancel.send(());
    }
    let mut ts = state.timer_state.write().await;
    ts.status = TimerStatus::Idle;
    ts.remaining_secs = 0;
    Ok(())
}

#[tauri::command]
pub async fn pause_timer(state: State<'_, AppState>) -> Result<(), String> {
    let mut ts = state.timer_state.write().await;
    if matches!(ts.status, TimerStatus::Running) {
        ts.status = TimerStatus::Paused;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_timer_state(state: State<'_, AppState>) -> Result<TimerState, String> {
    let ts = state.timer_state.read().await;
    Ok(ts.clone())
}

#[tauri::command]
pub async fn start_pomodoro(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    work_mins: Option<u32>,
    short_break_mins: Option<u32>,
    long_break_mins: Option<u32>,
    rounds: Option<u32>,
) -> Result<(), String> {
    cancel_existing_timer(state.inner()).await;

    let config = state.config.read().await;
    let work_secs = work_mins
        .unwrap_or(config.timer.pomodoro_work_mins) as u64 * 60;
    let _sbreak_secs = short_break_mins
        .unwrap_or(config.timer.pomodoro_short_break_mins) as u64 * 60;
    let _lbreak_secs = long_break_mins
        .unwrap_or(config.timer.pomodoro_long_break_mins) as u64 * 60;
    let _rounds = rounds.unwrap_or(config.timer.pomodoro_rounds);
    drop(config);

    {
        let mut ts = state.timer_state.write().await;
        ts.mode = TimerMode::Pomodoro;
        ts.status = TimerStatus::Running;
        ts.total_secs = work_secs;
        ts.remaining_secs = work_secs;
        ts.pomodoro_round = 1;
        ts.pomodoro_phase = Some(crate::timer::engine::PomodoroPhase::Work);
    }

    let (tx, _rx) = tokio::sync::oneshot::channel::<()>();
    *state.timer_cancel.write().await = Some(tx);

    spawn_timer_tick(app, state.inner().clone());
    Ok(())
}
