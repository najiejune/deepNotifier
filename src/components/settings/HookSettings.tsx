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

/** Sound file select + preview button, shared by the event cards. */
function SoundPicker({
  value,
  sounds,
  onChange,
}: {
  value: string;
  sounds: string[];
  onChange: (v: string) => void;
}) {
  return (
    <div className="flex items-center gap-1">
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="text-[11px] font-mono bg-white border border-border-subtle rounded-sm px-2 py-1.5 text-text-secondary w-32"
      >
        {sounds.map((s) => <option key={s} value={s}>{s}</option>)}
        {!sounds.includes(value) && (
          <option value={value}>{value}</option>
        )}
      </select>
      <Button size="sm" variant="ghost" onClick={() => api.previewSound(value)} title="Preview">
        <Volume2 size={12} />
      </Button>
    </div>
  );
}

/** One hook event: a title row stating when it fires, plus its option rows. */
function EventGroup({
  title,
  desc,
  children,
}: {
  title: string;
  desc: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-2">
      <div>
        <p className="text-sm font-medium">{title}</p>
        <p className="text-xs text-text-muted mt-0.5">{desc}</p>
      </div>
      <div className="ml-4 space-y-2">{children}</div>
    </div>
  );
}

/** One option row inside an EventGroup: label left, control right. */
function OptionRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between">
      <p className="text-sm text-text-secondary">{label}</p>
      <div className="flex items-center gap-2">{children}</div>
    </div>
  );
}

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
          <p className="text-sm font-medium text-text">{t.settings.hook.enable}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.hook.enableDesc}</p>
        </div>
        <Toggle
          checked={config.enabled}
          onChange={(v) => onChange({ ...config, enabled: v })}
        />
      </div>

      {config.enabled && (
        <>
          <div className="border-t border-border-subtle" />

          {/* Per-event notification options */}
          <div className="space-y-4">
            <p className="text-base font-medium text-text">
              {t.settings.hook.eventNotifications}
            </p>

            {/* Stop Event */}
            <EventGroup
              title={t.settings.hook.stopEvent}
              desc={t.settings.hook.stopEventDesc}
            >
              <OptionRow label={t.settings.hook.sound}>
                {config.on_stop_sound && (
                  <SoundPicker
                    value={config.stop_sound_file}
                    sounds={sounds}
                    onChange={(v) => patchHook({ stop_sound_file: v })}
                  />
                )}
                <Toggle
                  checked={config.on_stop_sound}
                  onChange={(v) => patchHook({ on_stop_sound: v })}
                />
              </OptionRow>
              <OptionRow label={t.settings.hook.marquee}>
                <Toggle
                  checked={config.on_stop_marquee}
                  onChange={(v) => patchHook({ on_stop_marquee: v })}
                />
              </OptionRow>
            </EventGroup>

            {/* Notification Event */}
            <EventGroup
              title={t.settings.hook.notificationEvent}
              desc={t.settings.hook.notificationEventDesc}
            >
              <OptionRow label={t.settings.hook.sound}>
                {config.on_notification_sound && (
                  <SoundPicker
                    value={config.notification_sound_file}
                    sounds={sounds}
                    onChange={(v) => patchHook({ notification_sound_file: v })}
                  />
                )}
                <Toggle
                  checked={config.on_notification_sound}
                  onChange={(v) => patchHook({ on_notification_sound: v })}
                />
              </OptionRow>
              <OptionRow label={t.settings.hook.marquee}>
                <Toggle
                  checked={config.on_notification_marquee}
                  onChange={(v) => patchHook({ on_notification_marquee: v })}
                />
              </OptionRow>
            </EventGroup>

          </div>

          <div className="border-t border-border-subtle" />

          {/* Approval Timeout Alert */}
          <EventGroup
            title={t.settings.hook.approvalTimeoutSetting}
            desc={t.settings.hook.approvalTimeoutDesc}
          >
            <OptionRow label={t.settings.hook.approvalTimeout}>
              <Input
                type="number"
                min={0}
                value={config.approval_timeout_secs}
                onChange={(e) => onChange({ ...config, approval_timeout_secs: Math.max(0, Number(e.target.value) || 120) })}
                className="w-24"
              />
            </OptionRow>
            <OptionRow label={t.settings.hook.approvalTimeoutEnable}>
              <Toggle
                checked={config.approval_timeout_enabled}
                onChange={(v) => patchHook({ approval_timeout_enabled: v })}
              />
            </OptionRow>
            {config.approval_timeout_enabled && (
              <OptionRow label={t.settings.hook.sound}>
                {config.approval_timeout_sound_enabled && (
                  <SoundPicker
                    value={config.approval_timeout_sound_file}
                    sounds={sounds}
                    onChange={(v) => patchHook({ approval_timeout_sound_file: v })}
                  />
                )}
                <Toggle
                  checked={config.approval_timeout_sound_enabled}
                  onChange={(v) => patchHook({ approval_timeout_sound_enabled: v })}
                />
              </OptionRow>
            )}
          </EventGroup>

          <div className="border-t border-border-subtle" />

          {/* CLI Tools List */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <p className="text-base font-medium text-text">
                {t.settings.hook.cliTools}
              </p>
              <div className="flex items-center gap-2">
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
                <button
                  onClick={refreshCliStatus}
                  className="text-[10px] text-text-muted hover:text-text underline underline-offset-2"
                >
                  {"↻"}
                </button>
              </div>
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
                  <span className="flex items-center gap-2 text-[10px]">
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
