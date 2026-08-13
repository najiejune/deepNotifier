import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

type Severity = "Info" | "Warning" | "Critical";

interface ToastItemView {
  id: number;
  title: string;
  body: string;
  severity: Severity;
}

interface ToastSnapshot {
  toasts: ToastItemView[];
  opacity: number;
}

// Geometry must match src-tauri/src/notifier/toast.rs — the backend sizes the
// window to exactly fit N cards of these dimensions.
const CARD_H = 92;
const GAP = 8;
// Transparent padding around the cards; the backend enlarges the window by
// the same amount. Only needs to protect the rounded corners from being
// clipped by the window edge (Windows rounds transparent window corners).
// No box-shadow by design: a shadow extending past the window edge gets
// clipped flat and shows as a faint second rectangle around the card.
const PAD = 8;

// Atlassian / Bitbucket icon colors per severity.
const SEVERITY_STYLE: Record<Severity, { icon: string }> = {
  Info: { icon: "#0052CC" },
  Warning: { icon: "#FF8B00" },
  Critical: { icon: "#DE350B" },
};

// Card background is a light Bitbucket blue. The marquee opacity setting
// controls the transparency, but with a floor: below 0.8 the dark desktop
// would bleed through and sink text contrast below readable levels.
function cardBg(opacity: number): string {
  return `rgba(222, 235, 255, ${Math.max(opacity, 0.8)})`;
}

function SeverityGlyph({ severity }: { severity: Severity }) {
  const color = SEVERITY_STYLE[severity].icon;
  if (severity === "Warning") {
    return (
      <svg width="18" height="18" viewBox="0 0 16 16" style={{ flexShrink: 0, marginTop: 1 }}>
        <path d="M8 1.6 L14.8 13.6 H1.2 Z" fill={color} />
        <rect x="7.25" y="6" width="1.5" height="4" rx="0.75" fill="#fff" />
        <circle cx="8" cy="11.6" r="0.9" fill="#fff" />
      </svg>
    );
  }
  return (
    <svg width="18" height="18" viewBox="0 0 16 16" style={{ flexShrink: 0, marginTop: 1 }}>
      <circle cx="8" cy="8" r="7" fill={color} />
      {severity === "Info" ? (
        <>
          <rect x="7.25" y="7" width="1.5" height="4.5" rx="0.75" fill="#fff" />
          <circle cx="8" cy="4.6" r="1" fill="#fff" />
        </>
      ) : (
        <>
          <rect x="7.25" y="3.8" width="1.5" height="5.4" rx="0.75" fill="#fff" />
          <circle cx="8" cy="11.4" r="1" fill="#fff" />
        </>
      )}
    </svg>
  );
}

function ToastCard({ toast, opacity }: { toast: ToastItemView; opacity: number }) {
  return (
    <div
      onClick={() => invoke("toast_activate", { id: toast.id }).catch(() => {})}
      // Hover pause: restart the backend auto-dismiss timer.
      onMouseEnter={() => invoke("toast_keepalive", { id: toast.id }).catch(() => {})}
      style={{
        position: "relative",
        width: "100%",
        height: CARD_H,
        backgroundColor: cardBg(opacity),
        border: "1px solid rgba(9, 30, 66, 0.14)",
        borderRadius: 8,
        display: "flex",
        alignItems: "flex-start",
        gap: 10,
        padding: "14px 10px 14px 14px",
        cursor: "pointer",
        animation: "toast-in 180ms cubic-bezier(0.2, 0, 0, 1)",
      }}
    >
      {/* Severity glyph, no chip — plain Atlassian flag style */}
      <SeverityGlyph severity={toast.severity} />
      <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 5 }}>
        <div style={{ display: "flex", alignItems: "flex-start", gap: 8 }}>
          <span
            style={{
              flex: 1,
              minWidth: 0,
              fontSize: 13,
              fontWeight: 600,
              lineHeight: "18px",
              color: "#172B4D",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
          >
            {toast.title}
          </span>
          <button
            onClick={(e) => {
              e.stopPropagation();
              invoke("toast_dismiss", { id: toast.id }).catch(() => {});
            }}
            style={{
              flexShrink: 0,
              width: 22,
              height: 22,
              border: "none",
              background: "transparent",
              borderRadius: 6,
              color: "#42526E",
              fontSize: 15,
              fontWeight: 600,
              lineHeight: 1,
              cursor: "pointer",
              padding: 0,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = "rgba(9, 30, 66, 0.1)";
              e.currentTarget.style.color = "#172B4D";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = "transparent";
              e.currentTarget.style.color = "#42526E";
            }}
            aria-label="Dismiss"
          >
            ×
          </button>
        </div>
        <div
          style={{
            fontSize: 12,
            lineHeight: "16px",
            color: "#42526E",
            display: "-webkit-box",
            WebkitBoxOrient: "vertical",
            WebkitLineClamp: 2,
            overflow: "hidden",
            wordBreak: "break-word",
          }}
        >
          {toast.body}
        </div>
      </div>
    </div>
  );
}

function Toasts() {
  const [snapshot, setSnapshot] = useState<ToastSnapshot | null>(null);

  // State acquisition: push events + 1s polling. Same rationale as the
  // marquee page: on hybrid-GPU machines a webview can silently stop
  // receiving Tauri events, so polling get_toast_state is the fallback.
  useEffect(() => {
    const apply = (next: ToastSnapshot | null) =>
      setSnapshot((prev) =>
        JSON.stringify(prev) === JSON.stringify(next) ? prev : next,
      );

    const pull = () => {
      invoke<ToastSnapshot | null>("get_toast_state")
        .then(apply)
        .catch(() => {});
    };
    pull();
    const pollTimer = setInterval(pull, 1000);

    const unlisten = listen<ToastSnapshot>("toast-state", (e) => apply(e.payload));
    return () => {
      clearInterval(pollTimer);
      unlisten.then((f) => f());
    };
  }, []);

  const toasts = snapshot?.toasts ?? [];
  if (toasts.length === 0) return null;

  return (
    <div
      style={{
        width: "100vw",
        display: "flex",
        flexDirection: "column",
        gap: GAP,
        padding: PAD,
      }}
    >
      {toasts.map((t) => (
        <ToastCard key={t.id} toast={t} opacity={snapshot?.opacity ?? 0.9} />
      ))}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("toast-root")!).render(
  <React.StrictMode>
    <Toasts />
  </React.StrictMode>,
);
