import { useState, useEffect, useCallback, useMemo, createContext, useContext } from "react";
import { api } from "@/lib/tauri";
import type { AppConfig } from "@/types";

const defaultConfig: AppConfig = {
  general: { language: "en", mode: "Push", run_on_startup: false, minimize_to_tray: true, close_to_tray: true },
  webhook: { enabled: true, port: 3927, secret: "", github_events: [], gitlab_events: [], bitbucket_events: ["repo:push", "pullrequest:created", "pullrequest:updated", "pullrequest:approved", "pullrequest:merged", "repo:refs_changed", "pr:opened", "pr:modified", "pr:reviewer_approved", "pr:merged", "pr:declined"], custom_enabled: false, custom_title_path: "title", custom_body_path: "body", custom_severity: "Info" },
  poll: { enabled: false, endpoints: [] },
  notification: { sound_enabled: true, sound_file: "ping", sound_volume: 0.7, marquee_enabled: true, tray_enabled: true, max_history: 500 },
  dnd: { enabled: false, schedules: [] },
  timer: { pomodoro_work_mins: 25, pomodoro_short_break_mins: 1, pomodoro_long_break_mins: 0, pomodoro_rounds: 4, pomodoro_sound_file: "chime", auto_start_break: false, auto_start_work: false },
  marquee: { position: "Top", speed: 80, height: 40, font_size: 16, font_family: "sans-serif", icon_before: "", icon_after: "", bg_color: "#1e3a5f", text_color: "#ffffff", opacity: 0.9, duration_secs: 10 },
  todo: { pull_enabled: false, pull_endpoints: [], push_enabled: false, push_port: 3928 },
};

interface ConfigCtx {
  config: AppConfig;
  loading: boolean;
  save: (c: AppConfig) => Promise<void>;
  update: <K extends keyof AppConfig>(section: K, value: AppConfig[K]) => void;
  reset: () => Promise<void>;
}

const ConfigContext = createContext<ConfigCtx>({
  config: defaultConfig,
  loading: true,
  save: async () => {},
  update: () => {},
  reset: async () => {},
});

export function ConfigProvider({ children }: { children: React.ReactNode }) {
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getConfig().then(setConfig).finally(() => setLoading(false));
  }, []);

  const save = useCallback(async (c: AppConfig) => {
    await api.saveConfig(c);
    setConfig(c);
  }, []);

  const update = useCallback(
    <K extends keyof AppConfig>(section: K, value: AppConfig[K]) => {
      setConfig((prev) => {
        const next = { ...prev, [section]: value };
        api.saveConfig(next);
        return next;
      });
    },
    [],
  );

  const reset = useCallback(async () => {
    const c = await api.resetConfig();
    setConfig(c);
  }, []);

  const ctx = useMemo<ConfigCtx>(
    () => ({ config, loading, save, update, reset }),
    [config, loading, save, update, reset],
  );

  return (
    <ConfigContext.Provider value={ctx}>
      {children}
    </ConfigContext.Provider>
  );
}

export function useConfig() {
  return useContext(ConfigContext);
}
