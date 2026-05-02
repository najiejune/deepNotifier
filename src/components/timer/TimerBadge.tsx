import { cn } from "@/lib/cn";
import type { TimerState } from "@/types";

interface Props {
  timer: TimerState;
  countUp?: boolean;
  className?: string;
}

function formatTime(secs: number) {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

export function TimerBadge({ timer, countUp, className }: Props) {
  const isActive = timer.status === "running";
  const isIdle = timer.status === "idle" || timer.status === "completed";

  if (isIdle) return null;

  const displaySecs = countUp
    ? timer.total_secs - timer.remaining_secs
    : timer.remaining_secs;

  return (
    <div
      className={cn(
        "flex items-center gap-3 px-4 rounded-md border transition-colors",
        isActive
          ? "bg-tomato-dim border-tomato/20"
          : "bg-bg-layer border-border-subtle",
        className,
      )}
    >
      <span className="relative flex h-3 w-3 shrink-0">
        {isActive && (
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-tomato opacity-50" />
        )}
        <span
          className={cn(
            "relative inline-flex rounded-full h-3 w-3",
            isActive ? "bg-tomato" : "bg-text-muted",
          )}
        />
      </span>
      <span
        className={cn(
          "font-mono font-bold tabular-nums text-3xl tracking-tight",
          isActive ? "text-tomato" : "text-text-muted",
        )}
      >
        {countUp && "+"}
        {formatTime(displaySecs)}
      </span>
    </div>
  );
}
