import { useState, useEffect, useCallback } from "react";
import { api, onEvent } from "@/lib/tauri";

export function useDnd() {
  const [dndActive, setDndActive] = useState(false);

  useEffect(() => {
    api.getDndStatus().then(setDndActive);
    const unlisten = onEvent<boolean>("dnd-changed", setDndActive);
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const toggle = useCallback(async () => {
    const newState = await api.toggleDnd();
    setDndActive(newState);
  }, []);

  return { dndActive, toggle };
}
