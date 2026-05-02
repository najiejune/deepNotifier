import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppConfig, NotificationEvent, TimerState, TodoItem } from "@/types";

export const api = {
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (config: AppConfig) => invoke<void>("save_config", { config }),
  resetConfig: () => invoke<AppConfig>("reset_config"),

  getNotifications: (limit?: number) =>
    invoke<NotificationEvent[]>("get_notifications", { limit }),
  clearNotifications: () => invoke<void>("clear_notifications"),

  stopTimer: () => invoke<void>("stop_timer"),
  pauseTimer: () => invoke<void>("pause_timer"),
  getTimerState: () => invoke<TimerState>("get_timer_state"),
  startPomodoro: (opts?: {
    work_mins?: number;
    short_break_mins?: number;
    long_break_mins?: number;
    rounds?: number;
  }) =>
    invoke<void>("start_pomodoro", {
      workMins: opts?.work_mins ?? null,
      shortBreakMins: opts?.short_break_mins ?? null,
      longBreakMins: opts?.long_break_mins ?? null,
      rounds: opts?.rounds ?? null,
    }),

  toggleDnd: () => invoke<boolean>("toggle_dnd"),
  getDndStatus: () => invoke<boolean>("get_dnd_status"),

  showMarquee: (text: string) => invoke<void>("show_marquee", { text }),
  hideMarquee: () => invoke<void>("hide_marquee"),

  getTodos: () => invoke<TodoItem[]>("get_todos"),
  addTodo: (text: string, dueDate?: string) =>
    invoke<TodoItem>("add_todo", { text, dueDate }),
  toggleTodo: (id: string) => invoke<void>("toggle_todo", { id }),
  deleteTodo: (id: string) => invoke<void>("delete_todo", { id }),

  getHostIp: () => invoke<string>("get_host_ip"),
  getWanIp: () => invoke<string>("get_wan_ip"),

  listSounds: () => invoke<string[]>("list_sounds"),
  importSound: (path: string) => invoke<string>("import_sound", { path }),
  previewSound: (soundFile: string) => invoke<void>("preview_sound", { soundFile }),
};

export function onEvent<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(event, (e) => handler(e.payload));
}
