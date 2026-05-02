import { useState, useEffect, useCallback } from "react";
import { api, onEvent } from "@/lib/tauri";
import type { NotificationEvent } from "@/types";

export function useNotifications(limit = 100) {
  const [notifications, setNotifications] = useState<NotificationEvent[]>([]);

  const refresh = useCallback(async () => {
    const items = await api.getNotifications(limit);
    setNotifications(items);
  }, [limit]);

  const clear = useCallback(async () => {
    await api.clearNotifications();
    setNotifications([]);
  }, []);

  useEffect(() => {
    refresh();
    const unlisten = onEvent<NotificationEvent>("notification", (event) => {
      setNotifications((prev) => [event, ...prev].slice(0, limit));
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [refresh, limit]);

  return { notifications, refresh, clear };
}
