import { Radio, Volume2, Monitor, Bell, BellOff } from "lucide-react";
import { cn } from "@/lib/cn";
import { useDnd } from "@/hooks/useDnd";
import { useConfig } from "@/hooks/useConfig";
import { useI18n } from "@/i18n";
import { getCurrentWindow } from "@tauri-apps/api/window";

const win = getCurrentWindow();

const modeCfgMap = {
  dnd:      { dot: "bg-red",            icon: BellOff },
  standard: { dot: "bg-emerald-400",     icon: Bell },
  sound:    { dot: "bg-amber-400",       icon: Volume2 },
  marquee:  { dot: "bg-cyan",            icon: Monitor },
} as const;

type Mode = keyof typeof modeCfgMap;

const modeLabels: Record<Mode, keyof ReturnType<typeof import("@/i18n").useI18n>["titlebar"]> = {
  dnd:      "dnd",
  standard: "modeStandard",
  sound:    "modeSound",
  marquee:  "modeMarquee",
};

export function TitleBar() {
  const { dndActive, toggle } = useDnd();
  const { config } = useConfig();
  const t = useI18n();

  // Compute mode — runtime DND status OR persisted DND config
  const effectiveDnd = dndActive || config.dnd.enabled;

  let mode: Mode;
  if (effectiveDnd) {
    mode = "dnd";
  } else {
    const s = config.notification.sound_enabled;
    const m = config.notification.marquee_enabled;
    if (s && m) mode = "standard";
    else if (s) mode = "sound";
    else if (m) mode = "marquee";
    else mode = "dnd"; // both off = effective DND
  }

  const { dot, icon: Icon } = modeCfgMap[mode];
  const label = t.titlebar[modeLabels[mode]];

  return (
    <div
      className="flex items-center justify-between h-9 shrink-0 select-none"
      style={{ backgroundColor: "#0049b0" }}
    >
      <div data-tauri-drag-region className="flex items-center w-40 shrink-0 pl-4 border-r border-white/15">
        <div className="flex items-center gap-2">
          <Radio size={14} className="text-white" />
          <span className="text-xs font-semibold text-white">
            {t.app.title}
          </span>
        </div>
      </div>

      <div data-tauri-drag-region className="flex-1 h-full flex items-center pl-4">
        <button
          onClick={toggle}
          className={cn(
            "flex items-center gap-1.5 px-2 py-0.5 rounded-sm text-[11px] font-medium transition-all duration-200",
            mode === "dnd"
              ? "bg-red/30 text-white border border-red/30"
              : "text-white/70 hover:text-white hover:bg-white/10",
          )}
        >
          <Icon size={10} />
          {label}
          <span className={cn("ml-0.5 w-1.5 h-1.5 rounded-full", dot)} />
        </button>
      </div>

      <div className="flex items-center h-full">
        {/* Minimize — VSCode-style thick underscore */}
        <button
          onClick={(e) => { e.stopPropagation(); win.minimize(); }}
          className="h-full w-11 flex items-center justify-center text-white hover:bg-white/10 transition-colors"
        >
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="1" y="5.5" width="10" height="1" fill="currentColor" />
          </svg>
        </button>

        {/* Maximize — VSCode-style thick square outline */}
        <button
          onClick={(e) => { e.stopPropagation(); win.toggleMaximize(); }}
          className="h-full w-11 flex items-center justify-center text-white hover:bg-white/10 transition-colors"
        >
          <svg width="10" height="10" viewBox="0 0 10 10">
            <rect x="1" y="1" width="8" height="8" fill="none" stroke="currentColor" strokeWidth="1" rx="0" />
          </svg>
        </button>

        {/* Close — VSCode-style thick X */}
        <button
          onClick={(e) => { e.stopPropagation(); win.hide(); }}
          className="h-full w-11 flex items-center justify-center text-white hover:bg-[#c42b1c] transition-colors"
        >
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path d="M2 2L10 10M10 2L2 10" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
        </button>
      </div>
    </div>
  );
}
