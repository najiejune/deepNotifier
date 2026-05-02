import { useState } from "react";
import { Play, Pause, Square, RotateCcw } from "lucide-react";
import { cn } from "@/lib/cn";
import { Button } from "@/components/ui/Button";
import { useI18n } from "@/i18n";
import type { TimerState } from "@/types";

interface TimerDisplayProps {
  timer: TimerState;
  onStart: (durationSecs: number) => void;
  onPause: () => void;
  onStop: () => void;
  onStartPomodoro: () => void;
}

function formatTime(secs: number) {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

export function TimerDisplay({
  timer,
  onStart,
  onPause,
  onStop,
  onStartPomodoro,
}: TimerDisplayProps) {
  const t = useI18n();
  const [minutes, setMinutes] = useState(25);
  const isActive = timer.status === "running";
  const isIdle = timer.status === "idle" || timer.status === "completed";

  const phaseLabel =
    timer.mode === "pomodoro"
      ? timer.pomodoro_phase === "work"
        ? t.dashboard.focus
        : timer.pomodoro_phase === "short_break"
          ? t.dashboard.break
          : t.dashboard.longBreak
      : null;

  return (
    <div className="bg-white border border-border-subtle rounded-md p-5 shadow-sm">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <span className="text-[10px] font-mono uppercase tracking-[0.2em] text-text-muted">
            {timer.mode === "pomodoro" ? t.dashboard.pomodoro : t.dashboard.timer}
          </span>
          {phaseLabel && (
            <span className="text-[10px] font-mono text-accent bg-accent-dim px-1.5 py-0.5 rounded-sm">
              {phaseLabel} {timer.pomodoro_round}
            </span>
          )}
        </div>
        {isActive && (
          <span className="flex items-center gap-1.5 text-[10px] font-mono text-cyan">
            <span className="relative flex h-2 w-2">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-cyan opacity-50" />
              <span className="relative inline-flex rounded-full h-2 w-2 bg-cyan" />
            </span>
            {t.dashboard.running}
          </span>
        )}
      </div>

      <div
        className={cn(
          "font-mono text-5xl tracking-tighter text-center py-3 tabular-nums transition-colors",
          isActive && "text-accent",
          timer.status === "completed" && "text-cyan",
        )}
      >
        {isIdle && timer.mode === "countdown"
          ? formatTime(minutes * 60)
          : formatTime(timer.remaining_secs)}
      </div>

      {isIdle && timer.mode === "countdown" && (
        <div className="flex items-center justify-center gap-2 mt-3">
          <button
            onClick={() => setMinutes((m) => Math.max(1, m - 5))}
            className="w-7 h-7 rounded-sm bg-bg-layer border border-border-subtle text-text-secondary hover:text-text text-xs font-mono transition-colors"
          >
            -5
          </button>
          <div className="flex items-center gap-1">
            <input
              type="number"
              value={minutes}
              onChange={(e) =>
                setMinutes(Math.max(1, Math.min(180, Number(e.target.value))))
              }
              className="w-14 h-7 rounded-sm bg-white border border-border-subtle text-center text-sm font-mono text-text focus:outline-none focus:border-accent [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
            />
            <span className="text-xs text-text-muted font-mono">{t.dashboard.min}</span>
          </div>
          <button
            onClick={() => setMinutes((m) => Math.min(180, m + 5))}
            className="w-7 h-7 rounded-sm bg-bg-layer border border-border-subtle text-text-secondary hover:text-text text-xs font-mono transition-colors"
          >
            +5
          </button>
        </div>
      )}

      <div className="flex items-center justify-center gap-2 mt-4">
        {isIdle ? (
          <>
            <Button size="sm" variant="primary" onClick={() => onStart(minutes * 60)}>
              <Play size={12} /> {t.dashboard.start}
            </Button>
            <Button size="sm" variant="secondary" onClick={onStartPomodoro}>
              <RotateCcw size={12} /> {t.dashboard.pomodoro}
            </Button>
          </>
        ) : isActive ? (
          <>
            <Button size="sm" variant="secondary" onClick={onPause}>
              <Pause size={12} /> {t.dashboard.pause}
            </Button>
            <Button size="sm" variant="danger" onClick={onStop}>
              <Square size={12} /> {t.dashboard.stop}
            </Button>
          </>
        ) : (
          <Button size="sm" variant="secondary" onClick={onStop}>
            <Square size={12} /> {t.dashboard.reset}
          </Button>
        )}
      </div>
    </div>
  );
}
