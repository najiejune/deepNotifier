import { useState, useRef } from "react";
import { Play, Pencil, Trash2, Check, Circle, Calendar, X } from "lucide-react";
import { cn } from "@/lib/cn";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { useI18n } from "@/i18n";
import type { TodoItem } from "@/hooks/useTodos";

interface Props {
  todos: TodoItem[];
  filter: "all" | "today" | "week";
  onFilterChange: (f: "all" | "today" | "week") => void;
  onAdd: (text: string, dueDate: string) => void;
  onToggle: (id: string) => void;
  onRemove: (id: string) => void;
  currentTaskId?: string;
  onStart: (id: string) => void;
  onEditTask: (id: string) => void;
  getPomodoroCount?: (id: string) => number;
}

export function TodoList({
  todos,
  filter,
  onFilterChange,
  onAdd,
  onToggle,
  onRemove,
  currentTaskId,
  onStart,
  onEditTask,
  getPomodoroCount,
}: Props) {
  const t = useI18n();
  const [newText, setNewText] = useState("");
  const [dueDate, setDueDate] = useState("");
  const dateInputRef = useRef<HTMLInputElement>(null);

  const handleAdd = () => {
    if (!newText.trim()) return;
    onAdd(newText.trim(), dueDate);
    setNewText("");
    setDueDate("");
  };

  const filters: { key: "all" | "today" | "week"; label: string }[] = [
    { key: "all", label: t.pomodoro.filterAll },
    { key: "today", label: t.pomodoro.filterToday },
    { key: "week", label: t.pomodoro.filterWeek },
  ];

  return (
    <div className="flex flex-col h-full space-y-3">
      {/* Inline add bar */}
      <div className="flex items-center gap-2">
        <Input
          value={newText}
          onChange={(e) => setNewText(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleAdd()}
          placeholder={t.pomodoro.todoPlaceholder}
          className="flex-1 text-xs"
        />
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
            className={`h-8 w-[140px] rounded-sm border px-2 text-xs flex items-center gap-1.5 transition-colors ${
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
        <Button size="sm" variant="primary" onClick={handleAdd}>
          {t.pomodoro.addTodo}
        </Button>
      </div>

      {/* Filter tabs */}
      <div className="flex gap-1">
        {filters.map(({ key, label }) => (
          <button
            key={key}
            onClick={() => onFilterChange(key)}
            className={cn(
              "px-2 py-0.5 rounded-sm text-[11px] font-medium transition-colors",
              filter === key
                ? "bg-accent-dim text-accent"
                : "text-text-muted hover:text-text-secondary hover:bg-bg-layer",
            )}
          >
            {label}
          </button>
        ))}
      </div>

      {/* Todo list */}
      {todos.length === 0 ? (
        <p className="text-xs text-text-muted py-4 text-center">
          {t.pomodoro.noTodos}
        </p>
      ) : (
        <div className="space-y-1 flex-1 min-h-0 overflow-auto">
          {todos.map((todo) => {
            const isCurrent = currentTaskId === todo.id;
            return (
              <div
                key={todo.id}
                className={cn(
                  "flex items-center gap-2 px-2 py-1.5 rounded-sm border transition-all duration-150 group",
                  todo.completed
                    ? "bg-bg-layer border-border-subtle opacity-60"
                    : isCurrent
                      ? "bg-accent-dim border-accent/30"
                      : "bg-white border-border-subtle hover:border-border-default",
                )}
              >
                <button
                  onClick={() => onToggle(todo.id)}
                  className={cn(
                    "shrink-0 transition-colors",
                    todo.completed
                      ? "text-cyan"
                      : "text-text-muted hover:text-accent",
                  )}
                >
                  {todo.completed ? (
                    <Check size={14} />
                  ) : (
                    <Circle size={14} />
                  )}
                </button>

                <span
                  className={cn(
                    "flex-1 text-xs truncate",
                    todo.completed && "line-through text-text-muted",
                  )}
                >
                  {todo.text}
                </span>

                {todo.due_date && (
                  <span className="text-[10px] text-text-muted font-mono shrink-0">
                    {todo.due_date}
                  </span>
                )}

                {isCurrent && (
                  <span className="text-[10px] font-medium text-accent bg-accent/10 px-1 py-0.5 rounded-sm shrink-0">
                    {t.pomodoro.currentTask}
                  </span>
                )}

                {getPomodoroCount && getPomodoroCount(todo.id) > 0 && (
                  <span className="text-[10px] text-tomato font-medium shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                    {t.pomodoro.pomodoroCount} {getPomodoroCount(todo.id)}
                  </span>
                )}

                {!todo.completed && (
                  <div className="flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onEditTask(todo.id);
                      }}
                      className="p-1 text-text-muted hover:text-text-secondary transition-colors"
                      title={t.pomodoro.edit}
                    >
                      <Pencil size={12} />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onStart(todo.id);
                      }}
                      className="px-2 py-0.5 rounded-sm text-[10px] font-medium bg-accent text-white hover:bg-accent-glow transition-colors"
                    >
                      <Play size={10} className="inline mr-0.5" />
                      {t.pomodoro.start}
                    </button>
                  </div>
                )}

                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onRemove(todo.id);
                  }}
                  className="shrink-0 text-text-muted hover:text-red opacity-0 group-hover:opacity-100 transition-all"
                >
                  <Trash2 size={12} />
                </button>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
