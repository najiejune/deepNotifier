use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

mod commands;
mod config;
mod dnd;
mod error;
mod history;
mod notifier;
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
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize app state
            let config_dir = app
                .path()
                .app_config_dir()
                .expect("Failed to resolve app config dir");
            std::fs::create_dir_all(&config_dir).ok();

            let app_config = config::persistence::load_or_create(&config_dir);
            let (tx, rx) = tokio::sync::mpsc::channel::<notifier::dispatcher::NotificationEvent>(256);

            let todo_store = todo::store::TodoStore::new(&config_dir);

            // Configure the statically-defined marquee window (primary monitor)
            if let Some(marquee_win) = app.get_webview_window("marquee") {
                let _ = marquee_win.set_ignore_cursor_events(true);
            }

            // Create additional marquee windows for extra monitors
            if let Ok(monitors) = app.available_monitors() {
                for (i, monitor) in monitors.iter().skip(1).enumerate() {
                    let size = monitor.size();
                    let pos = monitor.position();
                    let sf = monitor.scale_factor();
                    let label = format!("marquee-{}", i + 1);
                    let builder = tauri::WebviewWindowBuilder::new(
                        app,
                        &label,
                        tauri::WebviewUrl::App("src/marquee/marquee.html".into()),
                    )
                    .title("")
                    .inner_size(size.width as f64 / sf, 40.0)
                    .position(pos.x as f64 / sf, pos.y as f64 / sf)
                    .decorations(false)
                    .transparent(true)
                    .always_on_top(true)
                    .skip_taskbar(true)
                    .visible(false)
                    .focused(false)
                    .resizable(false);

                    match builder.build() {
                        Ok(win) => {
                            let _ = win.set_ignore_cursor_events(true);
                        }
                        Err(e) => {
                            eprintln!("Failed to create marquee window for monitor: {}", e);
                        }
                    }
                }
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
            commands::timer_cmds::stop_timer,
            commands::timer_cmds::pause_timer,
            commands::timer_cmds::get_timer_state,
            commands::timer_cmds::start_pomodoro,
            commands::dnd_cmds::toggle_dnd,
            commands::dnd_cmds::get_dnd_status,
            commands::marquee_cmds::show_marquee,
            commands::marquee_cmds::hide_marquee,
            commands::todo_cmds::get_todos,
            commands::todo_cmds::add_todo,
            commands::todo_cmds::toggle_todo,
            commands::todo_cmds::delete_todo,
            commands::sound_cmds::list_sounds,
            commands::sound_cmds::import_sound,
            commands::sound_cmds::preview_sound,
        ])
        .run(tauri::generate_context!())
        .expect("error while running deepNotifier");
}
