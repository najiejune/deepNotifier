import { useState, useEffect } from "react";
import { Toggle } from "@/components/ui/Toggle";
import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";
import { useI18n } from "@/i18n";
import { cn } from "@/lib/cn";
import { api } from "@/lib/tauri";
import { Terminal, Volume2 } from "lucide-react";
import { CLI_ICON_MAP } from "./cliIcons";
import type { HookConfig, CliToolConfig, HookInstallResult } from "@/types";

interface Props {
  config: HookConfig;
  onChange: (config: HookConfig) => void;
}

const CLI_VENDORS: Record<string, string> = {
  claude: "Anthropic",
  codex: "OpenAI",
  opencode: "SST",
  gemini: "Google",
  kiro: "Amazon",
  codebuddy: "Tencent",
  qoder: "QoderAI",
  kimi: "Moonshot AI",
};

export function HookSettings({ config, onChange }: Props) {
  const t = useI18n();
  const [results, setResults] = useState<HookInstallResult[] | null>(null);
  const [installing, setInstalling] = useState(false);
  const [cliStatuses, setCliStatuses] = useState<Record<string, boolean>>({});
  const [sounds, setSounds] = useState<string[]>([]);

  useEffect(() => { api.listSounds().then(setSounds).catch(() => {}); }, []);

  const refreshCliStatus = async () => {
    try {
      const statuses = await api.checkCliStatus();
      const map: Record<string, boolean> = {};
      for (const s of statuses) {
        map[s.cli_id] = s.cli_installed;
      }
      setCliStatuses(map);
    } catch { /* ignore */ }
  };

  useEffect(() => {
    if (config.enabled) {
      refreshCliStatus();
    }
  }, [config.enabled]);

  const patchHook = (partial: Partial<HookConfig>) => {
    onChange({ ...config, ...partial });
  };

  const patchTool = (id: string, partial: Partial<CliToolConfig>) => {
    onChange({
      ...config,
      cli_tools: config.cli_tools.map((t) =>
        t.id === id ? { ...t, ...partial } : t,
      ),
    });
  };

  const handleInstall = async () => {
    if (!window.confirm(t.settings.hook.installAllConfirm)) return;
    setInstalling(true);
    setResults(null);
    try {
      const res = await api.installHooks();
      setResults(res);
      const updated = await api.getConfig();
      onChange({ ...updated.hook });
      refreshCliStatus();
    } catch (e) {
      setResults([{ cli_id: "", success: false, message: String(e), config_path: "", events_injected: [] }]);
    } finally {
      setInstalling(false);
    }
  };

  const handleUninstall = async (cliIds?: string[]) => {
    setInstalling(true);
    setResults(null);
    try {
      const res = await api.uninstallHooks(cliIds);
      setResults(res);
      const updated = await api.getConfig();
      onChange({ ...updated.hook });
      refreshCliStatus();
    } catch (e) {
      setResults([{ cli_id: "", success: false, message: String(e), config_path: "", events_injected: [] }]);
    } finally {
      setInstalling(false);
    }
  };

  return (
    <div className="space-y-4">
      {/* Master Enable */}
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.hook.enable}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.hook.enableDesc}</p>
        </div>
        <Toggle
          checked={config.enabled}
          onChange={(v) => onChange({ ...config, enabled: v })}
        />
      </div>

      {config.enabled && (
        <>
          {/* Approval Timeout */}
          <div>
            <p className="text-[10px] font-medium text-text-muted mb-1">{t.settings.hook.approvalTimeout}</p>
            <Input
              type="number"
              value={String(config.approval_timeout_secs)}
              onChange={(e) => onChange({ ...config, approval_timeout_secs: Number(e.target.value) || 120 })}
              className="font-mono text-xs"
            />
            <p className="text-[9px] text-text-muted mt-0.5">{t.settings.hook.approvalTimeoutDesc}</p>
          </div>

          {/* Shared Hook Notification Settings */}
          <div className="space-y-3 border border-border-subtle rounded-sm p-3 bg-white/50">
            <p className="text-[10px] font-mono text-text-muted uppercase tracking-wider">{t.settings.notifications.sound} & {t.settings.hook.marquee}</p>

            {/* Stop Event */}
            <div className="space-y-2">
              <p className="text-[10px] font-medium text-text-muted">{t.settings.hook.events}: {t.settings.hook.stopEvent}</p>
              <div className="flex items-center gap-4 ml-1 flex-wrap">
                <label className="flex items-center gap-1.5 text-xs text-text-secondary cursor-pointer">
                  <Toggle
                    checked={config.on_stop_sound}
                    onChange={(v) => patchHook({ on_stop_sound: v })}
                  />
                  {t.settings.hook.sound}
                </label>
                {config.on_stop_sound && (
                  <div className="flex items-center gap-1">
                    <select
                      value={config.stop_sound_file}
                      onChange={(e) => patchHook({ stop_sound_file: e.target.value })}
                      className="text-[10px] font-mono bg-white border border-border-subtle rounded-sm px-1.5 py-0.5 text-text-secondary w-36"
                    >
                      {sounds.map((s) => <option key={s} value={s}>{s}</option>)}
                      {!sounds.includes(config.stop_sound_file) && (
                        <option value={config.stop_sound_file}>{config.stop_sound_file}</option>
                      )}
                    </select>
                    <Button size="sm" variant="ghost" onClick={() => api.previewSound(config.stop_sound_file)} title="Preview">
                      <Volume2 size={12} />
                    </Button>
                  </div>
                )}
                <label className="flex items-center gap-1.5 text-xs text-text-secondary cursor-pointer">
                  <Toggle
                    checked={config.on_stop_marquee}
                    onChange={(v) => patchHook({ on_stop_marquee: v })}
                  />
                  {t.settings.hook.marquee}
                </label>
              </div>
            </div>

            {/* Notification Event */}
            <div className="space-y-2">
              <p className="text-[10px] font-medium text-text-muted">{t.settings.hook.events}: {t.settings.hook.notificationEvent}</p>
              <div className="flex items-center gap-4 ml-1 flex-wrap">
                <label className="flex items-center gap-1.5 text-xs text-text-secondary cursor-pointer">
                  <Toggle
                    checked={config.on_notification_sound}
                    onChange={(v) => patchHook({ on_notification_sound: v })}
                  />
                  {t.settings.hook.sound}
                </label>
                {config.on_notification_sound && (
                  <div className="flex items-center gap-1">
                    <select
                      value={config.notification_sound_file}
                      onChange={(e) => patchHook({ notification_sound_file: e.target.value })}
                      className="text-[10px] font-mono bg-white border border-border-subtle rounded-sm px-1.5 py-0.5 text-text-secondary w-36"
                    >
                      {sounds.map((s) => <option key={s} value={s}>{s}</option>)}
                      {!sounds.includes(config.notification_sound_file) && (
                        <option value={config.notification_sound_file}>{config.notification_sound_file}</option>
                      )}
                    </select>
                    <Button size="sm" variant="ghost" onClick={() => api.previewSound(config.notification_sound_file)} title="Preview">
                      <Volume2 size={12} />
                    </Button>
                  </div>
                )}
                <label className="flex items-center gap-1.5 text-xs text-text-secondary cursor-pointer">
                  <Toggle
                    checked={config.on_notification_marquee}
                    onChange={(v) => patchHook({ on_notification_marquee: v })}
                  />
                  {t.settings.hook.marquee}
                </label>
              </div>
            </div>

            {/* Approval Timeout */}
            <div className="space-y-2">
              <p className="text-[10px] font-medium text-text-muted">{t.settings.hook.approvalTimeoutSetting}</p>
              <div className="ml-1 space-y-2">
                <label className="flex items-center gap-1.5 text-xs text-text-secondary cursor-pointer">
                  <Toggle
                    checked={config.approval_timeout_enabled}
                    onChange={(v) => patchHook({ approval_timeout_enabled: v })}
                  />
                  {t.settings.hook.approvalTimeoutDesc}
                </label>
                <div className="flex items-center gap-4 flex-wrap">
                  <label className="flex items-center gap-1.5 text-xs text-text-secondary cursor-pointer">
                    <Toggle
                      checked={config.approval_timeout_sound_enabled}
                      onChange={(v) => patchHook({ approval_timeout_sound_enabled: v })}
                    />
                    {t.settings.hook.sound}
                  </label>
                  {config.approval_timeout_sound_enabled && (
                    <div className="flex items-center gap-1">
                      <select
                        value={config.approval_timeout_sound_file}
                        onChange={(e) => patchHook({ approval_timeout_sound_file: e.target.value })}
                        className="text-[10px] font-mono bg-white border border-border-subtle rounded-sm px-1.5 py-0.5 text-text-secondary w-36"
                      >
                        {sounds.map((s) => <option key={s} value={s}>{s}</option>)}
                        {!sounds.includes(config.approval_timeout_sound_file) && (
                          <option value={config.approval_timeout_sound_file}>{config.approval_timeout_sound_file}</option>
                        )}
                      </select>
                      <Button size="sm" variant="ghost" onClick={() => api.previewSound(config.approval_timeout_sound_file)} title="Preview">
                        <Volume2 size={12} />
                      </Button>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-2">
            <button
              onClick={handleInstall}
              disabled={installing}
              className="px-3 py-1.5 text-xs bg-accent text-white rounded-sm hover:bg-accent/90 disabled:opacity-50"
            >
              {installing ? "..." : t.settings.hook.installAll}
            </button>
            <button
              onClick={() => { if (window.confirm(t.settings.hook.uninstallAllConfirm)) handleUninstall(); }}
              disabled={installing}
              className="px-3 py-1.5 text-xs border border-border-subtle rounded-sm hover:bg-white disabled:opacity-50"
            >
              {t.settings.hook.uninstallAll}
            </button>
          </div>

          {/* Install Results */}
          {results && (
            <div className="space-y-1">
              {results.map((r, i) => (
                <div
                  key={i}
                  className={cn(
                    "text-xs px-2.5 py-1.5 rounded-sm border",
                    r.success ? "border-green-200 bg-green-50 text-green-800" : "border-red-200 bg-red-50 text-red-800",
                  )}
                >
                  <span className="font-medium">{r.cli_id}</span>: {r.message}
                  {r.events_injected.length > 0 && (
                    <span className="text-text-muted"> — {r.events_injected.join(", ")}</span>
                  )}
                </div>
              ))}
            </div>
          )}

          {/* CLI Tools List */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <p className="text-[11px] font-mono text-text-muted uppercase tracking-wider">
                {t.settings.hook.cliTools}
              </p>
              <button
                onClick={refreshCliStatus}
                className="text-[10px] text-text-muted hover:text-text underline underline-offset-2"
              >
                {"↻"}
              </button>
            </div>

            {config.cli_tools.map((tool) => (
              <div
                key={tool.id}
                className="border border-border-subtle rounded-sm bg-white/60 hover:bg-white transition-colors"
              >
                <div className="flex items-center gap-3 px-3 py-2.5">
                  <Toggle
                    checked={tool.enabled}
                    onChange={(v) => patchTool(tool.id, { enabled: v })}
                  />
                  <span className="flex-shrink-0">
                    {(() => {
                      const Icon = CLI_ICON_MAP[tool.id];
                      return Icon ? <Icon size={20} /> : <Terminal size={16} />;
                    })()}
                  </span>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-text truncate">{tool.name}</p>
                    <p className="text-[10px] text-text-muted truncate">
                      {CLI_VENDORS[tool.id] || tool.id}
                    </p>
                  </div>
                  {/* Status badges */}
                  <span className="flex items-center gap-2 text-[10px] font-mono">
                    <span className={cn(
                      "px-1.5 py-0.5 rounded-sm",
                      cliStatuses[tool.id] ? "text-green-600 bg-green-50" : "text-text-muted/50 bg-gray-50",
                    )}>
                      {cliStatuses[tool.id] ? t.settings.hook.cliInstalled : t.settings.hook.cliNotInstalled}
                    </span>
                    <span className={cn(
                      "px-1.5 py-0.5 rounded-sm",
                      tool.install_status === "Installed" ? "text-green-600 bg-green-50"
                      : typeof tool.install_status === "object" ? "text-red-500 bg-red-50"
                      : "text-text-muted/50 bg-gray-50",
                    )}>
                      {tool.install_status === "Installed" ? t.settings.hook.hookInstalled
                      : typeof tool.install_status === "object" ? t.settings.hook.hookNotInstalled
                      : t.settings.hook.hookNotInstalled}
                    </span>
                  </span>
                  {/* Per-tool Install/Uninstall */}
                  <div className="flex gap-1.5">
                    <button
                      onClick={() => {
                        patchTool(tool.id, { enabled: true });
                        api.installHooks([tool.id]).then((res) => {
                          setResults(res);
                          api.getConfig().then((cfg) => onChange({ ...cfg.hook }));
                          refreshCliStatus();
                        }).catch((e) =>
                          setResults([{ cli_id: tool.id, success: false, message: String(e), config_path: "", events_injected: [] }])
                        );
                      }}
                      disabled={installing}
                      className="px-2.5 py-1 text-xs bg-accent text-white rounded-sm hover:bg-accent/90 disabled:opacity-50"
                    >
                      {t.settings.hook.install}
                    </button>
                    <button
                      onClick={() => handleUninstall([tool.id])}
                      disabled={installing}
                      className="px-2.5 py-1 text-xs border border-border-subtle rounded-sm hover:bg-white disabled:opacity-50"
                    >
                      {t.settings.hook.uninstall}
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}
