import { useState } from "react";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { Toggle } from "@/components/ui/Toggle";
import { useI18n } from "@/i18n";
import { cn } from "@/lib/cn";
import { api } from "@/lib/tauri";
import type { MarqueeConfig, MarqueePosition } from "@/types";

interface Props {
  config: MarqueeConfig;
  onChange: (config: MarqueeConfig) => void;
}

const FONT_OPTIONS = [
  { value: "sans-serif", label: "System Sans" },
  { value: "'Noto Sans SC', sans-serif", label: "Noto Sans SC" },
  { value: "'Noto Serif SC', Georgia, serif", label: "Noto Serif (Poster)" },
  { value: "Impact, 'SimHei', sans-serif", label: "Impact / 黑体" },
  { value: "'Comic Sans MS', 'YouYuan', cursive", label: "Comic / 幼圆" },
  { value: "'ZCOOL KuaiLe', cursive", label: "ZCOOL KuaiLe" },
  { value: "'Liu Jian Mao Cao', cursive", label: "Liu Jian Mao Cao" },
  { value: "'JetBrains Mono', 'Courier New', monospace", label: "JetBrains Mono" },
  { value: "Georgia, 'KaiTi', serif", label: "Georgia / 楷体" },
];

interface Preset {
  bg_color: string;
  text_color: string;
  font_size: number;
  font_family: string;
  opacity: number;
  icon_before: string;
  icon_after: string;
}

const PRESETS: Record<string, Preset> = {
  poster: {
    bg_color: "#1a1a2e",
    text_color: "#e94560",
    font_size: 22,
    font_family: "Impact, 'SimHei', sans-serif",
    opacity: 0.7,
    icon_before: "⚡",
    icon_after: "⚡",
  },
  anime: {
    bg_color: "#6c3bd6",
    text_color: "#ffd700",
    font_size: 18,
    font_family: "'ZCOOL KuaiLe', cursive",
    opacity: 0.7,
    icon_before: "✨",
    icon_after: "✨",
  },
  professional: {
    bg_color: "#0d1b2a",
    text_color: "#e0e1dd",
    font_size: 17,
    font_family: "'Noto Serif SC', Georgia, serif",
    opacity: 0.7,
    icon_before: "◆",
    icon_after: "◆",
  },
  kawaii: {
    bg_color: "#ffd1dc",
    text_color: "#d63384",
    font_size: 16,
    font_family: "'Comic Sans MS', 'YouYuan', cursive",
    opacity: 0.7,
    icon_before: "🌸",
    icon_after: "🌸",
  },
  transparent: {
    bg_color: "transparent",
    text_color: "#ffffff",
    font_size: 16,
    font_family: "'Noto Sans SC', sans-serif",
    opacity: 0.7,
    icon_before: "",
    icon_after: "",
  },
};

export function MarqueeSettings({ config, onChange }: Props) {
  const t = useI18n();
  const [preview, setPreview] = useState(false);

  const activePreset = Object.entries(PRESETS).find(([, preset]) =>
    (Object.keys(preset) as (keyof Preset)[]).every(
      (k) => config[k] === preset[k],
    ),
  )?.[0] ?? null;

  const applyPreset = (key: string) => {
    onChange({ ...config, ...PRESETS[key] });
  };

  const patch = (partial: Partial<MarqueeConfig>) =>
    onChange({ ...config, ...partial });

  const togglePreview = (on: boolean) => {
    setPreview(on);
    if (on) {
      api.showMarquee(t.settings.marquee.previewText);
    } else {
      api.hideMarquee();
    }
  };

  return (
    <div className="space-y-5">
      {/* Preview */}
      <div className="flex items-center justify-between p-3 rounded-sm border border-border-subtle bg-white/50">
        <div>
          <p className="text-sm font-medium">{t.settings.marquee.preview}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.marquee.previewText}</p>
        </div>
        <Toggle checked={preview} onChange={togglePreview} />
      </div>

      {/* Preset Themes */}
      <div>
        <div className="flex items-center gap-2 mb-2">
          <p className="text-[11px] font-mono text-text-muted uppercase tracking-wider">
            {t.settings.marquee.presetThemes}
          </p>
          {activePreset !== null && (
            <span className="text-[10px] font-medium text-accent bg-accent/10 px-1.5 py-0.5 rounded-sm">
              {t.settings.marquee.presets[activePreset as keyof typeof t.settings.marquee.presets]}
            </span>
          )}
        </div>
        <div className="grid grid-cols-5 gap-2">
          {Object.entries(PRESETS).map(([key, preset]) => (
            <button
              key={key}
              onClick={() => applyPreset(key)}
              className={cn(
                "flex flex-col items-center gap-1.5 p-2 rounded-sm border transition-all duration-150",
                "hover:border-accent/40 hover:shadow-sm",
                activePreset === key
                  ? "border-accent ring-1 ring-accent/20"
                  : "border-border-subtle",
              )}
            >
              <span
                className="w-full h-7 rounded-sm flex items-center justify-center text-[11px] font-medium"
                style={{
                  backgroundColor:
                    preset.bg_color === "transparent" ? "#222" : preset.bg_color,
                  color: preset.text_color,
                  fontFamily: preset.font_family,
                }}
              >
                {(preset.icon_before || "") + "Aa" + (preset.icon_after || "")}
              </span>
              <span className="text-[10px] text-text-muted leading-tight text-center">
                {t.settings.marquee.presets[key as keyof typeof t.settings.marquee.presets]}
              </span>
            </button>
          ))}
        </div>
      </div>

      {/* Position & Duration */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.marquee.position}</p>
          <Select
            value={config.position}
            onChange={(e) => patch({ position: e.target.value as MarqueePosition })}
            options={[
              { value: "Top", label: t.settings.marquee.top },
              { value: "Bottom", label: t.settings.marquee.bottom },
            ]}
          />
        </div>
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.marquee.duration}</p>
          <Input
            type="number"
            value={config.duration_secs}
            onChange={(e) => patch({ duration_secs: Number(e.target.value) || 10 })}
          />
        </div>
      </div>

      {/* Speed & Height */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.marquee.speed}</p>
          <Input
            type="number"
            value={config.speed}
            onChange={(e) => patch({ speed: Number(e.target.value) || 80 })}
          />
        </div>
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.marquee.height}</p>
          <Input
            type="number"
            value={config.height}
            onChange={(e) => patch({ height: Number(e.target.value) || 40 })}
          />
        </div>
      </div>

      {/* Font Family & Font Size */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.marquee.fontFamily}</p>
          <Select
            value={config.font_family}
            onChange={(e) => patch({ font_family: e.target.value })}
            options={FONT_OPTIONS.map((f) => ({ value: f.value, label: f.label }))}
          />
        </div>
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.marquee.fontSize}</p>
          <Input
            type="number"
            value={config.font_size}
            onChange={(e) => patch({ font_size: Number(e.target.value) || 16 })}
          />
        </div>
      </div>

      {/* Icon Before / After */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.marquee.iconBefore}</p>
          <Input
            value={config.icon_before}
            onChange={(e) => patch({ icon_before: e.target.value })}
            placeholder={t.settings.marquee.iconPlaceholder}
          />
        </div>
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.marquee.iconAfter}</p>
          <Input
            value={config.icon_after}
            onChange={(e) => patch({ icon_after: e.target.value })}
            placeholder={t.settings.marquee.iconPlaceholder}
          />
        </div>
      </div>

      {/* Colors */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.marquee.bgColor}</p>
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={config.bg_color === "transparent" ? "#000000" : config.bg_color}
              onChange={(e) => patch({ bg_color: e.target.value })}
              className="w-8 h-8 rounded-sm cursor-pointer border border-border-subtle bg-transparent p-0.5"
            />
            <Input
              value={config.bg_color}
              onChange={(e) => patch({ bg_color: e.target.value })}
              className="flex-1 font-mono"
              placeholder="transparent"
            />
          </div>
        </div>
        <div>
          <p className="text-xs text-text-muted mb-1">{t.settings.marquee.textColor}</p>
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={config.text_color}
              onChange={(e) => patch({ text_color: e.target.value })}
              className="w-8 h-8 rounded-sm cursor-pointer border border-border-subtle bg-transparent p-0.5"
            />
            <Input
              value={config.text_color}
              onChange={(e) => patch({ text_color: e.target.value })}
              className="flex-1 font-mono"
            />
          </div>
        </div>
      </div>

      {/* Opacity */}
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.marquee.opacity}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.marquee.opacityDesc}</p>
        </div>
        <div className="flex items-center gap-2">
          <input
            type="range"
            min="0.1"
            max="1"
            step="0.05"
            value={config.opacity}
            onChange={(e) => patch({ opacity: parseFloat(e.target.value) })}
            className="w-24 h-1 accent-accent"
          />
          <span className="text-xs font-mono text-text-muted w-10 text-right">
            {Math.round(config.opacity * 100)}%
          </span>
        </div>
      </div>
    </div>
  );
}
