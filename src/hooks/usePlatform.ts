import { useState } from "react";

// Tauri v2 ships no `platform()` in @tauri-apps/api (it moved to plugin-os,
// whose Rust side this app doesn't include), so detect the mobile webviews
// via user agent instead: Android System WebView and iOS WKWebView both
// identify themselves, and desktop WebView2 never matches these tokens.
function detectPlatform(): string {
  const ua = navigator.userAgent;
  if (/Android/i.test(ua)) return "android";
  // iPadOS 13+ can report a desktop "Macintosh" UA; the "Mobile/" token
  // still gives it away.
  if (/iPhone|iPad|iPod/i.test(ua) || (/Macintosh/i.test(ua) && /Mobile\//i.test(ua))) return "ios";
  if (/Windows/i.test(ua)) return "windows";
  if (/Mac OS X/i.test(ua)) return "macos";
  if (/Linux/i.test(ua)) return "linux";
  return "unknown";
}

// Pure and constant for the app's lifetime — compute once, not per consumer.
const detected = detectPlatform();

export function usePlatform() {
  const [platformName] = useState(detected);
  return {
    platform: platformName,
    isMobile: platformName === "android" || platformName === "ios",
  };
}
