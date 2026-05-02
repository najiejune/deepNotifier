use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub mode: TimerMode,
    pub status: TimerStatus,
    pub remaining_secs: u64,
    pub total_secs: u64,
    pub pomodoro_round: u32,
    pub pomodoro_phase: Option<PomodoroPhase>,
    pub started_at: Option<DateTime<Local>>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            mode: TimerMode::Countdown,
            status: TimerStatus::Idle,
            remaining_secs: 0,
            total_secs: 0,
            pomodoro_round: 0,
            pomodoro_phase: None,
            started_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerMode {
    Countdown,
    Pomodoro,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TimerStatus {
    Idle,
    Running,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PomodoroPhase {
    Work,
    ShortBreak,
    LongBreak,
}
