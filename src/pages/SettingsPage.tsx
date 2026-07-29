import { useState } from "react";
import {
  Globe,
  Webhook,
  Rss,
  Bell,
  Moon,
  Clock,
  Presentation,
  ListTodo,
  Terminal,
  RotateCcw,
  AlertTriangle,
  Bug,
} from "lucide-react";
import { cn } from "@/lib/cn";
import { useConfig } from "@/hooks/useConfig";
import { useI18n } from "@/i18n";
import { Button } from "@/components/ui/Button";
import { GeneralSettings } from "@/components/settings/GeneralSettings";
import { WebhookSettings } from "@/components/settings/WebhookSettings";
import { PollSettings } from "@/components/settings/PollSettings";
import { NotificationSettings } from "@/components/settings/NotificationSettings";
import { DndSettings } from "@/components/settings/DndSettings";
import { PomodoroSettings } from "@/components/settings/PomodoroSettings";
import { MarqueeSettings } from "@/components/settings/MarqueeSettings";
import { TodoSettings } from "@/components/settings/TodoSettings";
import { HookSettings } from "@/components/settings/HookSettings";
import { DebugFocus } from "@/components/settings/DebugFocus";

type Section =
  | "general"
  | "webhook"
  | "poll"
  | "notification"
  | "dnd"
  | "pomodoro"
  | "marquee"
  | "hook"
  | "todo"
  | "debug";

export function SettingsPage() {
  const { config, loading, update, reset } = useConfig();
  const t = useI18n();
  const [activeSection, setActiveSection] = useState<Section>("general");
  const [confirmReset, setConfirmReset] = useState(false);

  if (loading) {
    return (
      <div className="p-5">
        <span className="text-xs text-text-muted">Loading...</span>
      </div>
    );
  }

  const sections: { id: Section; label: string; icon: typeof Globe }[] = [
    { id: "general", label: t.settings.sections.general, icon: Globe },
    { id: "webhook", label: t.settings.sections.webhook, icon: Webhook },
    { id: "poll", label: t.settings.sections.poll, icon: Rss },
    { id: "notification", label: t.settings.sections.notifications, icon: Bell },
    { id: "dnd", label: t.settings.sections.dnd, icon: Moon },
    { id: "pomodoro", label: t.settings.sections.pomodoro, icon: Clock },
    { id: "marquee", label: t.settings.sections.marquee, icon: Presentation },
    { id: "hook", label: t.settings.sections.hook, icon: Terminal },
    { id: "todo", label: t.settings.sections.todo, icon: ListTodo },
    { id: "debug", label: "Debug Focus", icon: Bug },
  ];

  const renderSection = () => {
    switch (activeSection) {
      case "general":
        return (
          <GeneralSettings
            config={config.general}
            onChange={(v) => update("general", v)}
          />
        );
      case "webhook":
        return (
          <WebhookSettings
            config={config.webhook}
            onChange={(v) => update("webhook", v)}
          />
        );
      case "poll":
        return (
          <PollSettings
            config={config.poll}
            onChange={(v) => update("poll", v)}
          />
        );
      case "notification":
        return (
          <NotificationSettings
            config={config.notification}
            onChange={(v) => update("notification", v)}
          />
        );
      case "dnd":
        return (
          <DndSettings
            config={config.dnd}
            onChange={(v) => update("dnd", v)}
          />
        );
      case "pomodoro":
        return (
          <PomodoroSettings
            config={config.timer}
            onChange={(v) => update("timer", v)}
          />
        );
      case "marquee":
        return (
          <MarqueeSettings
            config={config.marquee}
            onChange={(v) => update("marquee", v)}
          />
        );
      case "hook":
        return (
          <HookSettings
            config={config.hook}
            onChange={(v) => update("hook", v)}
          />
        );
      case "todo":
        return (
          <TodoSettings
            config={config.todo}
            onChange={(v) => update("todo", v)}
          />
        );
      case "debug":
        return <DebugFocus />;
    }
  };

  return (
    <div className="p-2.5 h-full flex flex-col">
      <header className="flex items-center justify-between mb-4">
        <div>
          <h1 className="text-lg font-semibold tracking-tight text-text">
            {t.settings.title}
          </h1>
          <p className="text-xs text-text-muted mt-0.5 font-mono">
            {t.settings.subtitle}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {confirmReset ? (
            <>
              <span className="text-xs text-text-muted font-medium">
                确认恢复默认设置？
              </span>
              <Button
                size="sm"
                variant="ghost"
                onClick={async () => {
                  await reset();
                  setConfirmReset(false);
                }}
              >
                <AlertTriangle size={12} /> 确认
              </Button>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => setConfirmReset(false)}
                className="text-text-muted hover:text-text"
              >
                取消
              </Button>
            </>
          ) : (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setConfirmReset(true)}
            >
              <RotateCcw size={12} /> {t.settings.reset}
            </Button>
          )}
        </div>
      </header>

      <div className="flex gap-2 flex-1 overflow-hidden">
        <div className="w-40 shrink-0 space-y-0.5">
          {sections.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActiveSection(id)}
              className={cn(
                "w-full flex items-center gap-2.5 px-3 py-2 rounded-sm text-sm transition-all duration-150",
                activeSection === id
                  ? "bg-white text-text border border-border-subtle shadow-sm"
                  : "text-text-secondary hover:text-text hover:bg-white/60 border border-transparent",
              )}
            >
              <Icon
                size={14}
                className={activeSection === id ? "text-accent" : "text-text-muted"}
              />
              {label}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-auto pr-1">
          <div className="bg-white border border-border-subtle rounded-md p-5 shadow-sm">
            <h2 className="text-sm font-semibold mb-4 text-text">
              {sections.find((s) => s.id === activeSection)?.label}
            </h2>
            {renderSection()}
          </div>
        </div>
      </div>
    </div>
  );
}
