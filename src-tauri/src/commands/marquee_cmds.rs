use crate::config::schema::MarqueeConfig;
use crate::notifier::marquee;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn show_marquee(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    text: String,
) -> Result<(), String> {
    let marquee_cfg: MarqueeConfig = {
        let config = state.config.read().await;
        config.marquee.clone()
    };

    // Shared with the notification dispatcher path: per-monitor placement,
    // push + pull state, timed parking (never hide, see marquee::PARK_POS).
    marquee::show(&app, &text, &marquee_cfg);

    Ok(())
}

#[tauri::command]
pub async fn hide_marquee(app: tauri::AppHandle) -> Result<(), String> {
    marquee::clear_and_park(&app);
    Ok(())
}

/// Re-apply the current marquee config to on-screen bars without enqueuing a
/// new item (live restyle while the settings preview is visible).
#[tauri::command]
pub async fn refresh_marquee_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let marquee_cfg: MarqueeConfig = {
        let config = state.config.read().await;
        config.marquee.clone()
    };
    marquee::refresh_config(&app, &marquee_cfg);
    Ok(())
}
