# deepNotifier Build Documentation

[中文](构建文档.md) | [English](build-guide.md)

## Project Structure

```
deepNotifier/
├── src/                    # Frontend source (React + TypeScript)
│   ├── main.tsx            # Entry point, React Router mounting
│   ├── App.tsx             # Routing + Provider composition
│   ├── index.css           # Tailwind CSS entry
│   ├── components/         # UI components
│   │   ├── layout/         # Layout (TitleBar, Sidebar, MainLayout)
│   │   ├── settings/       # Settings page panels
│   │   ├── notifications/  # Notification components
│   │   ├── timer/          # Pomodoro timer components
│   │   └── ui/             # Common UI (Button, Input, Select, Toggle)
│   ├── hooks/              # React Hooks (useConfig, useTimer, useTodos, etc.)
│   ├── pages/              # Pages (Dashboard, Pomodoro, History, Settings)
│   ├── i18n/               # Internationalization (zh / en)
│   ├── lib/                # Utility functions (cn, tauri invoke)
│   ├── types/              # TypeScript type definitions
│   ├── marquee/            # Standalone marquee (scrolling ticker) window
│   └── toast/              # Toast popup notification frontend components
├── src-tauri/              # Rust backend (Tauri 2)
│   ├── src/
│   │   ├── lib.rs          # Tauri Builder + Plugin registration
│   │   ├── main.rs         # Program entry point
│   │   ├── commands/       # Tauri commands (IPC)
│   │   ├── config/         # Config read/write (schema, persistence)
│   │   ├── webhook/        # Webhook server (GitHub/GitLab/Bitbucket/Custom)
│   │   ├── poller/         # Polling scheduler
│   │   ├── notifier/       # Notification dispatch (sound, marquee, toast, dispatcher)
│   │   ├── dnd/            # DND (Do Not Disturb) mode management
│   │   ├── timer/          # Pomodoro timer engine
│   │   ├── todo/           # Todo management (store, puller, server)
│   │   ├── history/        # Notification history
│   │   ├── tray/           # System tray
│   │   ├── state.rs        # Global state
│   │   └── error.rs        # Error types
│   ├── Cargo.toml          # Rust dependencies
│   ├── tauri.conf.json     # Tauri configuration
│   ├── build.rs            # Tauri build script
│   └── icons/              # Application icons
├── index.html              # Main window HTML entry
├── package.json            # Node dependencies and scripts
├── vite.config.ts          # Vite build configuration
├── tsconfig.json           # TypeScript configuration
└── docs/                   # Documentation
```

## Environment Requirements

| Tool | Minimum Version | Notes |
|------|----------|------|
| Node.js | >= 18 | Frontend build |
| Rust | Latest stable | Install via [rustup](https://rustup.rs/) |
| Windows | 10+ | Requires Visual Studio Build Tools (C++ desktop development) |
| macOS | 11+ | Requires Xcode Command Line Tools |
| Linux | — | Requires `libwebkit2gtk-4.1-dev` `libgtk-3-dev`, etc. |

Install the Rust toolchain:

```bash
rustup install stable
rustup target add wasm32-unknown-unknown  # Optional, WebAssembly target
```

Additional Linux dependencies (Debian/Ubuntu):

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev \
  libasound2-dev libssl-dev
```

## Development

### Install Dependencies

```bash
npm install
```

### Start Development Mode

```bash
npm run tauri dev
```

Execution flow:
1. `tauri dev` starts the Rust backend compilation (debug mode)
2. Starts the Vite dev server at the same time (`localhost:1420`)
3. The Tauri WebView loads the Vite dev server page
4. Frontend changes trigger hot module replacement (HMR); Rust changes trigger recompilation

### Frontend-Only Development

```bash
npm run dev        # Start Vite, open localhost:1420 in a browser
```

Note: Without the Tauri runtime, all `@tauri-apps/api` calls will fail.

### Rust Type Checking

```bash
cd src-tauri
cargo check       # Check only, no binary produced
cargo clippy      # Lint checks
```

## Building

### Frontend Build

```bash
npm run build      # Equivalent to tsc -b && vite build
```

Steps:
1. `tsc -b` — TypeScript type checking, generates `tsconfig.tsbuildinfo`
2. `vite build` — Rollup bundling, outputs to `dist/`

The Vite build produces two entry artifacts:
- `dist/index.html` → main window
- `dist/src/marquee/marquee.html` → marquee window

### Tauri Build (Full Packaging)

```bash
npm run tauri build
```

Execution flow:
1. Runs `beforeBuildCommand` — `npm run build`
2. Compiles Rust source code (release mode, `--release`)
3. Embeds `dist/` into the Tauri WebView
4. Generates platform installers according to the `bundle` configuration

Output path: `src-tauri/target/release/bundle/`

| Platform | Artifacts |
|------|------|
| Windows | `.msi` / `.nsis.exe` |
| macOS | `.dmg` / `.app` |
| Linux | `.deb` / `.rpm` / `.AppImage` |

### One-Click Build Script (Windows)

`build-release.bat` in the root directory wraps the full build environment initialization:

```bat
@echo off
set PATH=%USERPROFILE%\.cargo\bin;%PATH%
call "...VsDevCmd.bat" -arch=x64 -host_arch=x64
npm run tauri build
```

Suitable for running directly by double-clicking in a terminal without configured Rust/MSVC environment variables.

### Build Parameters

```bash
# Compile binary only (no installer)
npm run tauri build -- --bundles none

# Specify target platform (requires cross-compilation environment)
npm run tauri build -- --target x86_64-pc-windows-msvc

# Debug build
npm run tauri build -- --debug

# Specify installer format
npm run tauri build -- --bundles msi        # MSI only
npm run tauri build -- --bundles deb,appimage  # Multiple Linux formats
```

### Cross-Compilation

Building a Linux target on Windows:

```bash
rustup target add x86_64-unknown-linux-gnu
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

Building a Windows target on macOS:

```bash
rustup target add x86_64-pc-windows-msvc
# Requires the mingw-w64 cross-compilation toolchain
brew install mingw-w64
```

> Cross-compilation may run into linker errors; building natively on each target platform is recommended.

## Replacing Application Icons

Icon source file: `src-tauri/icons/icon-source.png` (1024×1024, transparent background).

```bash
# Generate the full icon set from the source image (overwrites src-tauri/icons/)
npx tauri icon src-tauri/icons/icon-source.png -o src-tauri/icons

# Key: Cargo incremental builds do not detect changes in icons/; you must
# force the build script to rerun, otherwise the exe still embeds the old icon
touch src-tauri/build.rs        # Windows Git Bash; or cargo clean -p deep-notifier

npm run tauri build
```

Notes:

- **Taskbar/tray icons** come from the runtime-loaded `icons/*.png` files and take effect immediately after replacement; **desktop shortcut icons** come from the exe's embedded resources (embedded by `build.rs` at compile time via `icon.ico`). The two come from different sources, so you must rebuild and reinstall.
- If the desktop icon still shows the old image after reinstalling, it is a Windows icon cache issue; restart Explorer (or run `ie4uinit.exe -ClearIconCache`) to refresh it.

## Configuration Reference

### Tauri Configuration (`src-tauri/tauri.conf.json`)

```jsonc
{
  "productName": "deepNotifier",
  "version": "0.1.0",
  "identifier": "com.deepnotifier.app",  // Bundle ID
  "build": {
    "beforeDevCommand": "npm run dev",     // Frontend start command in dev mode
    "devUrl": "http://localhost:1420",     // URL loaded in dev mode
    "beforeBuildCommand": "npm run build", // Run before build
    "frontendDist": "../dist"              // Frontend output directory (relative to src-tauri)
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "deepNotifier",
        "width": 1000, "height": 630,
        "minWidth": 864, "minHeight": 500,
        "decorations": false  // No native title bar; custom implementation
      },
      {
        "label": "marquee",
        "url": "src/marquee/marquee.html",
        "width": 1920, "height": 40,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "skipTaskbar": true
      }
    ]
    // Note: do not configure "trayIcon" in app — the config file would register
    // a tray icon with no menu, duplicating the TrayIconBuilder in code
    // (tray/menu.rs, id "main"), resulting in two tray icons
  },
  "bundle": {
    "active": true,
    "targets": "all",           // Generate installers in all formats
    "icon": [                   // Icons of various sizes
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### Vite Configuration (`vite.config.ts`)

- **Dev Server**: port `1420`, strict port
- **Path alias**: `@` → `src/`
- **Build entries**: dual entries — `index.html` (main window) + `src/marquee/marquee.html` (marquee window)
- **Plugins**: `@vitejs/plugin-react` + `@tailwindcss/vite`

### TypeScript Configuration (`tsconfig.json`)

- **Target**: ES2020
- **JSX**: `react-jsx` (automatic import)
- **Strict**: all enabled (`strict`, `noUnusedLocals`, `noUnusedParameters`)
- **Path mapping**: `@/*` → `src/*`

## Dependency List

### Frontend (npm)

| Package | Version | Purpose |
|------|------|------|
| react / react-dom | ^19 | UI framework |
| react-router-dom | ^7 | Routing |
| @tauri-apps/api | ^2 | Tauri IPC calls |
| @tauri-apps/plugin-notification | ^2 | Notification permission / auxiliary |
| @tauri-apps/plugin-dialog | ^2 | File dialogs |
| @tauri-apps/plugin-shell | ^2 | Shell operations |
| lucide-react | ^0.470 | Icon library |
| tailwindcss | ^4 | CSS framework |
| @tailwindcss/vite | ^4 | Tailwind Vite plugin |
| clsx | ^2 | Conditional class name composition |
| tailwind-merge | ^3 | Class name deduplication and merging |
| vite | ^6 | Build tool |

### Backend (Cargo)

| Package | Version | Purpose |
|------|------|------|
| tauri | 2 | Desktop framework (tray-icon, image-png) |
| tauri-plugin-notification | 2 | Notification permission / auxiliary |
| tauri-plugin-dialog | 2 | File dialogs |
| tauri-plugin-shell | 2 | Shell |
| axum | 0.8 | HTTP server (Webhook / Todo Push) |
| tokio | 1 (full) | Async runtime |
| reqwest | 0.12 | HTTP client (Poll) |
| serde / serde_json | 1 | Serialization |
| toml | 0.8 | TOML config file parsing |
| rodio | 0.20 | Audio playback |
| hmac / sha2 / hex | — | Webhook signature verification |
| chrono | 0.4 | Time handling |
| uuid | 1 (v4) | Unique ID generation |
| tracing / tracing-subscriber | 0.3 / 0.3 | Logging |
| dirs | 6 | System directories |
| thiserror | 2 | Error derive macro |

## Configuration and Data Storage

All configuration and user data are stored in the system application config directory:

| Platform | Path |
|------|------|
| Windows | `%APPDATA%\com.deepnotifier.app\` |
| macOS | `~/Library/Application Support/com.deepnotifier.app/` |
| Linux | `~/.config/com.deepnotifier.app/` |

Files:

| File | Format | Content |
|------|------|------|
| `config.toml` | TOML | All application settings |
| `todos.json` | JSON | Todo item list |
| `sounds/` | Directory | Imported custom audio files |

## Release Process

1. Update the version number:
   - `package.json` → `version`
   - `src-tauri/Cargo.toml` → `version`
   - `src-tauri/tauri.conf.json` → `version`

2. Update the changelog (e.g., `CHANGELOG.md`)

3. Commit and tag:
   ```bash
   git add -A && git commit -m "release v0.1.0"
   git tag v0.1.0
   git push origin main --tags
   ```

4. Build the installers:
   ```bash
   npm run tauri build
   ```

5. Upload the installers from `src-tauri/target/release/bundle/` to GitHub Releases
