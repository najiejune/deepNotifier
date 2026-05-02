import { useState, useEffect, useCallback } from "react";
import { api, onEvent } from "@/lib/tauri";
import type { TodoItem } from "@/types";

export type { TodoItem };

export function useTodos() {
  const [todos, setTodos] = useState<TodoItem[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchTodos = useCallback(async () => {
    try {
      const list = await api.getTodos();
      setTodos(list);
    } catch (e) {
      console.warn("Failed to fetch todos:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTodos();
  }, [fetchTodos]);

  // Listen for backend push/pull updates
  useEffect(() => {
    const unlisten = onEvent("todos-updated", () => {
      fetchTodos();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [fetchTodos]);

  const add = useCallback(
    async (text: string, dueDate: string): Promise<TodoItem | undefined> => {
      try {
        const item = await api.addTodo(text, dueDate || undefined);
        setTodos((prev) => [item, ...prev]);
        return item;
      } catch (e) {
        console.warn("Failed to add todo:", e);
        return undefined;
      }
    },
    [],
  );

  const toggle = useCallback(async (id: string) => {
    try {
      await api.toggleTodo(id);
      setTodos((prev) =>
        prev.map((t) => (t.id === id ? { ...t, completed: !t.completed } : t)),
      );
    } catch (e) {
      console.warn("Failed to toggle todo:", e);
    }
  }, []);

  const remove = useCallback(async (id: string) => {
    try {
      await api.deleteTodo(id);
      setTodos((prev) => prev.filter((t) => t.id !== id));
    } catch (e) {
      console.warn("Failed to delete todo:", e);
    }
  }, []);

  const filterByDate = useCallback(
    (items: TodoItem[], filter: "all" | "today" | "week") => {
      if (filter === "all") return items;
      const now = new Date();
      const startOfToday = new Date(
        now.getFullYear(),
        now.getMonth(),
        now.getDate(),
      );
      if (filter === "today") {
        const endOfToday = new Date(startOfToday.getTime() + 86400000);
        return items.filter((t) => {
          if (!t.due_date) return true;
          const d = new Date(t.due_date);
          return d >= startOfToday && d < endOfToday;
        });
      }
      if (filter === "week") {
        const endOfWeek = new Date(
          startOfToday.getTime() + 7 * 86400000,
        );
        return items.filter((t) => {
          if (!t.due_date) return true;
          const d = new Date(t.due_date);
          return d >= startOfToday && d < endOfWeek;
        });
      }
      return items;
    },
    [],
  );

  return { todos, loading, add, toggle, remove, filterByDate, refetch: fetchTodos };
}
