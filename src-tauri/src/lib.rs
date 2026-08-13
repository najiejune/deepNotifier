use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

mod commands;
mod config;
mod dnd;
mod error;
mod history;
mod hook;
pub mod notifier;
mod poller;
mod state;
mod timer;
mod todo;
mod tray;
mod webhook;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing for debug output
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize app state
            let config_dir = app
                .path()
                .app_config_dir()
                .expect("Failed to resolve app config dir");
            std::fs::create_dir_all(&config_dir).ok();

            // Extract built-in Klaudio sounds to config sounds directory
            notifier::embedded_sounds::extract_to(&config_dir.join("sounds"));

            let app_config = config::persistence::load_or_create(&config_dir);

            // Sync the OS auto-launch registration with the saved setting
            // (e.g. after a reinstall the registry entry may be missing).
            {
                use tauri_plugin_autostart::ManagerExt;
                let autostart = app.autolaunch();
                let want = app_config.general.run_on_startup;
                if autostart.is_enabled().unwrap_or(false) != want {
                    let r = if want {
                        autostart.enable()
                    } else {
                        autostart.disable()
                    };
                    if let Err(e) = r {
                        tracing::warn!("Failed to sync autostart registration: {}", e);
                    }
                }
            }
            let (tx, rx) = tokio::sync::mpsc::channel::<notifier::dispatcher::NotificationEvent>(256);

            let todo_store = todo::store::TodoStore::new(&config_dir);

            // Configure the statically-defined marquee window (primary monitor).
            // Marquee windows are never hidden: they are parked off-screen and
            // kept visible so WebView2 never suspends their pages
            // (see notifier::marquee::PARK_POS).
            if let Some(marquee_win) = app.get_webview_window("marquee") {
                notifier::toast::strip_window_frame(&marquee_win);
                let _ = marquee_win.set_ignore_cursor_events(true);
                let _ = marquee_win.show();
            }

            // Create additional marquee windows for extra monitors
            if let Ok(monitors) = app.available_monitors() {
                let primary = app.primary_monitor().ok().flatten();
                for (i, monitor) in monitors.iter().enumerate() {
                    tracing::info!(
                        "Monitor {}: name={:?} pos={:?} size={:?} scale={}",
                        i,
                        monitor.name(),
                        monitor.position(),
                        monitor.size(),
                        monitor.scale_factor()
                    );
                }
                let mut extra_idx = 0;
                for monitor in monitors.iter() {
                    // Only create dynamic windows for non-primary monitors;
                    // enumeration order is not guaranteed to put primary first.
                    let is_primary = primary.as_ref().is_some_and(|p| {
                        p.position() == monitor.position() && p.size() == monitor.size()
                    });
                    if is_primary {
                        continue;
                    }
                    extra_idx += 1;
                    let size = monitor.size();
                    let pos = monitor.position();
                    let sf = monitor.scale_factor();
                    let label = format!("marquee-{}", extra_idx);
                    let builder = tauri::WebviewWindowBuilder::new(
                        app,
                        &label,
                        tauri::WebviewUrl::App("src/marquee/marquee.html".into()),
                    )
                    .title("")
                    .inner_size(size.width as f64 / sf, 40.0)
                    .position(pos.x as f64 / sf, pos.y as f64 / sf)
                    .decorations(false)
                    // Transparent on secondary monitors too. Previously opaque
                    // out of concern that layered WebView2 windows fail to
                    // composite on displays driven by a different GPU; verified
                    // working on a hybrid-GPU laptop (iGPU primary + dGPU
                    // external, 2026-08) — bar and text render, alpha applies.
                    // If a machine ever shows only a bare frame here, revert
                    // to `.transparent(false)`.
                    .transparent(true)
                    .shadow(false)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .visible(false)
                    .focusable(false)
                    .resizable(false);

                    match builder.build() {
                        Ok(win) => {
                            tracing::info!("Created marquee window '{}' at {:?}", label, pos);
                            notifier::toast::strip_window_frame(&win);
                            let _ = win.set_ignore_cursor_events(true);
                            // Move it onto its own monitor first (physical
                            // coords) so it adopts that monitor's DPI context.
                            let _ = win.set_position(tauri::PhysicalPosition::new(pos.x, pos.y));
                            let _ = win.show();
                        }
                        Err(e) => {
                            tracing::error!("Failed to create marquee window for monitor: {}", e);
                        }
                    }
                }
            }

            // Park every marquee window below its ASSIGNED monitor (keeps each
            // window's DPI context pinned to its own screen).
            notifier::marquee::park_all(&app.handle());

            // Create the toast window (custom notification cards, primary
            // monitor bottom-right). Same lifecycle rules as the marquee
            // windows: never hidden, parked off-screen so WebView2 never
            // suspends the page. Unlike the marquee it must receive clicks
            // (card body focuses the source terminal, × dismisses), so
            // ignore_cursor_events stays OFF; focusable(false) keeps it from
            // stealing focus when clicked.
            match tauri::WebviewWindowBuilder::new(
                app,
                "toast",
                tauri::WebviewUrl::App("src/toast/toast.html".into()),
            )
            .title("")
            .inner_size(
                notifier::toast::WIDTH_LOGICAL + 2.0 * notifier::toast::PAD_LOGICAL,
                notifier::toast::CARD_H_LOGICAL + 2.0 * notifier::toast::PAD_LOGICAL,
            )
            .decorations(false)
            .transparent(true)
            // No DWM undecorated shadow: on old Windows builds (Win10 LTSC
            // 1809) it renders as a hard-edged rectangle around the card.
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .focusable(false)
            .resizable(false)
            .build()
            {
                Ok(win) => {
                    notifier::toast::strip_window_frame(&win);
                    let _ = win.show();
                    notifier::marquee::park_marquee_window(&win);
                    tracing::info!("Created toast window");
                }
                Err(e) => {
                    tracing::error!("Failed to create toast window: {}", e);
                }
            }

            tracing::info!(
                "Marquee windows at startup: {:?}",
                app.webview_windows().keys().collect::<Vec<_>>()
            );

            let cli_installed_cache: state::CliStatusCache =
                Arc::new(RwLock::new(std::collections::HashMap::new()));

            // Pre-warm CLI installation status cache in the background
            {
                let cache = cli_installed_cache.clone();
                let cwd = std::env::current_dir().unwrap_or_else(|_| config_dir.clone());
                tauri::async_runtime::spawn(async move {
                    let metas = hook::cli_configs::all_cli_metas();
                    let mut map = std::collections::HashMap::new();
                    for meta in &metas {
                        let installed = hook::cli_configs::check_cli_installed(meta, &cwd);
                        map.insert(meta.id.to_string(), installed);
                    }
                    *cache.write().await = map;
                    tracing::info!("CLI installation status cache warmed");
                });
            }

            let approval_sessions: state::ApprovalSessions =
                Arc::new(RwLock::new(std::collections::HashMap::new()));

            // Clean up stale marker files from previous runs
            if let Some(home) = dirs::home_dir() {
                let approval_dir = home.join(".deepnotifier").join("approval");
                let _ = std::fs::remove_dir_all(&approval_dir);
            }

            // Spawn a periodic cleanup for stale approval sessions
            {
                let sessions = approval_sessions.clone();
                tauri::async_runtime::spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(120)).await;
                        sessions.write().await.retain(|_, s| {
                            s.start.elapsed().as_secs() < 600 // 10 min max
                        });
                    }
                });
            }

            let state = AppState {
                config: Arc::new(RwLock::new(app_config)),
                config_dir,
                notification_tx: tx,
                notification_rx: Arc::new(RwLock::new(Some(rx))),
                dnd_active: Arc::new(RwLock::new(false)),
                history: Arc::new(RwLock::new(history::store::NotificationHistory::new(500))),
                timer_state: Arc::new(RwLock::new(timer::engine::TimerState::default())),
                timer_cancel: Arc::new(RwLock::new(None)),
                todo_store: Arc::new(RwLock::new(todo_store)),
                cli_installed_cache,
                pending_pid: Arc::new(RwLock::new(None)),
                approval_sessions,
            };
            app.manage(state.clone());

            // Setup system tray
            tray::menu::create_tray(app)?;

            // Start notification dispatcher
            notifier::dispatcher::start(app.handle().clone(), state.clone());

            // Start webhook server if Push mode enabled
            let mode = state.config.blocking_read().general.mode.clone();

            if matches!(mode, config::schema::NotificationMode::Push | config::schema::NotificationMode::Both) {
                webhook::server::start(app.handle().clone(), state.clone());
            }

            if matches!(mode, config::schema::NotificationMode::Pull | config::schema::NotificationMode::Both) {
                poller::scheduler::start(state.clone());
            }

            // Start todo pull scheduler
            {
                let cfg = state.config.blocking_read();
                if cfg.todo.pull_enabled {
                    drop(cfg);
                    todo::puller::start_scheduler(
                        app.handle().clone(),
                        state.config.clone(),
                        state.todo_store.clone(),
                    );
                } else {
                    drop(cfg);
                }
            }

            // Start todo push server
            {
                let cfg = state.config.blocking_read();
                if cfg.todo.push_enabled {
                    let port = cfg.todo.push_port;
                    drop(cfg);
                    let handle = app.handle().clone();
                    let store = state.todo_store.clone();
                    tauri::async_runtime::spawn(async move {
                        todo::server::start(handle, store, port).await;
                    });
                } else {
                    drop(cfg);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config_cmds::get_config,
            commands::config_cmds::save_config,
            commands::config_cmds::reset_config,
            commands::config_cmds::get_host_ip,
            commands::config_cmds::get_wan_ip,
            commands::notification_cmds::get_notifications,
            commands::notification_cmds::clear_notifications,
            commands::notification_cmds::focus_pending_pid,
            commands::notification_cmds::debug_focus_pid,
            commands::notification_cmds::debug_get_pending_pid,
            commands::timer_cmds::stop_timer,
            commands::timer_cmds::pause_timer,
            commands::timer_cmds::get_timer_state,
            commands::timer_cmds::start_pomodoro,
            commands::dnd_cmds::toggle_dnd,
            commands::dnd_cmds::get_dnd_status,
            commands::marquee_cmds::show_marquee,
            commands::marquee_cmds::hide_marquee,
            commands::marquee_cmds::refresh_marquee_config,
            notifier::marquee::get_marquee_state,
            notifier::toast::get_toast_state,
            notifier::toast::toast_dismiss,
            notifier::toast::toast_activate,
            notifier::toast::toast_keepalive,
            notifier::toast::toast_preview,
            commands::todo_cmds::get_todos,
            commands::todo_cmds::add_todo,
            commands::todo_cmds::toggle_todo,
            commands::todo_cmds::delete_todo,
            commands::sound_cmds::list_sounds,
            commands::sound_cmds::import_sound,
            commands::sound_cmds::preview_sound,
            commands::hook_cmds::install_hooks,
            commands::hook_cmds::uninstall_hooks,
            commands::hook_cmds::check_cli_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running deepNotifier");
}
