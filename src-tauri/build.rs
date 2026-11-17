fn main() {
    embed_default_config();
    tauri_build::build()
}

/// Compile-time default config embedding: when DEEPNOTIFIER_DEFAULT_CONFIG
/// points at a TOML file, its contents are baked into the binary (read via
/// include_str! from OUT_DIR) and used as the first-run/reset defaults.
/// The file is syntax-checked here so a bad default fails the build.
fn embed_default_config() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    let out_path = std::path::Path::new(&out_dir).join("embedded_default_config.toml");

    println!("cargo:rerun-if-env-changed=DEEPNOTIFIER_DEFAULT_CONFIG");

    let contents = match std::env::var("DEEPNOTIFIER_DEFAULT_CONFIG") {
        Ok(path) if !path.trim().is_empty() => {
            println!("cargo:rerun-if-changed={}", path);
            let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!(
                    "DEEPNOTIFIER_DEFAULT_CONFIG={} could not be read: {}",
                    path, e
                )
            });
            if let Err(e) = toml::from_str::<toml::Value>(&text) {
                panic!(
                    "DEEPNOTIFIER_DEFAULT_CONFIG={} is not valid TOML: {}",
                    path, e
                );
            }
            text
        }
        _ => String::new(),
    };

    std::fs::write(&out_path, contents).expect("write embedded default config");
}
