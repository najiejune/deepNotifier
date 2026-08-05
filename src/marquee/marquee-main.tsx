import React, { useEffect, useRef, useState } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface MarqueeConfig {
  position: "Top" | "Bottom";
  speed: number;
  height: number;
  font_size: number;
  font_family: string;
  icon_before: string;
  icon_after: string;
  bg_color: string;
  text_color: string;
  opacity: number;
  duration_secs: number;
  tracks: number;
}

interface MarqueeSnapshot {
  tracks: (string | null)[];
  config: MarqueeConfig;
}

const DEFAULT_CONFIG: MarqueeConfig = {
  position: "Top",
  speed: 100,
  height: 40,
  font_size: 16,
  font_family: "sans-serif",
  icon_before: "",
  icon_after: "",
  bg_color: "#1e3a5f",
  text_color: "#ffffff",
  opacity: 0.9,
  duration_secs: 30,
  tracks: 2,
};

const SEPARATOR = "  •  ";

/// Background color with the opacity applied to its alpha channel. The
/// opacity setting controls ONLY the bar's background: text always renders at
/// full strength so it stays readable on a see-through bar. Non-hex colors
/// fall back to being used as-is.
function bgWithOpacity(bg: string, opacity: number): string {
  if (bg === "transparent") return "transparent";
  const short = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/i.exec(bg);
  const long = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(bg);
  const [r, g, b] = long
    ? [parseInt(long[1], 16), parseInt(long[2], 16), parseInt(long[3], 16)]
    : short
      ? [parseInt(short[1] + short[1], 16), parseInt(short[2] + short[2], 16), parseInt(short[3] + short[3], 16)]
      : [];
  if (r === undefined) return bg;
  return `rgba(${r}, ${g}, ${b}, ${opacity})`;
}

function trackContent(config: MarqueeConfig, text: string): string {
  return (
    (config.icon_before ? config.icon_before + " " : "") +
    text +
    (config.icon_after ? " " + config.icon_after : "")
  );
}

function TrackRow({
  text,
  config,
  heightPct,
}: {
  text: string;
  config: MarqueeConfig;
  heightPct: number;
}) {
  const innerRef = useRef<HTMLDivElement>(null);
  const content = trackContent(config, text);
  const duration = content.length * (100 / config.speed);

  // JS-driven scrolling via `left` (NO CSS transform/animation). CSS
  // transform animations get their own compositor layer, which fails to
  // composite on the dGPU-driven monitor of hybrid-GPU laptops: the strip
  // background paints but the animated text layer never appears. Updating
  // `left` on an interval stays on the plain paint path and renders anywhere.
  useEffect(() => {
    const el = innerRef.current;
    if (!el) return;
    let start: number | null = null;
    const timer = setInterval(() => {
      const now = performance.now();
      if (start === null) start = now;
      const track = el.scrollWidth; // includes the 100% left padding
      const dist = track * 0.6666;
      const phase = (((now - start) / 1000) % duration) / duration;
      el.style.left = `${-phase * dist}px`;
    }, 33);
    return () => clearInterval(timer);
  }, [content, duration]);

  return (
    <div
      style={{
        width: "100%",
        height: `${heightPct}%`,
        display: "flex",
        alignItems: "center",
        overflow: "hidden",
        position: "relative",
      }}
    >
      <div
        ref={innerRef}
        style={{
          display: "inline-block",
          whiteSpace: "nowrap",
          color: config.text_color,
          fontSize: `${config.font_size}px`,
          fontFamily: config.font_family,
          fontWeight:
            config.font_family.includes("ZCOOL") || config.font_family.includes("Liu Jian")
              ? 400
              : 600,
          letterSpacing: "0.05em",
          position: "relative",
          left: 0,
          paddingLeft: "100%",
        }}
      >
        {content}
        <span style={{ padding: "0 80px" }}>{SEPARATOR}</span>
        {content}
        <span style={{ padding: "0 80px" }}>{SEPARATOR}</span>
        {content}
      </div>
    </div>
  );
}

function Marquee() {
  const [snapshot, setSnapshot] = useState<MarqueeSnapshot | null>(null);

  // State acquisition: push events + 1s polling. On hybrid-GPU machines the
  // webview hosted on the dGPU-driven monitor can silently stop receiving
  // Tauri events, so polling get_marquee_state is the reliable channel there.
  useEffect(() => {
    // Keep object identity when unchanged: a fresh snapshot every poll would
    // retrigger the scroll effects and restart the animations every second.
    const apply = (next: MarqueeSnapshot | null) =>
      setSnapshot((prev) =>
        JSON.stringify(prev) === JSON.stringify(next) ? prev : next,
      );

    const pull = () => {
      invoke<MarqueeSnapshot | null>("get_marquee_state", {
        dpr: window.devicePixelRatio,
        w: window.innerWidth,
        h: window.innerHeight,
      })
        .then(apply)
        .catch(() => {});
    };
    pull();
    const pollTimer = setInterval(pull, 1000);

    const unlisten = listen<MarqueeSnapshot>("marquee-state", (e) => apply(e.payload));
    return () => {
      clearInterval(pollTimer);
      unlisten.then((f) => f());
    };
  }, []);

  const config = snapshot?.config ?? DEFAULT_CONFIG;
  // Only active tracks are rendered. The container fills the whole window
  // (100vh) and the tracks split it evenly, rather than sizing content from
  // config.height: on mixed-DPI multi-monitor setups the backend's physical
  // pixel sizing and the webview's CSS viewport can disagree by a few pixels
  // (e.g. a 40px client holding 50px of content), which used to squash the
  // bar into a thin clipped line on the secondary monitor.
  const active = (snapshot?.tracks ?? []).filter((t): t is string => !!t);

  if (active.length === 0) return null;

  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        backgroundColor: bgWithOpacity(config.bg_color, config.opacity),
        overflow: "hidden",
        position: "relative",
      }}
    >
      {active.map((text, i) => (
        // Index in the key guards against duplicate texts on two tracks.
        <TrackRow
          key={`${i}:${text}`}
          text={text}
          config={config}
          heightPct={100 / active.length}
        />
      ))}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("marquee-root")!).render(
  <React.StrictMode>
    <Marquee />
  </React.StrictMode>,
);
