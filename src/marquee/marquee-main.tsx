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
}

const DEFAULT_CONFIG: MarqueeConfig = {
  position: "Top",
  speed: 80,
  height: 40,
  font_size: 16,
  font_family: "sans-serif",
  icon_before: "",
  icon_after: "",
  bg_color: "#1e3a5f",
  text_color: "#ffffff",
  opacity: 0.9,
  duration_secs: 10,
};

function Marquee() {
  const [text, setText] = useState("");
  const [config, setConfig] = useState<MarqueeConfig>(DEFAULT_CONFIG);
  const innerRef = useRef<HTMLDivElement>(null);

  // State acquisition: push events + 1s polling. On hybrid-GPU machines the
  // webview hosted on the dGPU-driven monitor can silently stop receiving
  // Tauri events, so polling get_marquee_state is the reliable channel there.
  useEffect(() => {
    const pull = () => {
      invoke<[string, MarqueeConfig] | null>("get_marquee_state", {
        dpr: window.devicePixelRatio,
        w: window.innerWidth,
        h: window.innerHeight,
      })
        .then((state) => {
          if (state) {
            // Keep object identity when unchanged: a fresh config object every
            // poll would retrigger the scroll effect and restart the
            // animation every second.
            setText((prev) => (prev === state[0] ? prev : state[0]));
            setConfig((prev) =>
              JSON.stringify(prev) === JSON.stringify(state[1]) ? prev : state[1],
            );
          }
        })
        .catch(() => {});
    };
    pull();
    const pollTimer = setInterval(pull, 1000);

    const u1 = listen<string>("marquee-text", (e) => setText(e.payload));
    const u2 = listen<MarqueeConfig>("marquee-config", (e) => setConfig(e.payload));
    return () => {
      clearInterval(pollTimer);
      u1.then((f) => f());
      u2.then((f) => f());
    };
  }, []);

  const separator = "  •  ";
  const content =
    (config.icon_before ? config.icon_before + " " : "") +
    text +
    (config.icon_after ? " " + config.icon_after : "");
  const duration = content.length * (100 / config.speed);

  // JS-driven scrolling via `left` (NO CSS transform/animation). CSS
  // transform animations get their own compositor layer, which fails to
  // composite on the dGPU-driven monitor of hybrid-GPU laptops: the strip
  // background paints but the animated text layer never appears. Updating
  // `left` on an interval stays on the plain paint path and renders anywhere.
  useEffect(() => {
    if (!text) return;
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
  }, [text, config, content, duration]);

  if (!text) return null;

  return (
    <div
      style={{
        width: "100vw",
        height: `${config.height}px`,
        backgroundColor: config.bg_color === "transparent" ? "transparent" : config.bg_color,
        opacity: config.opacity,
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
        <span style={{ padding: "0 80px" }}>{separator}</span>
        {content}
        <span style={{ padding: "0 80px" }}>{separator}</span>
        {content}
      </div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("marquee-root")!).render(
  <React.StrictMode>
    <Marquee />
  </React.StrictMode>,
);
