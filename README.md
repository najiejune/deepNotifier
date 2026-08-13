# deepNotifier

[English](README.md) | [中文](README.zh-CN.md)

Notification daemon for developers. Built with Rust + Tauri 2.

## Features

| Module | Description |
|--------|-------------|
| **Webhook** | Receive GitHub / GitLab / Bitbucket / custom webhook events |
| **Poll** | Periodically poll HTTP endpoints and parse JSON notifications |
| **CLI Hook** | Inject hooks into CLI AI assistants (Claude Code, Codex, Kimi Code, etc.) — get turn-finished, notification and approval-timeout alerts |
| **Notifications** | Sound, in-app toast popup (Bitbucket style), scrolling marquee |
| **Do Not Disturb** | Scheduled DND plans, weekly repeat, mutes notifications |
| **Pomodoro** | Pomodoro timer + todo tasks, with remote pull/push sync |
| **Marquee** | Multi-monitor support, danmaku-style multi-track (up to 3, configurable), queueing with Critical preemption, 5 preset themes, custom colors/fonts/icons |

## Requirements

- **Node.js** >= 18
- **Rust** latest stable
- **Windows**: Visual Studio Build Tools (Desktop development with C++)
- **macOS**: Xcode Command Line Tools
- **Linux**: `libwebkit2gtk`, `libgtk-3-dev`

## Quick Start

```bash
npm install
npm run tauri dev
```

## Usage

### Webhook

A local HTTP server receives platform webhook events and converts them into notifications.

**Default port**: `3927`

| Platform | Endpoint | Verification |
|----------|----------|--------------|
| GitHub | `POST /webhook/github` | HMAC-SHA256 (`X-Hub-Signature-256`) |
| GitLab | `POST /webhook/gitlab` | Token (`X-Gitlab-Token`) |
| Bitbucket | `POST /webhook/bitbucket` | HMAC-SHA256 (`X-Hub-Signature-256`) |
| Custom | `POST /webhook/custom` | None |

**Custom Webhook** — send any JSON, extract fields via dot-separated paths:

```bash
curl -X POST http://localhost:3927/webhook/custom \
  -H "Content-Type: application/json" \
  -d '{"title": "Deploy finished", "body": "Build #42 passed", "severity": "Info"}'
```

Configurable JSON paths (e.g. `repository.full_name`) and severity levels (Info / Warning / Critical).

### Poll

Periodically GET/POST an endpoint and parse JSON notifications from the response. Supports custom headers, request body, interval and timeout.

### CLI Hook

Injects hooks into CLI AI assistants (Claude Code, Codex, OpenCode, Gemini CLI, Kiro, CodeBuddy, Qoder, Kimi Code) so their events become notifications. Configure under **Settings → CLI Hook**: enable hook injection, set the webhook port, then install hooks per tool (or batch via "Install All Enabled").

| Alert | Trigger |
|-------|---------|
| **Stop** | The CLI finishes a turn |
| **Notification** | The CLI raises a notification needing attention |
| **Approval timeout** | An approval stays pending longer than the configured seconds |

Each event can independently toggle sound and marquee channels.

### Notification Channels

| Channel | Description |
|---------|-------------|
| **Sound** | Built-in ping / chime sounds, custom audio import supported |
| **Toast popup** | In-app Bitbucket-style toast card rendered at the corner of the work area (never covers the taskbar), semi-transparent (reuses the marquee opacity), pure SVG severity icons, queued display |
| **Marquee** | Top/bottom scrolling bar, danmaku-style multi-track (1–3 tracks, default 2), queueing with Critical preemption, synced across monitors, 5 preset themes |

**Toast duration is configurable per severity** (Settings → Notification):

| Severity | Duration |
|----------|----------|
| Info | `toast_info_secs` seconds (default 10) |
| Warning | `toast_warning_secs` seconds (default 10) |
| Critical | `toast_critical_secs` seconds (default 0) |

Set any of them to `0` to make that level **stay until manually dismissed**.

### Do Not Disturb (DND)

- Manual toggle + scheduled plans
- Weekly repeat, multiple time ranges per day
- All channels (sound, marquee, toast) are muted together; notifications are still received and recorded

### Pomodoro

1. Add tasks with due dates on the "Todo" page
2. Configure work/break duration and rounds per task
3. Start focusing; completed sessions are recorded automatically

### Marquee Themes

| Theme | Style |
|-------|-------|
| **Poster** | Dark background + red text, Impact font |
| **Anime** | Purple background + gold text, playful font |
| **Business** | Deep blue background + white text, serif font |
| **Kawaii** | Pink background + magenta text, handwriting font |
| **Transparent** | Transparent background + white text, minimal |

Customizable: position (top/bottom), track count, speed, height, font size, font family, prefix/suffix icons, background color, text color, background opacity, display duration.

## Build

```bash
# TypeScript type check + Vite build
npm run build

# Build installer for the current platform
npm run tauri build

# Compile binary only
npm run tauri build -- --bundles none

# Specific bundle format
npm run tauri build -- --bundles msi
```

Artifacts: `src-tauri/target/release/bundle/`

| Platform | Artifacts |
|----------|-----------|
| Windows | `.msi` / `.nsis.exe` |
| macOS | `.dmg` / `.app` |
| Linux | `.deb` / `.rpm` / `.AppImage` |

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite 6 + Tailwind CSS 4
- **Backend**: Rust + Tauri 2 + axum + tokio
- **Plugins**: tauri-plugin-notification, tauri-plugin-shell, tauri-plugin-dialog
