import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";

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

function Marquee() {
  const [text, setText] = useState("");
  const [config, setConfig] = useState<MarqueeConfig>({
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
  });

  useEffect(() => {
    const u1 = listen<string>("marquee-text", (e) => setText(e.payload));
    const u2 = listen<MarqueeConfig>("marquee-config", (e) => setConfig(e.payload));
    return () => {
      u1.then((f) => f());
      u2.then((f) => f());
    };
  }, []);

  if (!text) return null;

  const separator = "  •  ";
  const content =
    (config.icon_before ? config.icon_before + " " : "") +
    text +
    (config.icon_after ? " " + config.icon_after : "");
  const duration = content.length * (100 / config.speed);

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
      }}
    >
      <div
        style={{
          display: "inline-block",
          whiteSpace: "nowrap",
          color: config.text_color,
          fontSize: `${config.font_size}px`,
          fontFamily: config.font_family,
          fontWeight: config.font_family.includes("ZCOOL") ||
            config.font_family.includes("Liu Jian")
            ? 400
            : 600,
          letterSpacing: "0.05em",
          animation: `scroll ${duration}s linear infinite`,
          paddingLeft: "100%",
        }}
      >
        {content}
        <span style={{ padding: "0 80px" }}>{separator}</span>
        {content}
        <span style={{ padding: "0 80px" }}>{separator}</span>
        {content}
      </div>
      <style>{`
        @keyframes scroll {
          0% { transform: translateX(0); }
          100% { transform: translateX(-66.66%); }
        }
      `}</style>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("marquee-root")!).render(
  <React.StrictMode>
    <Marquee />
  </React.StrictMode>,
);
