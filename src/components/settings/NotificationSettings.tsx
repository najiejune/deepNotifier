import { useState, useEffect, useRef } from "react";
import { Input } from "@/components/ui/Input";
import { Toggle } from "@/components/ui/Toggle";
import { Button } from "@/components/ui/Button";
import { useI18n } from "@/i18n";
import { api } from "@/lib/tauri";
import { Upload, Volume2, Bell } from "lucide-react";
import type { NotificationConfig } from "@/types";
import { open } from "@tauri-apps/plugin-dialog";

interface Props {
  config: NotificationConfig;
  onChange: (config: NotificationConfig) => void;
}

export function NotificationSettings({ config, onChange }: Props) {
  const t = useI18n();
  const [sounds, setSounds] = useState<string[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    api.listSounds().then(setSounds).catch(() => setSounds(["ping", "chime", "bell", "alarm"]));
  }, []);

  const handleImport = async () => {
    try {
      const selected = await open({
        filters: [{ name: "Audio", extensions: ["wav", "mp3", "ogg", "flac"] }],
      });
      if (selected && typeof selected === "string") {
        const name = await api.importSound(selected);
        setSounds((prev) => {
          if (!prev.includes(name)) return [...prev, name];
          return prev;
        });
        onChange({ ...config, sound_file: name });
      }
    } catch {
      // Fallback to HTML file input if dialog plugin unavailable
      fileInputRef.current?.click();
    }
  };

  const handleFileInput = async (e: React.ChangeEvent<HTMLInputElement>) => {
    // HTML file input fallback only provides the file name, not path
    // Dialog plugin is the primary method
    const file = e.target.files?.[0];
    if (file) {
      const name = file.name.replace(/\.[^.]+$/, "");
      setSounds((prev) => {
        if (!prev.includes(name)) return [...prev, name];
        return prev;
      });
      onChange({ ...config, sound_file: name });
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.notifications.sound}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.notifications.soundDesc}</p>
        </div>
        <Toggle checked={config.sound_enabled} onChange={(v) => onChange({ ...config, sound_enabled: v })} />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.notifications.soundFile}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.notifications.soundFileDesc}</p>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={config.sound_file}
            onChange={(e) => onChange({ ...config, sound_file: e.target.value })}
            className="text-[11px] font-mono bg-white border border-border-subtle rounded-sm px-2 py-1.5 text-text-secondary w-28"
          >
            {sounds.map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
            {!sounds.includes(config.sound_file) && config.sound_file && (
              <option value={config.sound_file}>{config.sound_file}</option>
            )}
          </select>
          <Button size="sm" variant="ghost" onClick={() => api.previewSound(config.sound_file)} title="Preview">
            <Volume2 size={12} />
          </Button>
          <Button size="sm" variant="ghost" onClick={handleImport} title="Import audio file">
            <Upload size={12} />
          </Button>
          <input
            ref={fileInputRef}
            type="file"
            accept="audio/*"
            className="hidden"
            onChange={handleFileInput}
          />
        </div>
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.notifications.volume}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.notifications.volumeDesc}</p>
        </div>
        <div className="flex items-center gap-2">
          <input
            type="range" min="0" max="1" step="0.1"
            value={config.sound_volume}
            onChange={(e) => onChange({ ...config, sound_volume: parseFloat(e.target.value) })}
            className="w-24 h-1 accent-accent"
          />
          <span className="text-xs font-mono text-text-muted w-8 text-right">
            {Math.round(config.sound_volume * 100)}%
          </span>
        </div>
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.notifications.marquee}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.notifications.marqueeDesc}</p>
        </div>
        <Toggle checked={config.marquee_enabled} onChange={(v) => onChange({ ...config, marquee_enabled: v })} />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.notifications.systemTray}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.notifications.systemTrayDesc}</p>
        </div>
        <div className="flex items-center gap-2">
          <Button size="sm" variant="ghost" onClick={() => api.previewToast()} title="Preview">
            <Bell size={12} />
          </Button>
          <Toggle checked={config.tray_enabled} onChange={(v) => onChange({ ...config, tray_enabled: v })} />
        </div>
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.notifications.toastInfo}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.notifications.toastInfoDesc}</p>
        </div>
        <Input
          type="number" min={0}
          value={config.toast_info_secs}
          onChange={(e) => onChange({ ...config, toast_info_secs: Math.max(0, Number(e.target.value) || 0) })}
          className="w-24"
        />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.notifications.toastWarning}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.notifications.toastWarningDesc}</p>
        </div>
        <Input
          type="number" min={0}
          value={config.toast_warning_secs}
          onChange={(e) => onChange({ ...config, toast_warning_secs: Math.max(0, Number(e.target.value) || 0) })}
          className="w-24"
        />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.notifications.toastCritical}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.notifications.toastCriticalDesc}</p>
        </div>
        <Input
          type="number" min={0}
          value={config.toast_critical_secs}
          onChange={(e) => onChange({ ...config, toast_critical_secs: Math.max(0, Number(e.target.value) || 0) })}
          className="w-24"
        />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.notifications.maxHistory}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.notifications.maxHistoryDesc}</p>
        </div>
        <Input
          type="number"
          value={config.max_history}
          onChange={(e) => onChange({ ...config, max_history: Number(e.target.value) || 500 })}
          className="w-24"
        />
      </div>
    </div>
  );
}
