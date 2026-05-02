use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, AppHandle, Emitter, Manager,
};

pub fn create_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<crate::state::AppState>();
    let config = state.config.blocking_read();
    let language = config.general.language.clone();
    drop(config);

    let labels = tray_labels(&language);
    let show = MenuItem::with_id(app, "show", labels.show, true, None::<&str>)?;
    let dnd = MenuItem::with_id(app, "dnd", labels.dnd, false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &dnd, &quit])?;

    let app_handle = app.handle().clone();

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("deepNotifier")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "dnd" => {
                // Toggle DND state directly
                if let Some(state) = app.try_state::<crate::state::AppState>() {
                    let state = state.inner().clone();
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let mut dnd = state.dnd_active.write().await;
                        *dnd = !*dnd;
                        let new_state = *dnd;
                        drop(dnd);

                        // Notify frontend of DND state change
                        let _ = handle.emit("dnd-changed", new_state);
                    });
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

struct TrayLabels {
    show: &'static str,
    dnd: &'static str,
    quit: &'static str,
}

fn tray_labels(language: &str) -> TrayLabels {
    if language == "zh" {
        TrayLabels {
            show: "显示 deepNotifier",
            dnd: "勿扰模式",
            quit: "退出",
        }
    } else {
        TrayLabels {
            show: "Show deepNotifier",
            dnd: "Do Not Disturb",
            quit: "Quit",
        }
    }
}

pub fn rebuild_tray(app: &AppHandle, language: &str) -> Result<(), Box<dyn std::error::Error>> {
    let labels = tray_labels(language);
    let show = MenuItem::with_id(app, "show", labels.show, true, None::<&str>)?;
    let dnd = MenuItem::with_id(app, "dnd", labels.dnd, false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &dnd, &quit])?;

    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu))?;
    }

    Ok(())
}
