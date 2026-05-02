import { useState, useCallback } from "react";
import type { TodoTimerConfig } from "@/types";

const STORAGE_KEY = "deepnotifier_todo_timer_configs";

function loadFromStorage(): Record<string, TodoTimerConfig> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw);
  } catch {}
  return {};
}

function saveToStorage(data: Record<string, TodoTimerConfig>) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
  } catch {}
}

export function useTodoTimerConfigs() {
  const [configs, setConfigs] = useState<Record<string, TodoTimerConfig>>(loadFromStorage);

  const setConfig = useCallback((id: string, config: TodoTimerConfig) => {
    setConfigs((prev) => {
      const next = { ...prev, [id]: config };
      saveToStorage(next);
      return next;
    });
  }, []);

  const removeConfig = useCallback((id: string) => {
    setConfigs((prev) => {
      const next = { ...prev };
      delete next[id];
      saveToStorage(next);
      return next;
    });
  }, []);

  const getConfig = useCallback(
    (id: string): TodoTimerConfig | undefined => configs[id],
    [configs],
  );

  return { configs, setConfig, removeConfig, getConfig };
}
