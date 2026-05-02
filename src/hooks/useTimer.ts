import { useState, useEffect, useCallback } from "react";
import { api, onEvent } from "@/lib/tauri";
import type { TimerState } from "@/types";

const defaultTimer: TimerState = {
  mode: "pomodoro",
  status: "idle",
  remaining_secs: 0,
  total_secs: 0,
  pomodoro_round: 0,
};

export function useTimer() {
  const [timer, setTimer] = useState<TimerState>(defaultTimer);

  useEffect(() => {
    api.getTimerState().then(setTimer);
    const unlisten1 = onEvent<number>("timer-tick", (remaining_secs) => {
      setTimer((prev) => ({ ...prev, remaining_secs }));
    });
    const unlisten2 = onEvent<void>("timer-completed", () => {
      setTimer((prev) => ({ ...prev, status: "completed" as const }));
    });
    return () => {
      unlisten1.then((fn) => fn());
      unlisten2.then((fn) => fn());
    };
  }, []);

  const startPomodoro = useCallback(
    async (opts?: {
      work_mins?: number;
      short_break_mins?: number;
      long_break_mins?: number;
      rounds?: number;
    }) => {
      await api.startPomodoro(opts);
      setTimer((prev) => ({
        ...prev,
        mode: "pomodoro",
        status: "running",
        pomodoro_round: 1,
        pomodoro_phase: "work",
      }));
    },
    [],
  );

  const pause = useCallback(async () => {
    await api.pauseTimer();
    setTimer((prev) => ({ ...prev, status: "paused" }));
  }, []);

  const stop = useCallback(async () => {
    await api.stopTimer();
    setTimer((prev) => ({ ...prev, status: "idle", remaining_secs: 0 }));
  }, []);

  const resume = useCallback(async () => {
    // Resume the previous pomodoro
    setTimer((prev) => ({ ...prev, status: "running" }));
    await api.startPomodoro();
  }, []);

  return { timer, startPomodoro, pause, resume, stop };
}
