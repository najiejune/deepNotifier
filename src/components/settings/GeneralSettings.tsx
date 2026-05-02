import { Select } from "@/components/ui/Select";
import { Toggle } from "@/components/ui/Toggle";
import { useI18n } from "@/i18n";
import type { GeneralConfig } from "@/types";

interface Props {
  config: GeneralConfig;
  onChange: (config: GeneralConfig) => void;
}

export function GeneralSettings({ config, onChange }: Props) {
  const t = useI18n();

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.general.language}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.general.languageDesc}</p>
        </div>
        <Select
          value={config.language}
          onChange={(e) => onChange({ ...config, language: e.target.value })}
          options={[
            { value: "en", label: "English" },
            { value: "zh", label: "中文" },
          ]}
          className="w-28"
        />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.general.notificationMode}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.general.notificationModeDesc}</p>
        </div>
        <Select
          value={config.mode}
          onChange={(e) => onChange({ ...config, mode: e.target.value as GeneralConfig["mode"] })}
          options={[
            { value: "Push", label: t.modes.push },
            { value: "Pull", label: t.modes.pull },
            { value: "Both", label: t.modes.both },
          ]}
          className="w-28"
        />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.general.runOnStartup}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.general.runOnStartupDesc}</p>
        </div>
        <Toggle checked={config.run_on_startup} onChange={(v) => onChange({ ...config, run_on_startup: v })} />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.general.minimizeToTray}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.general.minimizeToTrayDesc}</p>
        </div>
        <Toggle checked={config.minimize_to_tray} onChange={(v) => onChange({ ...config, minimize_to_tray: v })} />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.general.closeToTray}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.general.closeToTrayDesc}</p>
        </div>
        <Toggle checked={config.close_to_tray} onChange={(v) => onChange({ ...config, close_to_tray: v })} />
      </div>
    </div>
  );
}
