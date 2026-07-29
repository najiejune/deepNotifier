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
    marquee::park_all(&app);
    Ok(())
}
