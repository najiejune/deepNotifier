import { useState, useEffect } from "react";
import { Copy, Check } from "lucide-react";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { Toggle } from "@/components/ui/Toggle";
import { useI18n } from "@/i18n";
import { api } from "@/lib/tauri";
import type { WebhookConfig } from "@/types";

interface Props {
  config: WebhookConfig;
  onChange: (config: WebhookConfig) => void;
}

const GH_EVENTS = ["push", "pull_request", "issues", "issue_comment", "release", "star", "fork", "watch"];
const GL_EVENTS = ["Push Hook", "Merge Request Hook", "Issue Hook", "Note Hook", "Pipeline Hook"];
const BB_EVENTS = [
  "repo:push", "pullrequest:created", "pullrequest:updated", "pullrequest:approved", "pullrequest:merged",
  "repo:refs_changed", "pr:opened", "pr:modified", "pr:reviewer_approved", "pr:merged", "pr:declined",
];

const LANGS = [
  "bash", "bat", "ps1", "golang", "python", "js", "ts", "rust", "c", "cpp", "java",
] as const;
type Lang = (typeof LANGS)[number];

function genSnippet(lang: Lang, url: string): string {
  const D = JSON.stringify;
  const payload = D({ title: "Deploy completed", body: "Build #42 passed", severity: "Info" });
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
    json={"title": "Deploy completed", "body": "Build #42 passed", "severity": "Info"})`;
    case "js":
      return `fetch("${url}", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ title: "Deploy completed", body: "Build #42 passed", severity: "Info" })
});`;
    case "ts":
      return `await fetch("${url}", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ title: "Deploy completed", body: "Build #42 passed", severity: "Info" })
});`;
    case "rust":
      return `reqwest::Client::new()
    .post("${url}")
    .json(&serde_json::json!({
        "title": "Deploy completed",
        "body": "Build #42 passed",
        "severity": "Info"
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

function CheckboxGroup({
  label,
  allOptions,
  selected,
  onChange,
}: {
  label: string;
  allOptions: string[];
  selected: string[];
  onChange: (v: string[]) => void;
}) {
  return (
    <div>
      <p className="text-[11px] font-mono text-text-muted uppercase tracking-wider mb-1.5">{label}</p>
      <div className="flex flex-wrap gap-1.5">
        {allOptions.map((opt) => {
          const checked = selected.includes(opt);
          return (
            <button
              key={opt}
              onClick={() =>
                onChange(checked ? selected.filter((s) => s !== opt) : [...selected, opt])
              }
              className={`px-2 py-0.5 rounded-sm text-[11px] font-mono border transition-colors ${
                checked
                  ? "bg-accent-dim text-accent border-accent/30"
                  : "bg-white text-text-muted border-border-subtle hover:border-border-default"
              }`}
            >
              {opt}
            </button>
          );
        })}
      </div>
    </div>
  );
}

function CopyBtn({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // fallback
      const ta = document.createElement("textarea");
      ta.value = text;
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
    <button
      onClick={handleCopy}
      className="shrink-0 p-0.5 rounded-sm text-text-muted hover:text-accent hover:bg-accent-dim transition-colors"
      title={copied ? "Copied!" : "Copy URL"}
    >
      {copied ? <Check size={12} /> : <Copy size={12} />}
    </button>
  );
}

function UrlRow({ label, url }: { label: string; url: string }) {
  return (
    <div>
      <p className="text-[11px] font-mono text-text-muted uppercase tracking-wider mb-1.5">{label}</p>
      <div className="flex items-center gap-2">
        <code className="flex-1 text-[11px] font-mono text-text-secondary bg-black/5 rounded-sm px-2 py-1 truncate">
          {url}
        </code>
        <CopyBtn text={url} />
      </div>
    </div>
  );
}

function CustomCodeSnippet({ url }: { url: string }) {
  const [lang, setLang] = useState<Lang>("bash");
  const [copied, setCopied] = useState(false);
  const snippet = genSnippet(lang, url);

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
        <p className="text-[11px] font-mono text-text-muted uppercase tracking-wider">Example</p>
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

export function WebhookSettings({ config, onChange }: Props) {
  const t = useI18n();
  const [hostIp, setHostIp] = useState<string>("localhost");

  useEffect(() => {
    api.getWanIp().then(setHostIp).catch(() => {
      api.getHostIp().then(setHostIp).catch(() => {});
    });
  }, []);

  const githubUrl = `http://${hostIp}:${config.port}/webhook/github`;
  const gitlabUrl = `http://${hostIp}:${config.port}/webhook/gitlab`;
  const bitbucketUrl = `http://${hostIp}:${config.port}/webhook/bitbucket`;
  const customUrl = `http://${hostIp}:${config.port}/webhook/custom`;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.webhook.enable}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.webhook.enableDesc}</p>
        </div>
        <Toggle checked={config.enabled} onChange={(v) => onChange({ ...config, enabled: v })} />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.webhook.port}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.webhook.portDesc}</p>
        </div>
        <Input
          type="number"
          value={config.port}
          onChange={(e) => onChange({ ...config, port: Number(e.target.value) || 3927 })}
          className="w-24"
        />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.webhook.secret}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.webhook.secretDesc}</p>
        </div>
        <Input
          type="password"
          value={config.secret}
          onChange={(e) => onChange({ ...config, secret: e.target.value })}
          placeholder={t.settings.webhook.secretPlaceholder}
          className="w-48"
        />
      </div>

      <div className="border-t border-border-subtle pt-4 space-y-3">
        <UrlRow label="GitHub" url={githubUrl} />
        <CheckboxGroup
          label={t.settings.webhook.githubEvents}
          allOptions={GH_EVENTS}
          selected={config.github_events}
          onChange={(v) => onChange({ ...config, github_events: v })}
        />

        <div className="border-t border-border-subtle/50 pt-3">
          <UrlRow label="GitLab" url={gitlabUrl} />
          <CheckboxGroup
            label={t.settings.webhook.gitlabEvents}
            allOptions={GL_EVENTS}
            selected={config.gitlab_events}
            onChange={(v) => onChange({ ...config, gitlab_events: v })}
          />
        </div>

        <div className="border-t border-border-subtle/50 pt-3">
          <UrlRow label="Bitbucket" url={bitbucketUrl} />
          <CheckboxGroup
            label={t.settings.webhook.bitbucketEvents}
            allOptions={BB_EVENTS}
            selected={config.bitbucket_events ?? []}
            onChange={(v) => onChange({ ...config, bitbucket_events: v })}
          />
        </div>
      </div>

      {/* Custom Webhook */}
      <div className="border-t border-border-subtle pt-4 space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm font-medium">{t.settings.webhook.customEnable}</p>
            <p className="text-xs text-text-muted mt-0.5">{t.settings.webhook.customEnableDesc}</p>
          </div>
          <Toggle checked={config.custom_enabled} onChange={(v) => onChange({ ...config, custom_enabled: v })} />
        </div>

        {config.custom_enabled && (
          <>
            <CustomCodeSnippet url={customUrl} />

            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium">{t.settings.webhook.customTitlePath}</p>
              </div>
              <Input
                value={config.custom_title_path}
                onChange={(e) => onChange({ ...config, custom_title_path: e.target.value })}
                placeholder="title"
                className="w-48"
              />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium">{t.settings.webhook.customBodyPath}</p>
              </div>
              <Input
                value={config.custom_body_path}
                onChange={(e) => onChange({ ...config, custom_body_path: e.target.value })}
                placeholder="body"
                className="w-48"
              />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium">{t.settings.webhook.customSeverity}</p>
              </div>
              <Select
                value={config.custom_severity}
                onChange={(e) => onChange({ ...config, custom_severity: e.target.value })}
                options={[
                  { value: "Info", label: "Info" },
                  { value: "Warning", label: "Warning" },
                  { value: "Critical", label: "Critical" },
                ]}
              />
            </div>
          </>
        )}
      </div>
    </div>
  );
}
