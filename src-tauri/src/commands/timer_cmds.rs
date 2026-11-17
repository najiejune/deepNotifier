use crate::state::AppState;
use crate::timer::engine::{PomodoroPhase, TimerMode, TimerState, TimerStatus};
use crate::timer::pomodoro::PomodoroMachine;
use crate::notifier::dispatcher::{NotificationEvent, NotificationSource, Severity};
use std::sync::atomic::Ordering;
use tauri::{Emitter, State};

async fn cancel_existing_timer(state: &AppState) {
    if let Some(cancel) = state.timer_cancel.write().await.take() {
        let _ = cancel.send(());
    }
}

struct PhaseChange {
    title: String,
    body: String,
    /// Session is over — stop the loop after notifying.
    is_completion: bool,
    /// A work phase just finished — used by the frontend for pomodoro counting.
    work_finished: bool,
}

fn spawn_timer_tick(
    app_handle: tauri::AppHandle,
    state: AppState,
    generation: u64,
    mut cancel_rx: tokio::sync::oneshot::Receiver<()>,
    mut machine: Option<PomodoroMachine>,
) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        // skip the first immediate tick
        interval.tick().await;

        loop {
            tokio::select! {
                _ = &mut cancel_rx => break,
                _ = interval.tick() => {}
            }

            // A newer timer session supersedes this loop (guards the race where
            // this loop was mid-tick while a new timer was started).
            if state.timer_generation.load(Ordering::SeqCst) != generation {
                break;
            }

            let mut change: Option<PhaseChange> = None;
            {
                let mut ts = state.timer_state.write().await;
                // Re-check the generation while holding the write lock: a new
                // session may have bumped it and rewritten the state while this
                // loop was waiting for the lock.
                if state.timer_generation.load(Ordering::SeqCst) != generation {
                    break;
                }
                if ts.status != TimerStatus::Running {
                    // Paused — wait for resume on the next tick.
                    continue;
                }

                if ts.remaining_secs > 0 {
                    ts.remaining_secs -= 1;
                    let _ = app_handle.emit("timer-tick", ts.remaining_secs);
                }

                if ts.remaining_secs == 0 {
                    match ts.mode {
                        TimerMode::Countdown => {
                            ts.status = TimerStatus::Completed;
                            change = Some(PhaseChange {
                                title: "Timer Completed".into(),
                                body: "Time's up!".into(),
                                is_completion: true,
                                work_finished: false,
                            });
                        }
                        TimerMode::Pomodoro => {
                            let finished_work =
                                matches!(ts.pomodoro_phase, Some(PomodoroPhase::Work));
                            let session_over = matches!(
                                ts.pomodoro_phase,
                                Some(PomodoroPhase::LongBreak) | None
                            );
                            match (session_over, machine.as_mut()) {
                                (true, _) | (_, None) => {
                                    // Long break finished (or no machine) — session done.
                                    ts.status = TimerStatus::Completed;
                                    change = Some(PhaseChange {
                                        title: "Pomodoro Completed".into(),
                                        body: "All rounds finished. Well done!".into(),
                                        is_completion: true,
                                        work_finished: finished_work,
                                    });
                                }
                                (false, Some(m)) => {
                                    let (new_phase, duration) = m.next_phase();
                                    ts.pomodoro_phase = Some(new_phase.clone());
                                    ts.pomodoro_round = m.round;
                                    ts.total_secs = duration;
                                    ts.remaining_secs = duration;
                                    let (title, body) = match new_phase {
                                        PomodoroPhase::ShortBreak => (
                                            "Work Session Done".to_string(),
                                            format!(
                                                "Take a {}-min short break.",
                                                m.short_break_mins
                                            ),
                                        ),
                                        PomodoroPhase::LongBreak => (
                                            "Work Session Done".to_string(),
                                            format!(
                                                "All {} rounds done. Take a {}-min long break.",
                                                m.total_rounds, m.long_break_mins
                                            ),
                                        ),
                                        PomodoroPhase::Work => (
                                            "Break Over".to_string(),
                                            format!(
                                                "Back to work — round {}/{}.",
                                                m.round, m.total_rounds
                                            ),
                                        ),
                                    };
                                    change = Some(PhaseChange {
                                        title,
                                        body,
                                        is_completion: false,
                                        work_finished: finished_work,
                                    });
                                    let _ = app_handle.emit("timer-state", ts.clone());
                                }
                            }
                        }
                    }
                }
            }

            let Some(change) = change else { continue };

            if change.work_finished {
                let _ = app_handle.emit("pomodoro-work-completed", ());
            }

            // Sound/marquee/toast are handled by the notification dispatcher,
            // which applies DND and the user's sound settings. Playing sound
            // directly here would bypass DND and double up with the dispatcher.

            let mode = state.timer_state.read().await.mode.clone();
            let source = match mode {
                TimerMode::Pomodoro => NotificationSource::Pomodoro,
                TimerMode::Countdown => NotificationSource::Timer,
            };
            let event = NotificationEvent {
                id: uuid::Uuid::new_v4().to_string(),
                source,
                event_type: if change.is_completion {
                    "timer.completed".into()
                } else {
                    "timer.phase".into()
                },
                title: change.title,
                body: change.body,
                severity: Severity::Warning,
                timestamp: chrono::Local::now(),
                raw_payload: None,
                url: None,
                pid: None,
            };
            let _ = state.notification_tx.send(event).await;

            if change.is_completion {
                let _ = app_handle.emit("timer-completed", ());
                break;
            }
        }
    });
}

#[tauri::command]
pub async fn stop_timer(state: State<'_, AppState>) -> Result<(), String> {
    // Bump the generation so any loop that misses the cancel signal still dies.
    state.timer_generation.fetch_add(1, Ordering::SeqCst);
    cancel_existing_timer(state.inner()).await;
    let mut ts = state.timer_state.write().await;
    ts.status = TimerStatus::Idle;
    ts.remaining_secs = 0;
    ts.total_secs = 0;
    ts.pomodoro_phase = None;
    ts.started_at = None;
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
pub async fn resume_timer(state: State<'_, AppState>) -> Result<(), String> {
    let mut ts = state.timer_state.write().await;
    if matches!(ts.status, TimerStatus::Paused) {
        ts.status = TimerStatus::Running;
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
    // Bump the generation BEFORE cancelling: an old loop that already passed
    // its generation check and is waiting on the timer_state write lock must
    // not run against the new session's state.
    let generation = state.timer_generation.fetch_add(1, Ordering::SeqCst) + 1;
    cancel_existing_timer(state.inner()).await;

    let config = state.config.read().await;
    let work_mins = work_mins.unwrap_or(config.timer.pomodoro_work_mins);
    let short_break_mins =
        short_break_mins.unwrap_or(config.timer.pomodoro_short_break_mins);
    let long_break_mins =
        long_break_mins.unwrap_or(config.timer.pomodoro_long_break_mins);
    let rounds = rounds.unwrap_or(config.timer.pomodoro_rounds);
    drop(config);

    let work_secs = work_mins as u64 * 60;

    {
        let mut ts = state.timer_state.write().await;
        ts.mode = TimerMode::Pomodoro;
        ts.status = TimerStatus::Running;
        ts.total_secs = work_secs;
        ts.remaining_secs = work_secs;
        ts.pomodoro_round = 1;
        ts.pomodoro_phase = Some(PomodoroPhase::Work);
        ts.started_at = Some(chrono::Local::now());
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    *state.timer_cancel.write().await = Some(tx);

    let machine = PomodoroMachine::new(rounds, work_mins, short_break_mins, long_break_mins);
    spawn_timer_tick(app, state.inner().clone(), generation, rx, Some(machine));
    Ok(())
}
