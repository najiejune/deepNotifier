use std::fs;
use std::path::{Path, PathBuf};

use crate::config::schema::CliToolConfig;
use crate::hook::cli_configs::{CliMeta, ConfigFormat};
use crate::hook::command_gen::{generate_webhook_command, GeneratedCommand};

const INJECT_MARKER: &str = "deepNotifier";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HookInstallResult {
    pub cli_id: String,
    pub success: bool,
    pub message: String,
    pub config_path: String,
    pub events_injected: Vec<String>,
}

/// Resolve config path placeholders (~ and relative paths)
fn resolve_config_path(template: &str, project_dir: &Path, home_dir: &Path) -> PathBuf {
    if let Some(rest) = template.strip_prefix("~/") {
        home_dir.join(rest)
    } else if template.starts_with('~') {
        home_dir.join(&template[1..])
    } else {
        project_dir.join(template)
    }
}

/// Check current OS is Windows
fn is_windows() -> bool {
    std::env::consts::OS == "windows"
}

/// Pick the right command for current platform
fn pick_command(cmd: &GeneratedCommand) -> &str {
    if is_windows() {
        &cmd.windows
    } else {
        &cmd.unix
    }
}

pub fn install_hooks(
    meta: &CliMeta,
    _tool: &CliToolConfig,
    port: u16,
    project_dir: &Path,
    approval_timeout_enabled: bool,
    approval_timeout_secs: u32,
) -> HookInstallResult {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut events_injected: Vec<String> = Vec::new();

    // Find the first existing config path, or the first path (to create)
    let mut chosen_template: Option<&str> = None;
    let mut chosen_path: Option<PathBuf> = None;

    for template in meta.config_paths {
        let resolved = resolve_config_path(template, project_dir, &home_dir);
        if resolved.exists() {
            chosen_template = Some(template);
            chosen_path = Some(resolved);
            break;
        }
        if chosen_path.is_none() {
            chosen_template = Some(template);
            chosen_path = Some(resolved);
        }
    }

    let (template, config_path) = match (chosen_template, chosen_path) {
        (Some(t), Some(p)) => (t, p),
        _ => {
            return HookInstallResult {
                cli_id: meta.id.to_string(),
                success: false,
                message: "No config path defined".into(),
                config_path: String::new(),
                events_injected: vec![],
            }
        }
    };

    // Create parent directories
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // Create plugin directories if needed
    for plugin_dir in meta.plugin_create_dirs {
        let resolved = resolve_config_path(plugin_dir, project_dir, &home_dir);
        fs::create_dir_all(&resolved).ok();
    }

    // Backup existing config
    if config_path.exists() {
        let bak_path = config_path.with_extension(
            config_path
                .extension()
                .map(|e| format!("{}.bak", e.to_string_lossy()))
                .unwrap_or_else(|| "bak".into()),
        );
        fs::copy(&config_path, &bak_path).ok();
    }

    // Generate commands for each event type
    let mut stop_cmd: Option<String> = None;
    let mut notif_cmd: Option<String> = None;
    let mut pretool_cmd: Option<String> = None;
    let mut posttool_cmd: Option<String> = None;

    if let Some(stop_event) = meta.stop_event {
        let cmd = generate_webhook_command(port, meta.id, meta.name, "stop", approval_timeout_secs);
        stop_cmd = Some(pick_command(&cmd).to_string());
        events_injected.push(stop_event.to_string());
    }

    if let Some(notif_event) = meta.notification_event {
        let cmd = generate_webhook_command(port, meta.id, meta.name, "notification", approval_timeout_secs);
        notif_cmd = Some(pick_command(&cmd).to_string());
        events_injected.push(notif_event.to_string());
    }

    if meta.approval_timeout_supported && approval_timeout_enabled {
        if let Some(pretool_event) = meta.pretool_event {
            let cmd = generate_webhook_command(port, meta.id, meta.name, "pretooluse", approval_timeout_secs);
            pretool_cmd = Some(pick_command(&cmd).to_string());
            events_injected.push(pretool_event.to_string());
        }
        if let Some(posttool_event) = meta.posttool_event {
            let cmd = generate_webhook_command(port, meta.id, meta.name, "posttooluse", approval_timeout_secs);
            posttool_cmd = Some(pick_command(&cmd).to_string());
            events_injected.push(posttool_event.to_string());
        }
    }

    // Perform the injection based on config format and CLI type
    let result = match meta.config_format {
        ConfigFormat::Json => inject_json(
            meta.id, &config_path, stop_cmd.as_deref(), notif_cmd.as_deref(),
            pretool_cmd.as_deref(), posttool_cmd.as_deref(),
            meta,
        ),
        ConfigFormat::Toml => inject_toml(
            meta.id, &config_path, stop_cmd.as_deref(), notif_cmd.as_deref(),
            pretool_cmd.as_deref(), posttool_cmd.as_deref(),
            meta,
        ),
    };

    match result {
        Ok(()) => HookInstallResult {
            cli_id: meta.id.to_string(),
            success: true,
            message: format!("Injected {} events into {}", events_injected.len(), template),
            config_path: config_path.to_string_lossy().to_string(),
            events_injected,
        },
        Err(e) => HookInstallResult {
            cli_id: meta.id.to_string(),
            success: false,
            message: e,
            config_path: config_path.to_string_lossy().to_string(),
            events_injected,
        },
    }
}

pub fn uninstall_hooks(meta: &CliMeta, project_dir: &Path) -> HookInstallResult {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // Find the config file
    let mut config_path: Option<PathBuf> = None;
    let mut _template_used: Option<&str> = None;

    for template in meta.config_paths {
        let resolved = resolve_config_path(template, project_dir, &home_dir);
        if resolved.exists() {
            config_path = Some(resolved);
            _template_used = Some(template);
            break;
        }
    }

    let config_path = match config_path {
        Some(p) => p,
        None => {
            return HookInstallResult {
                cli_id: meta.id.to_string(),
                success: false,
                message: "Config file not found".into(),
                config_path: String::new(),
                events_injected: vec![],
            }
        }
    };

    // Restore from backup if available
    let bak_path = config_path.with_extension(
        config_path
            .extension()
            .map(|e| format!("{}.bak", e.to_string_lossy()))
            .unwrap_or_else(|| "bak".into()),
    );

    if bak_path.exists() {
        if fs::copy(&bak_path, &config_path).is_ok() {
            let _ = fs::remove_file(&bak_path);
            return HookInstallResult {
                cli_id: meta.id.to_string(),
                success: true,
                message: "Restored from backup".into(),
                config_path: config_path.to_string_lossy().to_string(),
                events_injected: vec![],
            };
        }
    }

    // If no backup, try to remove injected entries from the file
    let result = match meta.config_format {
        ConfigFormat::Json => remove_json_hooks(meta.id, &config_path, meta),
        ConfigFormat::Toml => remove_toml_hooks(meta.id, &config_path),
    };

    match result {
        Ok(removed) => HookInstallResult {
            cli_id: meta.id.to_string(),
            success: true,
            message: format!("Removed {} hook entries", removed),
            config_path: config_path.to_string_lossy().to_string(),
            events_injected: vec![],
        },
        Err(e) => HookInstallResult {
            cli_id: meta.id.to_string(),
            success: false,
            message: e,
            config_path: config_path.to_string_lossy().to_string(),
            events_injected: vec![],
        },
    }
}

// ---- JSON injection ----

fn inject_json(
    cli_id: &str,
    config_path: &Path,
    stop_cmd: Option<&str>,
    notif_cmd: Option<&str>,
    pretool_cmd: Option<&str>,
    posttool_cmd: Option<&str>,
    meta: &CliMeta,
) -> Result<(), String> {
    let content = if config_path.exists() {
        fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?
    } else {
        String::from("{}")
    };

    let mut root: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON config: {}", e))?;

    // Handle OpenCode / CodeBuddy plugin format (create separate plugin.json)
    if !meta.plugin_create_dirs.is_empty() {
        return inject_json_plugin(cli_id, config_path, stop_cmd, notif_cmd, pretool_cmd, posttool_cmd, meta);
    }

    // For Qoder: hooks are in "hooks" key, use Claude Code format
    // For Gemini CLI: hooks are in "hooks" key but use "command" format
    // For Kiro: hooks are at top level, use Claude Code format
    match cli_id {
        "gemini" => inject_json_gemini(&mut root, stop_cmd, notif_cmd, pretool_cmd, posttool_cmd, meta)?,
        _ => inject_json_standard(&mut root, stop_cmd, notif_cmd, pretool_cmd, posttool_cmd, meta)?,
    }

    let new_content = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    fs::write(config_path, new_content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

/// Standard format (Claude Code, Qoder, Kiro):
/// hooks: { EventName: [ { matcher: "", hooks: [ { type: "command", command: "..." } ] } ] }
fn inject_json_standard(
    root: &mut serde_json::Value,
    stop_cmd: Option<&str>,
    notif_cmd: Option<&str>,
    pretool_cmd: Option<&str>,
    posttool_cmd: Option<&str>,
    meta: &CliMeta,
) -> Result<(), String> {
    let hooks = root
        .as_object_mut()
        .ok_or("Config root is not an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

    let hooks_obj = hooks
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    inject_standard_event(hooks_obj, meta.stop_event, stop_cmd);
    inject_standard_event(hooks_obj, meta.notification_event, notif_cmd);
    inject_standard_event(hooks_obj, meta.pretool_event, pretool_cmd);
    inject_standard_event(hooks_obj, meta.posttool_event, posttool_cmd);

    Ok(())
}

fn inject_standard_event(
    hooks_obj: &mut serde_json::Map<String, serde_json::Value>,
    event_name: Option<&str>,
    command: Option<&str>,
) {
    let event_name = match event_name {
        Some(e) => e,
        None => return,
    };
    let command = match command {
        Some(c) => c,
        None => return,
    };

    // Create the hook entry
    let entry = serde_json::json!({
        "matcher": "",
        "hooks": [{
            "type": "command",
            "command": command,
            INJECT_MARKER: true
        }],
        INJECT_MARKER: true
    });

    let arr = hooks_obj
        .entry(event_name)
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));

    if let Some(arr) = arr.as_array_mut() {
        // Remove existing deepNotifier entries for this event
        arr.retain(|v| !v.get(INJECT_MARKER).is_some());
        arr.push(entry);
    }
}

/// Gemini CLI format: hooks: { EventName: [ { command: "..." } ] }
fn inject_json_gemini(
    root: &mut serde_json::Value,
    stop_cmd: Option<&str>,
    notif_cmd: Option<&str>,
    pretool_cmd: Option<&str>,
    posttool_cmd: Option<&str>,
    meta: &CliMeta,
) -> Result<(), String> {
    let hooks = root
        .as_object_mut()
        .ok_or("Config root is not an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

    let hooks_obj = hooks
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    inject_gemini_event(hooks_obj, meta.stop_event, stop_cmd);
    inject_gemini_event(hooks_obj, meta.notification_event, notif_cmd);
    inject_gemini_event(hooks_obj, meta.pretool_event, pretool_cmd);
    inject_gemini_event(hooks_obj, meta.posttool_event, posttool_cmd);

    Ok(())
}

fn inject_gemini_event(
    hooks_obj: &mut serde_json::Map<String, serde_json::Value>,
    event_name: Option<&str>,
    command: Option<&str>,
) {
    let event_name = match event_name {
        Some(e) => e,
        None => return,
    };
    let command = match command {
        Some(c) => c,
        None => return,
    };

    let entry = serde_json::json!({
        "command": command,
        INJECT_MARKER: true
    });

    let arr = hooks_obj
        .entry(event_name)
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));

    if let Some(arr) = arr.as_array_mut() {
        arr.retain(|v| !v.get(INJECT_MARKER).is_some());
        arr.push(entry);
    }
}

/// Plugin format (OpenCode, CodeBuddy): create a separate plugin.json
fn inject_json_plugin(
    cli_id: &str,
    _config_path: &Path,
    stop_cmd: Option<&str>,
    notif_cmd: Option<&str>,
    pretool_cmd: Option<&str>,
    posttool_cmd: Option<&str>,
    meta: &CliMeta,
) -> Result<(), String> {
    use std::fs;

    // Build plugin manifest
    let mut hooks_obj = serde_json::Map::new();

    // OpenCode uses { event: { ... } }, CodeBuddy uses [ { event: "...", command: "..." } ]
    match cli_id {
        "opencode" => {
            inject_standard_event(&mut hooks_obj, meta.stop_event, stop_cmd);
            inject_standard_event(&mut hooks_obj, meta.notification_event, notif_cmd);
            inject_standard_event(&mut hooks_obj, meta.pretool_event, pretool_cmd);
            inject_standard_event(&mut hooks_obj, meta.posttool_event, posttool_cmd);

            let plugin = serde_json::json!({
                "name": INJECT_MARKER,
                "description": format!("deepNotifier hook integration for {}", meta.name),
                "version": "1.0.0",
                "hooks": hooks_obj
            });

            // Write to .opencode/plugins/deepnotifier/plugin.json
            // We need project_dir — use the plugin dir from meta
            // The plugin dir was already created in install_hooks
            // We need to resolve it here...
            // For now, write relative to the config path's parent
            let home = dirs::home_dir().unwrap_or_default();
            let project_dir = _config_path.parent().unwrap_or(Path::new("."));
            for dir in meta.plugin_create_dirs {
                let resolved = if let Some(rest) = dir.strip_prefix("~/") {
                    home.join(rest)
                } else {
                    project_dir.join(dir)
                };
                let plugin_file = resolved.join("plugin.json");
                if let Some(parent) = plugin_file.parent() {
                    fs::create_dir_all(parent).ok();
                }
                let content = serde_json::to_string_pretty(&plugin)
                    .map_err(|e| format!("Failed to serialize plugin JSON: {}", e))?;
                fs::write(&plugin_file, content)
                    .map_err(|e| format!("Failed to write plugin: {}", e))?;
            }
        }
        "codebuddy" => {
            // CodeBuddy uses array format: [ { event: "Stop", command: "...", deepNotifier: true } ]
            let mut entries: Vec<serde_json::Value> = Vec::new();

            if let Some(cmd) = stop_cmd {
                entries.push(serde_json::json!({
                    "event": meta.stop_event.unwrap(),
                    "command": cmd,
                    INJECT_MARKER: true
                }));
            }
            if let Some(cmd) = notif_cmd {
                entries.push(serde_json::json!({
                    "event": meta.notification_event.unwrap(),
                    "command": cmd,
                    INJECT_MARKER: true
                }));
            }
            if let Some(cmd) = pretool_cmd {
                entries.push(serde_json::json!({
                    "event": meta.pretool_event.unwrap(),
                    "command": cmd,
                    INJECT_MARKER: true
                }));
            }
            if let Some(cmd) = posttool_cmd {
                entries.push(serde_json::json!({
                    "event": meta.posttool_event.unwrap(),
                    "command": cmd,
                    INJECT_MARKER: true
                }));
            }

            let plugin = serde_json::json!({
                "name": INJECT_MARKER,
                "description": format!("deepNotifier hook integration for {}", meta.name),
                "version": "1.0.0",
                "hooks": entries
            });

            let home = dirs::home_dir().unwrap_or_default();
            let project_dir = _config_path.parent().unwrap_or(Path::new("."));
            for dir in meta.plugin_create_dirs {
                let resolved = if let Some(rest) = dir.strip_prefix("~/") {
                    home.join(rest)
                } else {
                    project_dir.join(dir)
                };
                let plugin_file = resolved.join("plugin.json");
                if let Some(parent) = plugin_file.parent() {
                    fs::create_dir_all(parent).ok();
                }
                let content = serde_json::to_string_pretty(&plugin)
                    .map_err(|e| format!("Failed to serialize plugin JSON: {}", e))?;
                fs::write(&plugin_file, content)
                    .map_err(|e| format!("Failed to write plugin: {}", e))?;
            }
        }
        _ => {}
    }

    Ok(())
}

// ---- TOML injection (Codex) ----

fn inject_toml(
    _cli_id: &str,
    config_path: &Path,
    stop_cmd: Option<&str>,
    notif_cmd: Option<&str>,
    pretool_cmd: Option<&str>,
    posttool_cmd: Option<&str>,
    meta: &CliMeta,
) -> Result<(), String> {
    let content = if config_path.exists() {
        fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?
    } else {
        String::new()
    };

    let mut root: toml::Value = if content.trim().is_empty() {
        toml::Value::Table(toml::Table::new())
    } else {
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse TOML config: {}", e))?
    };

    let hooks_table = root
        .as_table_mut()
        .ok_or("TOML root is not a table")?
        .entry("hooks")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));

    let hooks = hooks_table
        .as_table_mut()
        .ok_or("hooks is not a table")?;

    // Codex TOML format: [hooks] stop = ["cmd1", "cmd2"], pre_tool_use = [...]
    // Map event names to TOML key names
    if let Some(cmd) = stop_cmd {
        inject_toml_event(hooks, meta.stop_event.unwrap_or("stop"), cmd);
    }
    if let Some(cmd) = notif_cmd {
        inject_toml_event(hooks, meta.notification_event.unwrap_or("notification"), cmd);
    }
    if let Some(cmd) = pretool_cmd {
        inject_toml_event(hooks, meta.pretool_event.unwrap_or("pre_tool_use"), cmd);
    }
    if let Some(cmd) = posttool_cmd {
        inject_toml_event(hooks, meta.posttool_event.unwrap_or("post_tool_use"), cmd);
    }

    let new_content = toml::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize TOML: {}", e))?;
    fs::write(config_path, new_content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

fn inject_toml_event(
    hooks: &mut toml::Table,
    event_key: &str,
    command: &str,
) {
    // Codex uses lowercase underscored keys: "pre_tool_use", "post_tool_use", "stop"
    let key = event_key.to_lowercase().replace('.', "_");

    let arr = hooks
        .entry(&key)
        .or_insert_with(|| toml::Value::Array(Vec::new()));

    if let Some(arr) = arr.as_array_mut() {
        // Remove existing deepNotifier commands (those containing INJECT_MARKER in comment)
        arr.retain(|v| {
            if let Some(s) = v.as_str() {
                !s.contains(INJECT_MARKER)
            } else {
                true
            }
        });
        // Add a comment marker by prepending to the command string
        // TOML doesn't support per-value comments natively, so we add inline
        let tagged = format!("#deepNotifier\n{}", command);
        arr.push(toml::Value::String(tagged));
    }
}

// ---- Removal functions ----

fn remove_json_hooks(
    cli_id: &str,
    config_path: &Path,
    meta: &CliMeta,
) -> Result<usize, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    let mut root: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON config: {}", e))?;

    let mut removed = 0;

    // Also clean up plugin directories
    for plugin_dir in meta.plugin_create_dirs {
        let home = dirs::home_dir().unwrap_or_default();
        let project_dir = config_path.parent().unwrap_or(Path::new("."));
        let resolved = if let Some(rest) = plugin_dir.strip_prefix("~/") {
            home.join(rest)
        } else {
            project_dir.join(plugin_dir)
        };
        let plugin_file = resolved.join("plugin.json");
        if plugin_file.exists() {
            fs::remove_file(&plugin_file).ok();
            removed += 1;
        }
    }

    let hooks = match root.get_mut("hooks") {
        Some(h) => h,
        None => {
            // Write back unchanged
            let new_content = serde_json::to_string_pretty(&root)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
            fs::write(config_path, new_content).ok();
            return Ok(removed);
        }
    };

    match cli_id {
        "gemini" => {
            if let Some(obj) = hooks.as_object_mut() {
                for (_event, val) in obj.iter_mut() {
                    if let Some(arr) = val.as_array_mut() {
                        let before = arr.len();
                        arr.retain(|v| !v.get(INJECT_MARKER).is_some());
                        removed += before - arr.len();
                    }
                }
            }
        }
        _ => {
            if let Some(obj) = hooks.as_object_mut() {
                for (_event, val) in obj.iter_mut() {
                    if let Some(arr) = val.as_array_mut() {
                        let before = arr.len();
                        arr.retain(|v| !v.get(INJECT_MARKER).is_some());
                        removed += before - arr.len();
                    }
                }
            }
        }
    }

    let new_content = serde_json::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    fs::write(config_path, new_content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(removed)
}

fn remove_toml_hooks(
    _cli_id: &str,
    config_path: &Path,
) -> Result<usize, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    let mut root: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse TOML: {}", e))?;

    let mut removed = 0;

    if let Some(hooks) = root.get_mut("hooks") {
        if let Some(table) = hooks.as_table_mut() {
            for (_key, val) in table.iter_mut() {
                if let Some(arr) = val.as_array_mut() {
                    let before = arr.len();
                    arr.retain(|v| {
                        if let Some(s) = v.as_str() {
                            !s.starts_with("#deepNotifier")
                        } else {
                            true
                        }
                    });
                    removed += before - arr.len();
                }
            }
        }
    }

    let new_content = toml::to_string_pretty(&root)
        .map_err(|e| format!("Failed to serialize TOML: {}", e))?;
    fs::write(config_path, new_content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(removed)
}
