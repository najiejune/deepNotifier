use crate::timer::engine::PomodoroPhase;

/// Manages pomodoro state transitions
pub struct PomodoroMachine {
    pub phase: PomodoroPhase,
    pub round: u32,
    pub total_rounds: u32,
    pub work_mins: u32,
    pub short_break_mins: u32,
    pub long_break_mins: u32,
}

impl PomodoroMachine {
    pub fn new(
        total_rounds: u32,
        work_mins: u32,
        short_break_mins: u32,
        long_break_mins: u32,
    ) -> Self {
        Self {
            phase: PomodoroPhase::Work,
            round: 1,
            total_rounds,
            work_mins,
            short_break_mins,
            long_break_mins,
        }
    }

    /// Advance to next phase, returns the new phase and duration in seconds
    pub fn next_phase(&mut self) -> (PomodoroPhase, u64) {
        match self.phase {
            PomodoroPhase::Work => {
                if self.round >= self.total_rounds {
                    self.phase = PomodoroPhase::LongBreak;
                    (PomodoroPhase::LongBreak, self.long_break_mins as u64 * 60)
                } else {
                    self.phase = PomodoroPhase::ShortBreak;
                    (PomodoroPhase::ShortBreak, self.short_break_mins as u64 * 60)
                }
            }
            PomodoroPhase::ShortBreak => {
                self.round += 1;
                self.phase = PomodoroPhase::Work;
                (PomodoroPhase::Work, self.work_mins as u64 * 60)
            }
            PomodoroPhase::LongBreak => {
                self.round = 1;
                self.phase = PomodoroPhase::Work;
                (PomodoroPhase::Work, self.work_mins as u64 * 60)
            }
        }
    }

    /// Get current phase duration in seconds
    pub fn current_duration_secs(&self) -> u64 {
        match self.phase {
            PomodoroPhase::Work => self.work_mins as u64 * 60,
            PomodoroPhase::ShortBreak => self.short_break_mins as u64 * 60,
            PomodoroPhase::LongBreak => self.long_break_mins as u64 * 60,
        }
    }
}
