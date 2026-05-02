import { useState, useCallback, useEffect } from "react";

const STORAGE_KEY = "deepnotifier-pomodoro-counts";

function load(): Record<string, number> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function save(counts: Record<string, number>) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(counts));
}

export function usePomodoroCounts() {
  const [counts, setCounts] = useState<Record<string, number>>(load);

  useEffect(() => {
    save(counts);
  }, [counts]);

  const increment = useCallback((todoId: string) => {
    setCounts((prev) => ({
      ...prev,
      [todoId]: (prev[todoId] || 0) + 1,
    }));
  }, []);

  const get = useCallback(
    (todoId: string): number => counts[todoId] || 0,
    [counts],
  );

  return { counts, increment, get };
}
