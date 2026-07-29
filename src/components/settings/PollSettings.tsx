import { useState, useRef } from "react";
import { Input } from "@/components/ui/Input";
import { Toggle } from "@/components/ui/Toggle";
import { Button } from "@/components/ui/Button";
import { Select } from "@/components/ui/Select";
import { Plus, Trash2, X, Pencil } from "lucide-react";
import { useI18n } from "@/i18n";
import type { PollConfig, PollEndpoint } from "@/types";

interface Props {
  config: PollConfig;
  onChange: (config: PollConfig) => void;
}

function newEndpoint(): PollEndpoint {
  return {
    id: crypto.randomUUID(),
    name: "",
    url: "",
    interval_secs: 300,
    timeout_secs: 30,
    method: "GET",
    headers: {},
    enabled: true,
  };
}

interface HeaderRow {
  id: number;
  key: string;
  value: string;
}

function headersToRows(h: Record<string, string>, nextId: () => number): HeaderRow[] {
  return Object.entries(h).map(([key, value]) => ({ id: nextId(), key, value }));
}

function rowsToHeaders(rows: HeaderRow[]): Record<string, string> {
  const result: Record<string, string> = {};
  for (const r of rows) {
    if (r.key) result[r.key] = r.value;
  }
  return result;
}

function EndpointDialog({
  initial,
  onSave,
  onCancel,
}: {
  initial: PollEndpoint;
  onSave: (ep: PollEndpoint) => void;
  onCancel: () => void;
}) {
  const t = useI18n();
  const idCounter = useRef(0);
  const nextId = () => ++idCounter.current;
  const [ep, setEp] = useState<PollEndpoint>({ ...initial, headers: { ...initial.headers } });
  const [rows, setRows] = useState<HeaderRow[]>(() => headersToRows(initial.headers, nextId));

  const update = (patch: Partial<PollEndpoint>) => setEp((prev) => ({ ...prev, ...patch }));

  const updateRow = (id: number, patch: Partial<HeaderRow>) => {
    setRows((prev) => prev.map((r) => (r.id === id ? { ...r, ...patch } : r)));
  };

  const removeRow = (id: number) => {
    setRows((prev) => prev.filter((r) => r.id !== id));
  };

  const addRow = () => {
    setRows((prev) => [...prev, { id: nextId(), key: "", value: "" }]);
  };

  const handleSave = () => {
    if (!ep.name.trim() && !ep.url.trim()) return;
    const headers = rowsToHeaders(rows);
    onSave({ ...ep, headers });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={onCancel} />
      <div className="relative bg-bg-base border border-border-default rounded-lg shadow-lg w-[520px] max-h-[85vh] overflow-y-auto p-5 space-y-4">
        <p className="text-sm font-semibold">
          {initial.name || initial.url ? t.settings.poll.editEndpoint : t.settings.poll.addEndpoint}
        </p>

        <div className="space-y-3">
          <Input
            placeholder={t.settings.poll.endpointName}
            value={ep.name}
            onChange={(e) => update({ name: e.target.value })}
          />
          <div className="flex gap-2">
            <Select
              value={ep.method}
              onChange={(e) => update({ method: e.target.value as PollEndpoint["method"] })}
              options={[
                { value: "GET", label: "GET" },
                { value: "POST", label: "POST" },
              ]}
              className="w-20"
            />
            <Input
              placeholder={t.settings.poll.endpointUrl}
              value={ep.url}
              onChange={(e) => update({ url: e.target.value })}
              className="flex-1"
            />
          </div>
          <div className="flex items-center gap-2">
            <Input
              type="number"
              value={Math.round(ep.interval_secs / 60)}
              onChange={(e) => update({ interval_secs: (Number(e.target.value) || 1) * 60 })}
              className="w-20"
            />
            <span className="text-xs text-text-secondary">{t.settings.poll.interval}</span>
            <Input
              type="number"
              value={ep.timeout_secs}
              onChange={(e) => update({ timeout_secs: Number(e.target.value) || 30 })}
              className="w-20"
            />
            <span className="text-xs text-text-secondary">{t.settings.poll.timeout}</span>
          </div>

          {/* Headers */}
          <div>
            <p className="text-xs font-medium text-text-secondary mb-1.5">{t.settings.poll.headers}</p>
            <div className="space-y-1">
              {rows.map((row) => (
                <div key={row.id} className="flex items-center gap-1">
                  <Input
                    placeholder={t.settings.poll.headerKey}
                    value={row.key}
                    onChange={(e) => updateRow(row.id, { key: e.target.value })}
                    className="flex-[2] text-[11px] h-7"
                  />
                  <Input
                    placeholder={t.settings.poll.headerValue}
                    value={row.value}
                    onChange={(e) => updateRow(row.id, { value: e.target.value })}
                    className="flex-[3] text-[11px] h-7"
                  />
                  <button
                    onClick={() => removeRow(row.id)}
                    className="shrink-0 p-0.5 text-text-muted hover:text-red-400"
                  >
                    <X size={12} />
                  </button>
                </div>
              ))}
            </div>
            <Button size="sm" variant="ghost" onClick={addRow} className="mt-1 text-[11px]">
              {t.settings.poll.addHeader}
            </Button>
          </div>

          {/* Body (POST only) */}
          {ep.method === "POST" && (
            <div>
              <p className="text-xs font-medium text-text-secondary mb-1.5">{t.settings.poll.body}</p>
              <textarea
                placeholder={t.settings.poll.bodyPlaceholder}
                value={ep.body ?? ""}
                onChange={(e) => update({ body: e.target.value })}
                className="w-full bg-white border border-border-subtle rounded-sm px-2 py-1.5 text-[11px] font-mono text-text-secondary placeholder:text-text-muted resize-y min-h-[64px]"
                rows={3}
              />
            </div>
          )}
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button size="sm" variant="ghost" onClick={onCancel}>{t.settings.poll.cancel}</Button>
          <Button size="sm" onClick={handleSave}>{t.settings.poll.save}</Button>
        </div>
      </div>
    </div>
  );
}

export function PollSettings({ config, onChange }: Props) {
  const t = useI18n();
  const [dialog, setDialog] = useState<PollEndpoint | null>(null);

  const setEndpoints = (eps: PollEndpoint[]) =>
    onChange({ ...config, endpoints: eps });

  const handleSave = (ep: PollEndpoint) => {
    const exists = config.endpoints.some((e) => e.id === ep.id);
    if (exists) {
      setEndpoints(config.endpoints.map((e) => (e.id === ep.id ? ep : e)));
    } else {
      setEndpoints([...config.endpoints, ep]);
    }
    setDialog(null);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.poll.enable}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.poll.enableDesc}</p>
        </div>
        <Toggle checked={config.enabled} onChange={(v) => onChange({ ...config, enabled: v })} />
      </div>

      <div className="space-y-2">
        {config.endpoints.map((ep) => (
          <div
            key={ep.id}
            className="bg-bg-layer border border-border-subtle rounded-sm px-3 py-2 flex items-center gap-3 group"
          >
            <div className="flex-1 min-w-0">
              <p className="text-sm truncate">{ep.name || <span className="text-text-muted">Unnamed</span>}</p>
              <p className="text-[11px] font-mono text-text-muted truncate">
                {ep.method} {ep.url || "(no url)"} · {Math.round(ep.interval_secs / 60)}min
              </p>
            </div>
            <Toggle
              size="sm"
              checked={ep.enabled}
              onChange={(v) => {
                const next = config.endpoints.map((e) => (e.id === ep.id ? { ...e, enabled: v } : e));
                setEndpoints(next);
              }}
            />
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setDialog(ep)}
            >
              <Pencil size={12} />
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                setEndpoints(config.endpoints.filter((e) => e.id !== ep.id));
              }}
            >
              <Trash2 size={12} />
            </Button>
          </div>
        ))}
      </div>

      <Button size="sm" variant="secondary" onClick={() => setDialog(newEndpoint())}>
        <Plus size={12} /> {t.settings.poll.addEndpoint}
      </Button>

      {dialog && (
        <EndpointDialog
          initial={dialog}
          onSave={handleSave}
          onCancel={() => setDialog(null)}
        />
      )}
    </div>
  );
}
