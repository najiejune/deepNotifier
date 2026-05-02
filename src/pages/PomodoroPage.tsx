import { useState, useEffect } from "react";
import { Pause, Play, Square, Plus } from "lucide-react";
import { useTimer } from "@/hooks/useTimer";
import { useTodos } from "@/hooks/useTodos";
import { usePomodoroCounts } from "@/hooks/usePomodoroCounts";
import { useTodoTimerConfigs } from "@/hooks/useTodoTimerConfigs";
import { useConfig } from "@/hooks/useConfig";
import { useI18n } from "@/i18n";
import { onEvent } from "@/lib/tauri";
import { TodoList } from "@/components/timer/TodoList";
import { TodoDialog } from "@/components/timer/TodoDialog";
import { Button } from "@/components/ui/Button";
import type { TodoTimerConfig } from "@/types";

type DialogState =
  | { type: "add" }
  | { type: "edit"; taskId: string }
  | null;

function formatTime(secs: number) {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

export function PomodoroPage() {
  const t = useI18n();
  const timer = useTimer();
  const { todos, add, toggle, remove, filterByDate } = useTodos();
  const pomodoroCounts = usePomodoroCounts();
  const { config: globalConfig } = useConfig();
  const { setConfig, getConfig } = useTodoTimerConfigs();

  const [dateFilter, setDateFilter] = useState<"all" | "today" | "week">("all");
  const [currentTaskId, setCurrentTaskId] = useState<string | undefined>();
  const [dialog, setDialog] = useState<DialogState>(null);

  useEffect(() => {
    const unlisten = onEvent<void>("timer-completed", () => {
      if (currentTaskId) {
        pomodoroCounts.increment(currentTaskId);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [currentTaskId, pomodoroCounts]);

  const filtered = filterByDate(todos, dateFilter);

  const currentTodo = currentTaskId
    ? todos.find((td) => td.id === currentTaskId)
    : undefined;

  const defaultConfig: TodoTimerConfig = {
    workMins: globalConfig.timer.pomodoro_work_mins,
    shortBreakMins: globalConfig.timer.pomodoro_short_break_mins,
    longBreakMins: globalConfig.timer.pomodoro_long_break_mins,
    rounds: globalConfig.timer.pomodoro_rounds,
  };

  const handleDialogSave = async (
    text: string,
    dueDate: string,
    timerCfg: TodoTimerConfig,
  ) => {
    if (dialog?.type === "edit") {
      // Update only the per-task timer config; todo text can't be edited (text area disabled in edit mode)
      setConfig(dialog.taskId, timerCfg);
      setDialog(null);
    } else {
      const item = await add(text, dueDate);
      if (item) {
        setConfig(item.id, timerCfg);
      }
      setDialog(null);
    }
  };

  const handleStartPomodoro = (todoId: string) => {
    setCurrentTaskId(todoId);
    const tc = getConfig(todoId);
    if (tc) {
      timer.startPomodoro({
        work_mins: tc.workMins,
        short_break_mins: tc.shortBreakMins,
        long_break_mins: tc.longBreakMins,
        rounds: tc.rounds,
      });
    } else {
      timer.startPomodoro();
    }
  };

  const handlePauseResume = async () => {
    if (timer.timer.status === "running") {
      await timer.pause();
    } else if (timer.timer.status === "paused") {
      await timer.resume();
    }
  };

  const handleStop = async () => {
    await timer.stop();
    setCurrentTaskId(undefined);
  };

  const isRunning = timer.timer.status === "running";
  const isPaused = timer.timer.status === "paused";

  const editTarget = dialog?.type === "edit"
    ? todos.find((td) => td.id === dialog.taskId)
    : undefined;

  return (
    <div className="p-5 space-y-4 h-full flex flex-col">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold tracking-tight text-text">
            {t.pomodoro.title}
          </h1>
          <p className="text-xs text-text-muted font-mono mt-0.5">
            {t.pomodoro.subtitle}
          </p>
        </div>
        <Button size="sm" variant="primary" onClick={() => setDialog({ type: "add" })}>
          <Plus size={14} /> {t.pomodoro.addPomodoro}
        </Button>
      </header>

      {/* Current task + pomodoro bar */}
      {currentTaskId && timer.timer.status !== "idle" && (
        <div className="bg-accent-dim border border-accent/20 rounded-sm px-3 py-1.5 flex items-center gap-3">
          <span className="relative flex h-1.5 w-1.5 shrink-0">
            {isRunning && (
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-tomato opacity-50" />
            )}
            <span className="relative inline-flex rounded-full h-1.5 w-1.5 bg-tomato" />
          </span>
          <span className="text-base text-accent font-medium">
            {t.pomodoro.currentTask}:{" "}
            <span className="font-semibold">
              {currentTodo?.text ?? t.pomodoro.none}
            </span>
          </span>
          <span className="font-mono font-bold tabular-nums text-base text-tomato tracking-tight">
            {formatTime(timer.timer.remaining_secs)}
          </span>

          <button onClick={handlePauseResume}
            className="p-1 rounded-sm text-text-muted hover:text-accent hover:bg-accent-dim transition-colors"
            title={isPaused ? t.pomodoro.resume : t.pomodoro.pause}
          >
            {isPaused ? <Play size={14} /> : <Pause size={14} />}
          </button>

          <button onClick={handleStop}
            className="p-1 rounded-sm text-text-muted hover:text-tomato hover:bg-tomato-dim transition-colors"
            title={t.pomodoro.stop}
          >
            <Square size={14} />
          </button>
        </div>
      )}

      {/* Todo list */}
      <div className="flex-1 overflow-auto">
        <TodoList
          todos={filtered}
          filter={dateFilter}
          onFilterChange={setDateFilter}
          onAdd={add}
          onToggle={toggle}
          onRemove={remove}
          currentTaskId={currentTaskId}
          onStart={handleStartPomodoro}
          onEditTask={(id) => setDialog({ type: "edit", taskId: id })}
          getPomodoroCount={(id) => pomodoroCounts.get(id)}
        />
      </div>

      {dialog && (
        <TodoDialog
          initial={
            dialog.type === "edit" && editTarget
              ? { ...editTarget, timerConfig: getConfig(dialog.taskId) }
              : undefined
          }
          defaultConfig={defaultConfig}
          onSave={handleDialogSave}
          onCancel={() => setDialog(null)}
        />
      )}
    </div>
  );
}
