import { useState, useEffect } from "react";
import { Input } from "@/components/ui/Input";
import { Toggle } from "@/components/ui/Toggle";
import { Button } from "@/components/ui/Button";
import { useI18n } from "@/i18n";
import { api } from "@/lib/tauri";
import { Volume2 } from "lucide-react";
import type { TimerConfig } from "@/types";

interface Props {
  config: TimerConfig;
  onChange: (config: TimerConfig) => void;
}

export function PomodoroSettings({ config, onChange }: Props) {
  const t = useI18n();
  const [sounds, setSounds] = useState<string[]>(["chime", "ping", "bell", "alarm"]);

  useEffect(() => {
    api.listSounds().then(setSounds).catch(() => {});
  }, []);

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-4">
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.pomodoro.workMin}</p>
          <Input
            type="number"
            value={config.pomodoro_work_mins}
            onChange={(e) => onChange({ ...config, pomodoro_work_mins: Number(e.target.value) || 25 })}
          />
        </div>
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.pomodoro.shortBreakMin}</p>
          <Input
            type="number"
            value={config.pomodoro_short_break_mins}
            onChange={(e) => onChange({ ...config, pomodoro_short_break_mins: Number(e.target.value) || 1 })}
          />
        </div>
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.pomodoro.longBreakMin}</p>
          <Input
            type="number"
            value={config.pomodoro_long_break_mins}
            onChange={(e) => onChange({ ...config, pomodoro_long_break_mins: Number(e.target.value) || 0 })}
          />
        </div>
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.pomodoro.rounds}</p>
          <Input
            type="number"
            value={config.pomodoro_rounds}
            onChange={(e) => onChange({ ...config, pomodoro_rounds: Number(e.target.value) || 4 })}
          />
        </div>
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.pomodoro.sound}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.pomodoro.soundDesc}</p>
        </div>
        <div className="flex items-center gap-1">
          <select
            value={config.pomodoro_sound_file}
            onChange={(e) => onChange({ ...config, pomodoro_sound_file: e.target.value })}
            className="text-[11px] font-mono bg-white border border-border-subtle rounded-sm px-2 py-1.5 text-text-secondary w-28"
          >
            {sounds.map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
          <Button size="sm" variant="ghost" onClick={() => api.previewSound(config.pomodoro_sound_file)} title="Preview">
            <Volume2 size={12} />
          </Button>
        </div>
      </div>

      <div className="border-t border-border-subtle pt-4 space-y-3">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium">{t.settings.pomodoro.autoStartBreak}</p>
            <p className="text-xs text-text-muted mt-0.5">{t.settings.pomodoro.autoStartBreakDesc}</p>
          </div>
          <Toggle checked={config.auto_start_break} onChange={(v) => onChange({ ...config, auto_start_break: v })} />
        </div>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium">{t.settings.pomodoro.autoStartWork}</p>
            <p className="text-xs text-text-muted mt-0.5">{t.settings.pomodoro.autoStartWorkDesc}</p>
          </div>
          <Toggle checked={config.auto_start_work} onChange={(v) => onChange({ ...config, auto_start_work: v })} />
        </div>
      </div>
    </div>
  );
}
