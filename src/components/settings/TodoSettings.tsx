import { useState, useEffect, useRef } from "react";
import { Copy, Check, Pencil, X, Plus, Trash2 } from "lucide-react";
import { Input } from "@/components/ui/Input";
import { Toggle } from "@/components/ui/Toggle";
import { Button } from "@/components/ui/Button";
import { Select } from "@/components/ui/Select";
import { useI18n } from "@/i18n";
import { api } from "@/lib/tauri";
import type { TodoConfig, TodoPullEndpoint } from "@/types";

interface Props {
  config: TodoConfig;
  onChange: (config: TodoConfig) => void;
}

const LANGS = [
  "bash", "bat", "ps1", "golang", "python", "js", "ts", "rust", "c", "cpp", "java",
] as const;
type Lang = (typeof LANGS)[number];

function genTodoSnippet(lang: Lang, url: string): string {
  const payload = JSON.stringify({ text: "Review PR #42", due_date: "2026-05-02" });
  switch (lang) {
    case "bash":
      return `curl -X POST ${url} \\\n  -H "Content-Type: application/json" \\\n  -d '${payload}'`;
    case "bat":
      return `curl -X POST ${url} -H "Content-Type: application/json" -d "${payload.replace(/"/g, '\\"')}"`;
    case "ps1":
      return `Invoke-RestMethod -Uri "${url}" -Method Post -ContentType "application/json" -Body '${payload}'`;
    case "golang":
      return `package main

import (
    "bytes"
    "net/http"
)

func main() {
    body := []byte(\`${payload}\`)
    http.Post("${url}", "application/json", bytes.NewReader(body))
}`;
    case "python":
      return `import requests

requests.post("${url}",
    json={"text": "Review PR #42", "due_date": "2026-05-02"})`;
    case "js":
      return `fetch("${url}", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ text: "Review PR #42", due_date: "2026-05-02" })
});`;
    case "ts":
      return `await fetch("${url}", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ text: "Review PR #42", due_date: "2026-05-02" })
});`;
    case "rust":
      return `reqwest::Client::new()
    .post("${url}")
    .json(&serde_json::json!({
        "text": "Review PR #42",
        "due_date": "2026-05-02"
    }))
    .send()
    .await?;`;
    case "c":
      return `#include <curl/curl.h>

int main() {
    CURL *curl = curl_easy_init();
    if (curl) {
        curl_easy_setopt(curl, CURLOPT_URL, "${url}");
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, "${payload.replace(/"/g, '\\"')}");
        struct curl_slist *headers = NULL;
        headers = curl_slist_append(headers, "Content-Type: application/json");
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_perform(curl);
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);
    }
}`;
    case "cpp":
      return `#include <curl/curl.h>

int main() {
    CURL *curl = curl_easy_init();
    if (curl) {
        curl_easy_setopt(curl, CURLOPT_URL, "${url}");
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, R"(${payload})");
        struct curl_slist *headers = NULL;
        headers = curl_slist_append(headers, "Content-Type: application/json");
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_perform(curl);
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);
    }
}`;
    case "java":
      return `import java.net.http.*;
import java.net.URI;

HttpClient client = HttpClient.newHttpClient();
HttpRequest request = HttpRequest.newBuilder()
    .uri(URI.create("${url}"))
    .header("Content-Type", "application/json")
    .POST(HttpRequest.BodyPublishers.ofString("${payload.replace(/"/g, '\\"')}"))
    .build();
client.send(request, HttpResponse.BodyHandlers.ofString());`;
  }
}

function TodoCodeSnippet({ url }: { url: string }) {
  const [lang, setLang] = useState<Lang>("bash");
  const [copied, setCopied] = useState(false);
  const snippet = genTodoSnippet(lang, url);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(snippet);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = snippet;
      ta.style.position = "fixed";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-1.5">
        <p className="text-xs font-medium text-text-secondary">Example</p>
        <div className="flex items-center gap-2">
          <select
            value={lang}
            onChange={(e) => setLang(e.target.value as Lang)}
            className="text-[11px] font-mono bg-black/5 border border-border-subtle rounded-sm px-2 py-0.5 text-text-secondary"
          >
            {LANGS.map((l) => (
              <option key={l} value={l}>{l}</option>
            ))}
          </select>
          <button
            onClick={handleCopy}
            className="shrink-0 p-0.5 rounded-sm text-text-muted hover:text-accent hover:bg-accent-dim transition-colors"
            title={copied ? "Copied!" : "Copy"}
          >
            {copied ? <Check size={12} /> : <Copy size={12} />}
          </button>
        </div>
      </div>
      <pre className="bg-black/5 border border-border-subtle rounded-sm p-3 text-[11px] font-mono text-text-secondary overflow-x-auto whitespace-pre">
        {snippet}
      </pre>
    </div>
  );
}

// ── Endpoint Dialog ────────────────────────────────────────────

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

function newEndpoint(): TodoPullEndpoint {
  return {
    id: crypto.randomUUID(),
    name: "",
    url: "",
    interval_secs: 86400,
    method: "GET",
    headers: {},
    enabled: true,
  };
}

function EndpointDialog({
  initial,
  onSave,
  onCancel,
}: {
  initial: TodoPullEndpoint;
  onSave: (ep: TodoPullEndpoint) => void;
  onCancel: () => void;
}) {
  const t = useI18n();
  const idCounter = useRef(0);
  const nextId = () => ++idCounter.current;
  const [ep, setEp] = useState<TodoPullEndpoint>({ ...initial, headers: { ...initial.headers } });
  const [rows, setRows] = useState<HeaderRow[]>(() => headersToRows(initial.headers, nextId));
  const [bodyText, setBodyText] = useState(initial.body ?? "");

  const update = (patch: Partial<TodoPullEndpoint>) => setEp((prev) => ({ ...prev, ...patch }));

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
    if (!ep.url.trim()) return;
    onSave({ ...ep, headers: rowsToHeaders(rows), body: bodyText || undefined });
  };

  const isEdit = initial.url !== "";

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/40" onClick={onCancel} />
      <div className="relative bg-bg-base border border-border-default rounded-lg shadow-lg w-[520px] max-h-[85vh] overflow-y-auto p-5 space-y-4">
        <p className="text-sm font-semibold">
          {isEdit ? t.settings.todo.editEndpoint : t.settings.todo.addEndpoint}
        </p>

        <div className="space-y-3">
          <Input
            placeholder={t.settings.todo.endpointName}
            value={ep.name}
            onChange={(e) => update({ name: e.target.value })}
          />
          <div className="flex gap-2">
            <Select
              value={ep.method}
              onChange={(e) => update({ method: e.target.value as TodoPullEndpoint["method"] })}
              options={[
                { value: "GET", label: "GET" },
                { value: "POST", label: "POST" },
              ]}
              className="w-20"
            />
            <Input
              placeholder={t.settings.todo.endpointUrl}
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
              min={1}
            />
            <span className="text-xs text-text-secondary">{t.settings.todo.interval}</span>
          </div>

          {/* Headers */}
          <div>
            <p className="text-xs font-medium text-text-secondary mb-1.5">{t.settings.todo.headers}</p>
            <div className="space-y-1">
              {rows.map((row) => (
                <div key={row.id} className="flex items-center gap-1">
                  <Input
                    placeholder={t.settings.todo.headerKey}
                    value={row.key}
                    onChange={(e) => updateRow(row.id, { key: e.target.value })}
                    className="flex-[2] text-[11px] h-7"
                  />
                  <Input
                    placeholder={t.settings.todo.headerValue}
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
              {t.settings.todo.addHeader}
            </Button>
          </div>

          {/* Body (POST only) */}
          {ep.method === "POST" && (
            <div>
              <p className="text-xs font-medium text-text-secondary mb-1.5">{t.settings.todo.body}</p>
              <textarea
                placeholder={t.settings.todo.bodyPlaceholder}
                value={bodyText}
                onChange={(e) => setBodyText(e.target.value)}
                className="w-full bg-white border border-border-subtle rounded-sm px-2 py-1.5 text-[11px] font-mono text-text-secondary placeholder:text-text-muted resize-y min-h-[64px]"
                rows={3}
              />
            </div>
          )}
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button size="sm" variant="ghost" onClick={onCancel}>{t.settings.todo.cancel}</Button>
          <Button size="sm" onClick={handleSave}>{t.settings.todo.save}</Button>
        </div>
      </div>
    </div>
  );
}

// ── Main Component ──────────────────────────────────────────────

export function TodoSettings({ config, onChange }: Props) {
  const t = useI18n();
  const [hostIp, setHostIp] = useState<string>("localhost");
  const [dialog, setDialog] = useState<TodoPullEndpoint | null>(null);

  useEffect(() => {
    api.getWanIp().then(setHostIp).catch(() => {
      api.getHostIp().then(setHostIp).catch(() => {});
    });
  }, []);

  const pushUrl = `http://${hostIp}:${config.push_port}/todos`;

  const setEndpoints = (eps: TodoPullEndpoint[]) =>
    onChange({ ...config, pull_endpoints: eps });

  const handleSave = (ep: TodoPullEndpoint) => {
    const exists = config.pull_endpoints.some((e) => e.id === ep.id);
    if (exists) {
      setEndpoints(config.pull_endpoints.map((e) => (e.id === ep.id ? ep : e)));
    } else {
      setEndpoints([...config.pull_endpoints, ep]);
    }
    setDialog(null);
  };

  return (
    <div className="space-y-4">
      {/* Pull section */}
      <div>
        <div className="flex items-center justify-between mb-3">
          <div>
            <p className="text-sm font-medium">{t.settings.todo.pull}</p>
            <p className="text-xs text-text-muted mt-0.5">{t.settings.todo.pullEnableDesc}</p>
          </div>
          <Toggle checked={config.pull_enabled} onChange={(v) => onChange({ ...config, pull_enabled: v })} />
        </div>

        {config.pull_enabled && (
          <div className="space-y-2">
            {config.pull_endpoints.map((ep) => (
              <div
                key={ep.id}
                className="bg-bg-layer border border-border-subtle rounded-sm px-3 py-2 flex items-center gap-3 group"
              >
                <div className="flex-1 min-w-0">
                  <p className="text-sm truncate">
                    {ep.name || <span className="text-text-muted">{t.settings.todo.endpointName}</span>}
                  </p>
                  <p className="text-[11px] font-mono text-text-muted truncate">
                    {ep.method} {ep.url || "(no url)"} · {Math.round(ep.interval_secs / 60)}min
                  </p>
                </div>
                <Toggle
                  size="sm"
                  checked={ep.enabled}
                  onChange={(v) => {
                    const next = config.pull_endpoints.map((e) => (e.id === ep.id ? { ...e, enabled: v } : e));
                    setEndpoints(next);
                  }}
                />
                <Button size="sm" variant="ghost" onClick={() => setDialog(ep)}>
                  <Pencil size={12} />
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setEndpoints(config.pull_endpoints.filter((e) => e.id !== ep.id))}
                >
                  <Trash2 size={12} />
                </Button>
              </div>
            ))}

            <Button size="sm" variant="secondary" onClick={() => setDialog(newEndpoint())}>
              <Plus size={12} /> {t.settings.todo.addEndpoint}
            </Button>
          </div>
        )}
      </div>

      {/* Push section */}
      <div className="border-t border-border-subtle pt-4">
        <div className="flex items-center justify-between mb-3">
          <div>
            <p className="text-sm font-medium">{t.settings.todo.push}</p>
            <p className="text-xs text-text-muted mt-0.5">{t.settings.todo.pushEnableDesc}</p>
          </div>
          <Toggle checked={config.push_enabled} onChange={(v) => onChange({ ...config, push_enabled: v })} />
        </div>

        {config.push_enabled && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium">{t.settings.todo.pushPort}</p>
                <p className="text-xs text-text-muted mt-0.5">{t.settings.todo.pushPortDesc}</p>
              </div>
              <Input
                type="number"
                value={config.push_port}
                onChange={(e) => onChange({ ...config, push_port: Number(e.target.value) || 3928 })}
                className="w-24"
              />
            </div>

            <TodoCodeSnippet url={pushUrl} />
          </div>
        )}
      </div>

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
