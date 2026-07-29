import { useState } from "react";
import { api } from "@/lib/tauri";
import { Button } from "@/components/ui/Button";

export function DebugFocus() {
  const [pid, setPid] = useState("");
  const [result, setResult] = useState<string>("");
  const [pendingPid, setPendingPid] = useState<string>("");

  const testFocus = async () => {
    const p = parseInt(pid, 10);
    if (isNaN(p)) {
      setResult("Invalid PID");
      return;
    }
    try {
      const r = await api.debugFocusPid(p);
      setResult(JSON.stringify(r, null, 2));
    } catch (e) {
      setResult(`Error: ${e}`);
    }
  };

  const checkPending = async () => {
    try {
      const p = await api.debugGetPendingPid();
      setPendingPid(p === null ? "None" : String(p));
    } catch (e) {
      setPendingPid(`Error: ${e}`);
    }
  };

  const testFocusPending = async () => {
    try {
      await api.focusPendingPid();
      setResult("focusPendingPid called (check logs)");
    } catch (e) {
      setResult(`Error: ${e}`);
    }
  };

  return (
    <div className="space-y-3">
      <p className="text-xs text-text-muted">
        Test notification click → window focus pipeline. Check browser console and Rust
        logs for trace output.
      </p>

      <div className="flex items-center gap-2">
        <Button size="sm" variant="secondary" onClick={checkPending}>
          Check Pending PID
        </Button>
        <span className="text-xs text-text-muted font-mono">
          {pendingPid || "(not checked)"}
        </span>
      </div>

      <div className="flex items-center gap-2">
        <Button size="sm" variant="secondary" onClick={testFocusPending}>
          Test focusPendingPid
        </Button>
        <span className="text-xs text-text-muted">
          Calls the same command that onAction triggers
        </span>
      </div>

      <div className="flex items-center gap-2">
        <input
          type="text"
          placeholder="Enter PID"
          value={pid}
          onChange={(e) => setPid(e.target.value)}
          className="w-40 px-2 py-1 text-xs border border-border-subtle rounded bg-bg-base text-text font-mono"
        />
        <Button size="sm" variant="secondary" onClick={testFocus}>
          Test Focus PID
        </Button>
      </div>

      {result && (
        <pre className="text-xs font-mono bg-bg-muted p-2 rounded border border-border-subtle max-h-40 overflow-auto whitespace-pre-wrap">
          {result}
        </pre>
      )}
    </div>
  );
}
