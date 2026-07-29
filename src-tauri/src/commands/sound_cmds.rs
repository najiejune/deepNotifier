use crate::notifier::embedded_sounds;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_sounds(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let sounds_dir = state.config_dir.join("sounds");
    let mut builtins = vec![
        "ping".to_string(),
        "chime".to_string(),
        "bell".to_string(),
        "alarm".to_string(),
    ];
    builtins.extend(embedded_sounds::klaudio_sound_names());

    let mut sounds: Vec<String> = builtins;

    if let Ok(entries) = std::fs::read_dir(&sounds_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext.to_lowercase().as_str(), "wav" | "mp3" | "ogg" | "flac") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    if !sounds.contains(&name.to_string()) {
                        sounds.push(name.to_string());
                    }
                }
            }
        }
    }

    Ok(sounds)
}

#[tauri::command]
pub async fn import_sound(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    let sounds_dir = state.config_dir.join("sounds");
    std::fs::create_dir_all(&sounds_dir).map_err(|e| e.to_string())?;

    let src = std::path::Path::new(&path);
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("wav");
    let name = src
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("imported");
    let filename = format!("{}.{}", name, ext);
    let dest = sounds_dir.join(&filename);

    std::fs::copy(src, &dest).map_err(|e| e.to_string())?;

    Ok(name.to_string())
}

#[tauri::command]
pub async fn preview_sound(state: State<'_, AppState>, sound_file: String) -> Result<(), String> {
    let sounds_dir = state.config_dir.join("sounds");
    let volume = state.config.read().await.notification.sound_volume;
    crate::notifier::sound::play(&sound_file, &sounds_dir, volume);
    Ok(())
}
