use std::path::PathBuf;
use tracing::error;

/// Play a notification sound. If `sound_file` maps to an imported file, play it.
/// Otherwise generate a synthetic chime.
pub fn play(sound_file: &str, sounds_dir: &PathBuf, volume: f32) {
    let sound_file = sound_file.to_string();
    let sounds_dir = sounds_dir.clone();
    std::thread::spawn(move || {
        if let Err(e) = try_play(&sound_file, &sounds_dir, volume) {
            error!("Audio playback failed: {}", e);
        }
    });
}

fn try_play(sound_file: &str, sounds_dir: &PathBuf, volume: f32) -> Result<(), String> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default()
        .map_err(|e| format!("No audio output: {}", e))?;

    let sink = rodio::Sink::try_new(&stream_handle)
        .map_err(|e| format!("Failed to create sink: {}", e))?;

    sink.set_volume(volume.clamp(0.0, 1.0));

    // Try to play an imported audio file
    let file_played = try_play_file(&sink, sound_file, sounds_dir);

    if !file_played {
        // Fallback: synthetic chime
        let sample_rate = 44100u32;
        let samples = match sound_file {
            "ping" => generate_ping(sample_rate),
            "bell" => generate_bell(sample_rate),
            "alarm" => generate_alarm(sample_rate),
            _ => generate_chime(sample_rate),
        };
        let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, samples);
        sink.append(source);
    }

    sink.sleep_until_end();
    drop(_stream);
    Ok(())
}

fn try_play_file(sink: &rodio::Sink, name: &str, sounds_dir: &PathBuf) -> bool {
    for ext in &["wav", "mp3", "ogg", "flac"] {
        let path = sounds_dir.join(format!("{}.{}", name, ext));
        if path.exists() {
            if let Ok(file) = std::fs::File::open(&path) {
                if let Ok(decoder) = rodio::Decoder::new(std::io::BufReader::new(file)) {
                    sink.append(decoder);
                    return true;
                }
            }
        }
    }
    false
}

fn generate_chime(sample_rate: u32) -> Vec<i16> {
    let mut samples = Vec::new();
    append_tone(&mut samples, sample_rate, 523.25, 0.2, 0.4);
    append_tone(&mut samples, sample_rate, 0.0, 0.05, 0.0);
    append_tone_with_fade(&mut samples, sample_rate, 659.25, 0.35, 0.4);
    samples
}

fn generate_ping(sample_rate: u32) -> Vec<i16> {
    let mut samples = Vec::new();
    // Short high-pitched ping: E6 (1318Hz)
    for i in 0..2 {
        append_tone_with_fade(&mut samples, sample_rate, 1318.5, 0.15, 0.3);
        if i == 0 {
            append_tone(&mut samples, sample_rate, 0.0, 0.08, 0.0);
        }
    }
    samples
}

fn generate_bell(sample_rate: u32) -> Vec<i16> {
    let mut samples = Vec::new();
    // Rich bell: C5 -> E5 -> G5 with overlap
    append_tone_with_fade(&mut samples, sample_rate, 523.25, 0.3, 0.25);
    append_tone_with_fade(&mut samples, sample_rate, 659.25, 0.25, 0.2);
    append_tone_with_fade(&mut samples, sample_rate, 783.99, 0.35, 0.15);
    samples
}

fn generate_alarm(sample_rate: u32) -> Vec<i16> {
    let mut samples = Vec::new();
    // Alternating alarm: A4 (440Hz) and A5 (880Hz)
    for i in 0..4 {
        let freq = if i % 2 == 0 { 440.0 } else { 880.0 };
        append_tone(&mut samples, sample_rate, freq, 0.2, 0.35);
    }
    samples
}

fn append_tone(samples: &mut Vec<i16>, sample_rate: u32, freq: f32, duration_secs: f32, amplitude: f32) {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let val = (2.0 * std::f32::consts::PI * freq * t).sin() * amplitude;
        samples.push((val * i16::MAX as f32) as i16);
    }
}

fn append_tone_with_fade(samples: &mut Vec<i16>, sample_rate: u32, freq: f32, duration_secs: f32, amplitude: f32) {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let fade_start = (num_samples as f32 * 0.6) as usize;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let fade = if i > fade_start {
            1.0 - (i - fade_start) as f32 / (num_samples - fade_start) as f32
        } else {
            1.0
        };
        let val = (2.0 * std::f32::consts::PI * freq * t).sin() * amplitude * fade;
        samples.push((val * i16::MAX as f32) as i16);
    }
}
