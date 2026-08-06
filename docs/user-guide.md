# deepNotifier User Guide

[中文](使用文档.md) | [English](user-guide.md)

## Introduction

deepNotifier is a desktop notification daemon for developers. It receives notifications via Webhooks (push) and polling (pull), and provides three alerting methods: sound, scrolling marquee, and toast popup notifications. Built-in Pomodoro timer and task management help you stay focused without missing important notifications.

**Supported platforms**: Windows / macOS / Linux

---

## Installation & Launch

### Installation

Download the installer for your platform from the [Releases](https://github.com/najiejune/deepNotifier/releases) page:

| Platform | Package format |
|----------|----------------|
| Windows | `.msi` / `.exe` |
| macOS | `.dmg` |
| Linux | `.deb` / `.rpm` / `.AppImage` |

### Launch

After installation, double-click the app icon to launch. The app appears in the system tray, and the main window opens automatically.

The tray menu provides three options:
- **Show deepNotifier** — bring up the main window
- **Do Not Disturb** — quickly toggle DND mode
- **Quit** — fully exit the app

---

## Main Interface Overview

```
┌──────────────────────────────────────────────────┐
│  📻 deepNotifier │ [Mode]               ─  □  ✕  │  ← custom title bar
├────────┬─────────────────────────────────────────┤
│ Notification Center │                             │
│ Todo Tasks │             Content Area             │
│ History    │                                       │
│ Settings   │                                       │
└────────┴─────────────────────────────────────────┘
```

- **Left navigation bar**: entries to the four pages — Notification Center, Todo Tasks, History, Settings
- **Title bar**: the app name is shown on the left (width aligned with the sidebar, the divider continues the right border of the navigation bar below); window control buttons are on the right. The mode status button sits to the right of the divider
- **Close button**: hides to the tray by default instead of quitting the app

---

## Notification Center (Home)

The home page contains:

### Live Push

Monitors incoming notification events in real time. Each new notification automatically appears here with a brief highlight animation.

### Recent Notifications

Shows the full list of received notifications in reverse chronological order.

---

## Notification Source Configuration

### Method 1: Webhook (Push Mode)

The app runs a local HTTP server that receives webhook events from GitHub / GitLab / Bitbucket / custom sources.

#### Configuration Steps

1. Go to **Settings → Webhook**
2. Enable "Enable Webhook Server"
3. Set the port (default `3927`)
4. (Optional) Set a secret for signature verification
5. Select the event types to receive

#### GitHub Webhook Configuration

Add the following in your GitHub repository's **Settings → Webhooks**:

| Field | Value |
|-------|-------|
| Payload URL | `http://<yourIP>:3927/webhook/github` |
| Content type | `application/json` |
| Secret | The secret you set in deepNotifier |
| Events | As needed (push, pull_request, issues, etc.) |

#### GitLab Webhook Configuration

Add the following in your GitLab project's **Settings → Webhooks**:

| Field | Value |
|-------|-------|
| URL | `http://<yourIP>:3927/webhook/gitlab` |
| Secret Token | The secret you set in deepNotifier |
| Triggers | As needed (Push, Merge Request, Issues, etc.) |

#### Bitbucket Webhook Configuration

Add the following in your Bitbucket repository's **Settings → Webhooks**:

| Field | Value |
|-------|-------|
| URL | `http://<yourIP>:3927/webhook/bitbucket` |
| Secret | The secret you set in deepNotifier |
| Events | As needed (repo:push, pullrequest:created, etc.) |

#### Custom Webhook

Send any JSON to `/webhook/custom`; the title and body are extracted via dot-separated paths:

```bash
curl -X POST http://localhost:3927/webhook/custom \
  -H "Content-Type: application/json" \
  -d '{"title":"Deploy finished","body":"Build #42 passed","severity":"Info"}'
```

You can customize the title field path, body field path, and severity level in Settings.

> If deepNotifier is deployed locally, you need a tunneling tool (such as ngrok) to expose the local port to the public internet.

### Method 2: Polling (Pull Mode)

Periodically fetches data from specified HTTP endpoints, detects changes via content hashing, and sends notifications.

#### Configuration Steps

1. Go to **Settings → Polling**
2. Enable "Enable Polling"
3. Click "Add Endpoint"
4. Fill in the endpoint details:
   - **Endpoint Name**: a recognizable name
   - **URL**: the API address
   - **Interval (seconds)**: polling frequency
   - **Request Method**: GET or POST
   - **Request Headers**: custom HTTP headers
   - **POST Body**: request payload (for POST method)

---

## Notification Alert Methods

Configure under **Settings → Notifications**:

### Sound Alert

Plays a sound when a notification arrives.

- **Sound file**: 4 built-in sounds — ping, chime, bell, alarm
- **Volume**: 0% ~ 100%

### Scrolling Marquee

When a notification arrives, a scrolling marquee bar appears at the top or bottom of the screen.

- Multi-monitor support: displayed independently on each screen
- The marquee bar is transparent and always-on-top, and does not block mouse operations
- Multi-track (danmaku-style): up to 3 tracks scrolling different notifications simultaneously; the bar height adapts to the actual number of active notifications
- Notifications arriving at the same time are automatically queued and played in turn without overlapping; Critical notifications immediately preempt the display, and interrupted notifications automatically resume afterwards
- Appearance can be fine-tuned under **Settings → Marquee**

### Toast Popup Notification

When a notification arrives, an in-app Bitbucket-style popup card appears in the corner of the screen. Operating system native notifications (Windows Toast / macOS / Linux libnotify) are no longer used.

- Light blue background (`#DEEBFF`), semi-transparent (the opacity reuses the marquee opacity setting)
- Pure SVG severity icons: Info / Warning / Critical
- Shown in the corner of the screen work area, without covering the taskbar
- Multiple notifications are automatically queued and shown in turn; click ✕ to close manually

Popup display duration is configured per severity level (**Settings → Notifications**):

| Severity | Default duration |
|----------|------------------|
| Info | 10 seconds |
| Warning | 10 seconds |
| Critical | 0 seconds |

> A duration of 0 means the card never auto-dismisses and can only be closed manually via ✕.

---

## Do Not Disturb (DND) Mode

Suppress alerts during specified time periods.

### Manual Toggle

- Click the "DND" button in the title bar
- Or use the "Do Not Disturb" option in the tray menu

### Scheduled Plans

Go to **Settings → Do Not Disturb** to add plans:

1. Enable "Enable DND Mode"
2. Click "Add Schedule"
3. Configure:
   - **Name**: e.g. "Lunch Break", "Evening"
   - **Time Range**: start time ~ end time
   - **Active Days**: select Monday through Sunday

> Note: when DND mode is on, all three alert methods — sound, scrolling marquee, and toast popup — are muted together. Notifications are still received normally and stored in the history.

---

## Marquee Settings

Go to **Settings → Marquee**:

| Setting | Description | Default |
|---------|-------------|---------|
| Preview | Live preview of the marquee effect when enabled | Off |
| Presets | 5 one-click styles | — |
| Position | Top or bottom of the screen | Top |
| Duration | Seconds each message displays on each track | 30 |
| Tracks | Maximum number of marquee messages scrolling simultaneously (1~3); idle tracks take no screen space | 2 |
| Speed | Scrolling speed | 100 |
| Height | Height of a single track (pixels); total bar height = height × current number of notifications | 40 |
| Font | 9 Chinese/English fonts available | System Sans |
| Font Size | Text size (pixels) | 16 |
| Prefix/Suffix Icons | Emoji decorations before and after the text | — |
| Background Color | Supports transparency (enter `transparent`) | `#1e3a5f` |
| Text Color | — | `#ffffff` |
| Background Opacity | 0% ~ 100%; larger values make the background more transparent while the text stays opaque; 100% means a fully transparent background | 10% |

### Presets

| Preset | Style |
|--------|-------|
| Poster | Dark background + red text + Impact font |
| Anime & Games | Purple background + gold text + ZCOOL font |
| Business | Deep blue background + white text + Noto Serif |
| Cute | Pink background + magenta text + Comic Sans |
| Minimal Transparent | Transparent background + white text + Noto Sans |

Modifying any parameter of a preset counts as a custom style.

---

## Pomodoro Settings

Go to **Settings → Pomodoro** to configure the global parameters of the Pomodoro technique.

The Pomodoro timer follows a **Work → Short Break → Work → Short Break → ... → Long Break** phase structure that loops automatically. It is tied to tasks: every completed work round adds one 🍅 to the current task.

| Setting | Description | Default |
|---------|-------------|---------|
| Work Duration | Minutes of focused work per round | 25 |
| Short Break Duration | Minutes of a short break | 1 |
| Long Break Duration | Minutes of a long break | 0 (long break disabled) |
| Rounds | Number of short-break rounds before a long break | 4 |
| Sound | Sound played when the timer ends | chime |
| Auto-start Break | Automatically start the break countdown after work ends | Off |
| Auto-start Work | Automatically start the work countdown after a break ends | Off |

> A long break duration of 0 skips the long break, leaving only the work → short-break loop. With both auto-starts enabled, the Pomodoro timer runs continuously without manual operation.

---

## Pomodoro & Task Management

Go to the **Todo Tasks** page:

### Task Management

1. Enter a task in the "Task List" and press Enter or click "Add Task"
2. A due date can be set
3. Filtering supported: All / Today / This Week
4. Click a task to set it as the current focus target

### Configuring the Pomodoro Timer

Click the edit button next to a task to configure its Pomodoro parameters (work duration, short break duration, long break duration, rounds). Global defaults are configured under **Settings → Pomodoro**.

### Action Buttons

| Button | Function |
|--------|----------|
| ▶ Start | Start the timer |
| ⏸ Pause | Pause the timer |
| ▶ Resume | Resume the timer |
| ⏹ Stop | Finish early (does not count towards the pomodoro count) |

If you click "Finish Early" midway, this round does not count towards the pomodoro count. Only a naturally completed timer adds a 🍅 to the current task.

---

## History

Go to the **History** page:

- **Search**: search by title or content
- **Severity filter**: Info / Warning / Critical
- **Source filter**: GitHub / GitLab / Bitbucket / Custom / Polling Endpoint / Pomodoro
- **Date range**: filter by time period
- **Clear All**: delete all history records
- **Pagination**: shows the current item count and total item count

---

## General Settings

Go to **Settings → General**:

| Setting | Description |
|---------|-------------|
| Language | English / 中文 (instant UI switching) |
| Notification Mode | Push / Pull / Both |
| Run on Startup | Automatically run when the system starts |
| Minimize to Tray | Minimize to the tray instead of the taskbar |
| Close to Tray | Close button hides to the tray instead of quitting |

---

## Todo Sync

Go to **Settings → Todo**:

### Pull Todos

Periodically pull the todo list from a remote server:

1. Enable "Enable Pull"
2. Fill in the pull URL (response format: `{ "todos": [...] }`)
3. Set the pull interval (seconds)
4. Select the request method (GET / POST)

### Push Todos

Accept pushed todos over HTTP:

1. Enable "Enable Push Server"
2. Set the push port (default `3928`)
3. Third parties push todos via `POST http://localhost:3928/todos`

---

## Configuration Storage

All configuration is stored in the system application config directory:

- **Windows**: `%APPDATA%/com.deepnotifier.app/`
- **macOS**: `~/Library/Application Support/com.deepnotifier.app/`
- **Linux**: `~/.config/com.deepnotifier.app/`

File descriptions:

| File | Format | Content |
|------|--------|---------|
| `config.toml` | TOML | All application settings |
| `todos.json` | JSON | Todo list |

---

## Shortcuts & Actions

| Action | Description |
|--------|-------------|
| Click the close button ✕ | Hide to tray |
| Double-click the tray icon | Show the main window |

---

## FAQ

### Q: How do I fully quit the app?

A: Right-click the system tray icon → click "Quit".

### Q: A notification arrived but no sound played?

A: Check: 1) whether sound is enabled in notification settings; 2) whether DND mode is active; 3) whether the system volume is normal.

### Q: The scrolling marquee is not showing?

A: Check: 1) whether the marquee is enabled in notification settings; 2) whether DND mode is active; 3) click "Preview" in marquee settings to test.

### Q: The marquee only shows on the primary screen with multiple monitors?

A: deepNotifier automatically creates an independent marquee window for each monitor. If the marquee does not appear on a secondary screen, restart the app and try again.

### Q: Webhooks are not receiving notifications?

A: Make sure: 1) the webhook server is enabled; 2) the port is not blocked by a firewall; 3) if deepNotifier is on an intranet, configure tunneling (e.g. ngrok).

### Q: How do I reset all settings?

A: Go to **Settings** → click "Reset" in the top-right corner → click "Confirm".
