import {
  Github,
  Gitlab,
  Clock,
  Timer,
  Radio,
  Terminal,
  ArrowUpRight,
  AlertTriangle,
  Info,
  AlertCircle,
} from "lucide-react";
import { cn } from "@/lib/cn";
import { useI18n } from "@/i18n";
import type { NotificationEvent, NotificationSource } from "@/types";

function getSourceLabel(source: NotificationSource) {
  if (source === "GitHub") return "GitHub";
  if (source === "GitLab") return "GitLab";
  if (source === "Bitbucket") return "Bitbucket";
  if (source === "Custom") return "Custom";
  if (source === "Timer") return "Timer";
  if (source === "Pomodoro") return "Pomodoro";
  if (source === "System") return "System";
  if (typeof source === "object" && "Poll" in source)
    return source.Poll.endpoint_name;
  if (typeof source === "object" && "Hook" in source)
    return source.Hook.cli_name;
  return "Unknown";
}

function SourceIcon({ source }: { source: NotificationSource }) {
  const cls = "shrink-0";
  if (source === "GitHub") return <Github size={14} className={cls} />;
  if (source === "GitLab") return <Gitlab size={14} className={cls} />;
  if (source === "Bitbucket") return <Radio size={14} className={cls} />;
  if (source === "Custom") return <Radio size={14} className={cls} />;
  if (source === "Timer") return <Clock size={14} className={cls} />;
  if (source === "Pomodoro") return <Timer size={14} className={cls} />;
  if (source === "System") return <Radio size={14} className={cls} />;
  if (typeof source === "object" && "Hook" in source) return <Terminal size={14} className={cls} />;
  return <Radio size={14} className={cls} />;
}

const severityStyles = {
  Info: {
    border: "border-l-[#00875A]",
    bg: "bg-cyan-dim",
    icon: "text-cyan",
  },
  Warning: {
    border: "border-l-[#FF8B00]",
    bg: "bg-orange-50",
    icon: "text-orange-600",
  },
  Critical: {
    border: "border-l-red",
    bg: "bg-red-dim",
    icon: "text-red",
  },
} as const;

function SeverityBadge({ severity }: { severity: NotificationEvent["severity"] }) {
  const t = useI18n();
  const icons = { Info: Info, Warning: AlertTriangle, Critical: AlertCircle };
  const Icon = icons[severity];
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 text-[10px] font-medium",
        severityStyles[severity].icon,
      )}
    >
      <Icon size={10} />
      {t.notification[severity.toLowerCase() as keyof typeof t.notification] || severity}
    </span>
  );
}

function formatTimestamp(ts: string) {
  const t = useI18n();
  const date = new Date(ts);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return t.notification.justNow;
  if (diffMins < 60) return `${diffMins}${t.notification.mAgo}`;
  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}${t.notification.hAgo}`;
  return date.toLocaleDateString();
}

export function NotificationItem({
  event,
  isNew,
}: {
  event: NotificationEvent;
  isNew?: boolean;
}) {
  const s = severityStyles[event.severity];

  return (
    <div
      className={cn(
        "bg-white border border-border-subtle border-l-2 rounded-sm px-3 py-2.5 transition-all duration-300 group shadow-sm",
        s.border,
        s.bg,
        isNew && "animate-slide-up",
      )}
    >
      <div className="flex items-start gap-2.5">
        <div className={cn("mt-0.5", s.icon)}>
          <SourceIcon source={event.source} />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="text-xs font-medium text-text truncate">
              {event.title}
            </span>
            <SeverityBadge severity={event.severity} />
          </div>
          <p className="text-xs text-text-secondary mt-0.5 line-clamp-2">
            {event.body}
          </p>
          <div className="flex items-center gap-2 mt-1.5">
            <span className="text-[10px] font-mono text-text-muted">
              {getSourceLabel(event.source)}
            </span>
            <span className="text-[10px] text-text-muted/70">
              {formatTimestamp(event.timestamp)}
            </span>
            {event.url && (
              <a
                href={event.url}
                target="_blank"
                rel="noopener noreferrer"
                className="ml-auto text-text-muted hover:text-accent transition-colors"
                onClick={(e) => e.stopPropagation()}
              >
                <ArrowUpRight size={12} />
              </a>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
