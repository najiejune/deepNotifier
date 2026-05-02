import { useState, useMemo } from "react";
import { Search, Trash2, X } from "lucide-react";
import { useNotifications } from "@/hooks/useNotifications";
import { useI18n } from "@/i18n";
import { NotificationItem } from "@/components/notifications/NotificationItem";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import type { NotificationSource } from "@/types";

function sourceToString(s: NotificationSource): string {
  if (typeof s === "object" && "Poll" in s) return s.Poll.endpoint_name;
  return s;
}

export function HistoryPage() {
  const t = useI18n();
  const { notifications, clear } = useNotifications(200);
  const [search, setSearch] = useState("");
  const [severityFilter, setSeverityFilter] = useState<string>("all");
  const [sourceFilter, setSourceFilter] = useState<string>("all");
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");

  const sources = useMemo(() => {
    const set = new Set<string>();
    notifications.forEach((n) => set.add(sourceToString(n.source)));
    return Array.from(set);
  }, [notifications]);

  const filtered = useMemo(() => {
    return notifications.filter((n) => {
      if (severityFilter !== "all" && n.severity !== severityFilter)
        return false;
      if (sourceFilter !== "all" && sourceToString(n.source) !== sourceFilter)
        return false;
      if (search) {
        const q = search.toLowerCase();
        if (
          !n.title.toLowerCase().includes(q) &&
          !n.body.toLowerCase().includes(q)
        )
          return false;
      }
      if (dateFrom || dateTo) {
        const ts = new Date(n.timestamp).getTime();
        if (dateFrom && ts < new Date(dateFrom).getTime()) return false;
        if (dateTo) {
          const end = new Date(dateTo);
          end.setHours(23, 59, 59, 999);
          if (ts > end.getTime()) return false;
        }
      }
      return true;
    });
  }, [notifications, search, severityFilter, sourceFilter, dateFrom, dateTo]);

  return (
    <div className="p-5 space-y-4 h-full flex flex-col">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold tracking-tight text-text">
            {t.history.title}
          </h1>
          <p className="text-xs text-text-muted mt-0.5 font-mono">
            {filtered.length} {t.history.of} {notifications.length}{" "}
            {t.history.entries}
          </p>
        </div>
        <Button size="sm" variant="danger" onClick={clear}>
          <Trash2 size={12} /> {t.history.clearAll}
        </Button>
      </header>

      {/* Filter bar — row 1: search + quick filters */}
      <div className="flex items-center gap-2">
        <div className="relative flex-1">
          <Search
            size={12}
            className="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none"
          />
          <Input
            placeholder={t.history.search}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-7 text-xs"
          />
        </div>
        <Select
          value={severityFilter}
          onChange={(e) => setSeverityFilter(e.target.value)}
          options={[
            { value: "all", label: t.history.allSeverity },
            { value: "Info", label: t.notification.info },
            { value: "Warning", label: t.notification.warning },
            { value: "Critical", label: t.notification.critical },
          ]}
          className="w-28"
        />
        <Select
          value={sourceFilter}
          onChange={(e) => setSourceFilter(e.target.value)}
          options={[
            { value: "all", label: t.history.allSources },
            ...sources.map((s) => ({ value: s, label: s })),
          ]}
          className="w-32"
        />
      </div>

      {/* Filter bar — row 2: date range */}
      <div className="flex items-center gap-2">
        <Input
          type="date"
          value={dateFrom}
          onChange={(e) => setDateFrom(e.target.value)}
          className="w-36 text-xs"
        />
        <span className="text-[11px] text-text-muted">{t.history.dateTo}</span>
        <Input
          type="date"
          value={dateTo}
          onChange={(e) => setDateTo(e.target.value)}
          className="w-36 text-xs"
        />
        {(dateFrom || dateTo) && (
          <button
            onClick={() => { setDateFrom(""); setDateTo(""); }}
            className="flex items-center gap-1 text-[11px] text-text-muted hover:text-text-secondary transition-colors"
          >
            <X size={10} />
          </button>
        )}
      </div>

      <div className="flex-1 overflow-auto space-y-1.5 pr-1">
        {filtered.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-xs text-text-muted font-mono">
              {t.history.noMatching}
            </p>
          </div>
        ) : (
          filtered.map((n) => <NotificationItem key={n.id} event={n} />)
        )}
      </div>
    </div>
  );
}
