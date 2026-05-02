use crate::state::AppState;
use tauri::{Emitter, State};

#[tauri::command]
pub async fn toggle_dnd(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    let mut dnd = state.dnd_active.write().await;
    let old_state = *dnd;
    *dnd = !*dnd;
    let new_state = *dnd;
    drop(dnd); // Release lock before emitting

    tracing::info!("DND toggled: {} -> {}", old_state, new_state);

    // Notify frontend of DND state change
    let _ = app.emit("dnd-changed", new_state);

    Ok(new_state)
}

#[tauri::command]
pub async fn get_dnd_status(state: State<'_, AppState>) -> Result<bool, String> {
    let dnd = state.dnd_active.read().await;
    tracing::info!("DND status queried: {}", *dnd);
    Ok(*dnd)
}
