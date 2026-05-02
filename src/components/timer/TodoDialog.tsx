import { useState, useRef } from "react";
import { Calendar, X } from "lucide-react";
import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";
import { useI18n } from "@/i18n";
import type { TodoItem, TodoTimerConfig } from "@/types";

interface Props {
  initial?: TodoItem & { timerConfig?: TodoTimerConfig };
  defaultConfig: TodoTimerConfig;
  onSave: (
    text: string,
    dueDate: string,
    timerConfig: TodoTimerConfig,
  ) => void;
  onCancel: () => void;
}

export function TodoDialog({
  initial,
  defaultConfig,
  onSave,
  onCancel,
}: Props) {
  const t = useI18n();
  const isEdit = !!initial;
  const dateInputRef = useRef<HTMLInputElement>(null);

  const [text, setText] = useState(initial?.text ?? "");
  const [dueDate, setDueDate] = useState(initial?.due_date ?? "");
  const [workMins, setWorkMins] = useState(
    initial?.timerConfig?.workMins ?? defaultConfig.workMins,
  );
  const [shortBreakMins, setShortBreakMins] = useState(
    initial?.timerConfig?.shortBreakMins ?? defaultConfig.shortBreakMins,
  );
  const [longBreakMins, setLongBreakMins] = useState(
    initial?.timerConfig?.longBreakMins ?? defaultConfig.longBreakMins,
  );
  const [rounds, setRounds] = useState(
    initial?.timerConfig?.rounds ?? defaultConfig.rounds,
  );

  const handleSave = () => {
    if (!text.trim()) return;
    onSave(text.trim(), dueDate, {
      workMins,
      shortBreakMins,
      longBreakMins,
      rounds,
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={onCancel} />
      <div className="relative bg-bg-base border border-border-default rounded-lg shadow-lg w-[460px] p-5 space-y-4">
        <p className="text-sm font-semibold">
          {isEdit ? t.pomodoro.editTask : t.pomodoro.addPomodoro}
        </p>

        <div className="space-y-3">
          <Input
            placeholder={t.pomodoro.todoPlaceholder}
            value={text}
            onChange={(e) => setText(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSave()}
            autoFocus
          />

          {/* Due date */}
          <div className="relative flex items-center">
            <input
              ref={dateInputRef}
              type="date"
              value={dueDate}
              onChange={(e) => setDueDate(e.target.value)}
              className="absolute inset-0 opacity-0 pointer-events-none"
            />
            <button
              type="button"
              onClick={() => dateInputRef.current?.showPicker()}
              className={`h-8 w-full rounded-sm border px-2 text-xs flex items-center gap-1.5 transition-colors ${
                dueDate
                  ? "bg-white border-border-subtle text-text"
                  : "bg-white border-border-subtle text-text-muted hover:border-border-default"
              }`}
            >
              <Calendar size={12} />
              {dueDate || <span className="text-text-muted">{t.pomodoro.dueDate}</span>}
            </button>
            {dueDate && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setDueDate("");
                }}
                className="ml-1 text-text-muted hover:text-red transition-colors"
              >
                <X size={10} />
              </button>
            )}
          </div>

          {/* Pomodoro timer settings */}
          <div className="border-t border-border-subtle pt-3">
            <p className="text-xs font-medium text-text-secondary mb-2">
              {t.pomodoro.timerSettings}
            </p>
            <div className="grid grid-cols-2 gap-2">
              <div>
                <p className="text-[10px] text-text-muted mb-0.5">
                  {t.pomodoro.workMin}
                </p>
                <Input
                  type="number"
                  value={workMins}
                  onChange={(e) =>
                    setWorkMins(Number(e.target.value) || defaultConfig.workMins)
                  }
                  className="text-xs"
                  min={1}
                />
              </div>
              <div>
                <p className="text-[10px] text-text-muted mb-0.5">
                  {t.pomodoro.shortBreakMin}
                </p>
                <Input
                  type="number"
                  value={shortBreakMins}
                  onChange={(e) =>
                    setShortBreakMins(Number(e.target.value) || defaultConfig.shortBreakMins)
                  }
                  className="text-xs"
                  min={0}
                />
              </div>
              <div>
                <p className="text-[10px] text-text-muted mb-0.5">
                  {t.pomodoro.longBreakMin}
                </p>
                <Input
                  type="number"
                  value={longBreakMins}
                  onChange={(e) =>
                    setLongBreakMins(Number(e.target.value) || defaultConfig.longBreakMins)
                  }
                  className="text-xs"
                  min={0}
                />
              </div>
              <div>
                <p className="text-[10px] text-text-muted mb-0.5">
                  {t.pomodoro.rounds}
                </p>
                <Input
                  type="number"
                  value={rounds}
                  onChange={(e) =>
                    setRounds(Number(e.target.value) || defaultConfig.rounds)
                  }
                  className="text-xs"
                  min={1}
                />
              </div>
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button size="sm" variant="ghost" onClick={onCancel}>
            {t.pomodoro.cancel}
          </Button>
          <Button size="sm" onClick={handleSave}>
            {t.pomodoro.save}
          </Button>
        </div>
      </div>
    </div>
  );
}
