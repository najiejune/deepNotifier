use std::io::Write;
use std::path::Path;

pub const KLAUDIO_SOUNDS: &[(&str, &[u8])] = &[
    ("klaudio-minimal-zen-notification", include_bytes!("../../resources/sounds/klaudio-minimal-zen-notification.wav")),
    ("klaudio-minimal-zen-stop", include_bytes!("../../resources/sounds/klaudio-minimal-zen-stop.wav")),
    ("klaudio-retro-8bit-notification", include_bytes!("../../resources/sounds/klaudio-retro-8bit-notification.wav")),
    ("klaudio-retro-8bit-stop", include_bytes!("../../resources/sounds/klaudio-retro-8bit-stop.wav")),
    ("klaudio-sci-fi-terminal-notification", include_bytes!("../../resources/sounds/klaudio-sci-fi-terminal-notification.wav")),
    ("klaudio-sci-fi-terminal-stop", include_bytes!("../../resources/sounds/klaudio-sci-fi-terminal-stop.wav")),
    ("klaudio-victory-fanfare-notification", include_bytes!("../../resources/sounds/klaudio-victory-fanfare-notification.wav")),
    ("klaudio-victory-fanfare-stop", include_bytes!("../../resources/sounds/klaudio-victory-fanfare-stop.wav")),
];

/// Extract embedded Klaudio WAV files to the config sounds directory.
/// Skips any file that already exists.
pub fn extract_to(sounds_dir: &Path) {
    if let Err(e) = std::fs::create_dir_all(sounds_dir) {
        tracing::error!("Failed to create sounds dir: {}", e);
        return;
    }
    let mut written = 0usize;
    let mut skipped = 0usize;
    for &(name, data) in KLAUDIO_SOUNDS {
        let dest = sounds_dir.join(format!("{}.wav", name));
        if dest.exists() {
            skipped += 1;
            continue;
        }
        match std::fs::File::create(&dest) {
            Ok(mut f) => {
                if let Err(e) = f.write_all(data) {
                    tracing::error!("Failed to write {}: {}", name, e);
                } else {
                    written += 1;
                }
            }
            Err(e) => tracing::error!("Failed to create {}: {}", name, e),
        }
    }
    if written > 0 {
        tracing::info!("Extracted {} Klaudio sound(s) to {}", written, sounds_dir.display());
    }
    if skipped > 0 {
        tracing::debug!("{} Klaudio sound(s) already present", skipped);
    }
}

pub fn klaudio_sound_names() -> Vec<String> {
    KLAUDIO_SOUNDS.iter().map(|(n, _)| n.to_string()).collect()
}
