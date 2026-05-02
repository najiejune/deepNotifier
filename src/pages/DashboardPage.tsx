import { useEffect, useState, useRef } from "react";
import { Bell, BellOff, Zap } from "lucide-react";
import { useNotifications } from "@/hooks/useNotifications";
import { useDnd } from "@/hooks/useDnd";
import { useI18n } from "@/i18n";
import { NotificationItem } from "@/components/notifications/NotificationItem";
import { cn } from "@/lib/cn";

export function DashboardPage() {
  const t = useI18n();
  const { notifications } = useNotifications(50);
  const { dndActive, toggle } = useDnd();
  const [newIds, setNewIds] = useState<Set<string>>(new Set());
  const prevLen = useRef(0);

  useEffect(() => {
    if (notifications.length > prevLen.current) {
      const newest = notifications.slice(
        0,
        notifications.length - prevLen.current,
      );
      setNewIds(new Set(newest.map((n) => n.id)));
      const timeout = setTimeout(() => setNewIds(new Set()), 800);
      prevLen.current = notifications.length;
      return () => clearTimeout(timeout);
    }
    prevLen.current = notifications.length;
  }, [notifications]);

  return (
    <div className="p-5 space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold tracking-tight text-text">
            {t.dashboard.title}
          </h1>
          <p className="text-xs text-text-muted mt-0.5 font-mono">
            {notifications.length} {t.history.entries}
          </p>
        </div>
        <button
          onClick={toggle}
          className={cn(
            "flex items-center gap-2 px-3 py-1.5 rounded-sm text-xs font-medium border transition-all duration-200",
            dndActive
              ? "bg-red-dim text-red border-red/15 hover:bg-red/10"
              : "bg-white text-text-secondary border-border-subtle hover:border-border-default shadow-sm",
          )}
        >
          {dndActive ? <BellOff size={12} /> : <Bell size={12} />}
          {dndActive ? t.dashboard.dndActive : t.dashboard.dndOff}
        </button>
      </header>

      <div className="bg-white border border-border-subtle rounded-md p-5 shadow-sm">
        <div className="flex items-center gap-2 mb-3">
          <Zap size={14} className="text-accent" />
          <span className="text-[10px] font-mono uppercase tracking-[0.2em] text-text-muted">
            {t.dashboard.liveFeed}
          </span>
          <span className="relative flex h-2 w-2 ml-auto">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-accent opacity-40" />
            <span className="relative inline-flex rounded-full h-2 w-2 bg-accent" />
          </span>
        </div>
        {notifications.length === 0 ? (
          <div className="text-center py-8">
            <Bell size={24} className="mx-auto text-text-muted/30 mb-2" />
            <p className="text-xs text-text-muted">
              {t.dashboard.waitingForNotifications}
            </p>
          </div>
        ) : (
          <div className="space-y-2 max-h-[240px] overflow-auto pr-1">
            {notifications.slice(0, 10).map((n) => (
              <NotificationItem key={n.id} event={n} isNew={newIds.has(n.id)} />
            ))}
          </div>
        )}
      </div>

      <div>
        <h2 className="text-sm font-semibold mb-3 text-text">
          {t.dashboard.recentNotifications}
        </h2>
        {notifications.length === 0 ? (
          <div className="bg-white border border-border-subtle rounded-md p-8 text-center shadow-sm">
            <p className="text-xs text-text-muted">
              {t.dashboard.noNotifications}
            </p>
          </div>
        ) : (
          <div className="space-y-1.5">
            {notifications.map((n) => (
              <NotificationItem key={n.id} event={n} isNew={newIds.has(n.id)} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
