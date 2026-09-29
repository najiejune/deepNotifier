import { useState, useEffect, useCallback, useRef } from "react";
import { Save, Plug, Unplug, Trash2, AlertTriangle } from "lucide-react";
import { Input } from "@/components/ui/Input";
import { Toggle } from "@/components/ui/Toggle";
import { Button } from "@/components/ui/Button";
import { useI18n } from "@/i18n";
import { useConfig } from "@/hooks/useConfig";
import { usePlatform } from "@/hooks/usePlatform";
import { api, onEvent } from "@/lib/tauri";
import { cn } from "@/lib/cn";
import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";
import type { ServerConfig, ServerStatus } from "@/types";

interface Props {
  config: ServerConfig;
  onChange: (config: ServerConfig) => void;
}

const stateDot: Record<ServerStatus["state"], string> = {
  offline: "bg-border-strong",
  connecting: "bg-amber-400",
  online: "bg-cyan",
};

export function ServerSettings({ config, onChange }: Props) {
  const t = useI18n();
  const { isMobile } = usePlatform();
  const { reload } = useConfig();
  const [url, setUrl] = useState(config.server_url);
  const [token, setToken] = useState(config.api_token);
  // Dirty flags: a reload() (e.g. after register) must not silently discard
  // edits the user is in the middle of typing.
  const urlTouched = useRef(false);
  const tokenTouched = useRef(false);
  const [status, setStatus] = useState<ServerStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [confirmUnregister, setConfirmUnregister] = useState(false);

  // OS-level notification permission so alerts can actually show on mobile.
  const ensureNotifyPermission = useCallback(async () => {
    if (!isMobile) return;
    try {
      if (!(await isPermissionGranted())) {
        await requestPermission();
      }
    } catch {
      // plugin unavailable on this platform — ignore
    }
  }, [isMobile]);

  useEffect(() => {
    if (!urlTouched.current) setUrl(config.server_url);
    if (!tokenTouched.current) setToken(config.api_token);
  }, [config.server_url, config.api_token]);

  useEffect(() => {
    api.getMobileServerStatus().then(setStatus).catch(() => {});
    // The backend owns device_id/ws_token — refresh our copy of the config.
    reload();
    const unlisten = onEvent<ServerStatus>("server-status", (s) => {
      setStatus(s);
      if (s.state === "online") ensureNotifyPermission();
    });
    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, [ensureNotifyPermission, reload]);

  const handleRegister = async () => {
    setBusy(true);
    setActionError(null);
    try {
      await api.mobileServerRegister(url.trim(), token.trim());
      await ensureNotifyPermission();
      await api.mobileServerConnect();
      await reload();
      // Backend-trimmed values win again over local edits.
      urlTouched.current = false;
      tokenTouched.current = false;
    } catch (e) {
      setActionError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleToggleConnect = async () => {
    if (!status) return;
    setBusy(true);
    setActionError(null);
    try {
      if (status.state === "online" || status.state === "connecting") {
        await api.mobileServerDisconnect();
      } else {
        await api.mobileServerConnect();
      }
    } catch (e) {
      setActionError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleUnregister = async () => {
    setBusy(true);
    setActionError(null);
    try {
      await api.mobileServerUnregister();
      await reload();
      urlTouched.current = false;
      tokenTouched.current = false;
    } catch (e) {
      setActionError(String(e));
    } finally {
      setBusy(false);
      setConfirmUnregister(false);
    }
  };

  const state = status?.state ?? "offline";
  const deviceId = status?.device_id || config.device_id;
  const connected = state === "online" || state === "connecting";
  const stateLabel = {
    offline: t.settings.server.stateOffline,
    connecting: t.settings.server.stateConnecting,
    online: t.settings.server.stateOnline,
  }[state];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-3">
        <div>
          <p className="text-sm font-medium">{t.settings.server.serverUrl}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.server.serverUrlDesc}</p>
        </div>
        <Input
          value={url}
          onChange={(e) => {
            urlTouched.current = true;
            setUrl(e.target.value);
          }}
          placeholder={t.settings.server.serverUrlPlaceholder}
          className="w-64 max-w-[50%] font-mono text-xs"
        />
      </div>

      <div className="flex items-center justify-between gap-3">
        <div>
          <p className="text-sm font-medium">{t.settings.server.apiToken}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.server.apiTokenDesc}</p>
        </div>
        <Input
          type="password"
          value={token}
          onChange={(e) => {
            tokenTouched.current = true;
            setToken(e.target.value);
          }}
          placeholder={t.settings.server.apiTokenPlaceholder}
          className="w-64 max-w-[50%] font-mono text-xs"
        />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{t.settings.server.autoConnect}</p>
          <p className="text-xs text-text-muted mt-0.5">{t.settings.server.autoConnectDesc}</p>
        </div>
        <Toggle
          checked={config.auto_connect}
          onChange={(v) => onChange({ ...config, auto_connect: v })}
        />
      </div>

      {/* Live connection status */}
      <div className="border-t border-border-subtle pt-4 space-y-1.5">
        <div className="flex items-center gap-2">
          <span className={cn("w-2 h-2 rounded-full", stateDot[state])} />
          <span className="text-sm font-medium">{stateLabel}</span>
          {status?.server_url && (
            <code className="text-[11px] font-mono text-text-muted bg-black/5 rounded-sm px-1.5 py-0.5 truncate">
              {status.server_url}
            </code>
          )}
        </div>
        <p className="text-xs text-text-muted">
          {t.settings.server.deviceId}:{" "}
          <span className="font-mono">{deviceId || t.settings.server.notRegistered}</span>
        </p>
        {(status?.last_error || actionError) && (
          <p className="text-xs text-red break-words">
            {status?.last_error ?? actionError}
          </p>
        )}
      </div>

      <div className="flex flex-wrap items-center gap-2 border-t border-border-subtle pt-4">
        <Button
          size="sm"
          variant="primary"
          disabled={busy || !url.trim() || !token.trim()}
          onClick={handleRegister}
        >
          <Save size={12} /> {t.settings.server.saveRegister}
        </Button>
        <Button
          size="sm"
          variant="secondary"
          disabled={busy || !deviceId}
          onClick={handleToggleConnect}
        >
          {connected ? <Unplug size={12} /> : <Plug size={12} />}
          {connected ? t.settings.server.disconnect : t.settings.server.connect}
        </Button>
        {confirmUnregister ? (
          <>
            <span className="text-xs text-text-muted">
              {t.settings.server.unregisterConfirm}
            </span>
            <Button size="sm" variant="danger" disabled={busy} onClick={handleUnregister}>
              <AlertTriangle size={12} /> {t.settings.server.unregister}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              className="text-text-muted hover:text-text"
              onClick={() => setConfirmUnregister(false)}
            >
              {t.settings.poll.cancel}
            </Button>
          </>
        ) : (
          <Button
            size="sm"
            variant="ghost"
            className="text-text-muted hover:text-red"
            disabled={busy || !deviceId}
            onClick={() => setConfirmUnregister(true)}
          >
            <Trash2 size={12} /> {t.settings.server.unregister}
          </Button>
        )}
      </div>
    </div>
  );
}
