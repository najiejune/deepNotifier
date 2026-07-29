use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigFormat {
    Json,
    Toml,
}

#[derive(Debug, Clone)]
pub struct CliMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub _vendor: &'static str,
    pub stop_event: Option<&'static str>,
    pub notification_event: Option<&'static str>,
    pub pretool_event: Option<&'static str>,
    pub posttool_event: Option<&'static str>,
    pub config_paths: &'static [&'static str],
    pub config_format: ConfigFormat,
    pub plugin_create_dirs: &'static [&'static str],
    pub approval_timeout_supported: bool,
    pub binary_names: &'static [&'static str],
    pub sentinel_paths: &'static [&'static str],
}

pub fn all_cli_metas() -> Vec<CliMeta> {
    vec![
        CliMeta {
            id: "claude",
            name: "Claude Code",
            _vendor: "Anthropic",
            stop_event: Some("Stop"),
            notification_event: Some("Notification"),
            pretool_event: Some("PreToolUse"),
            posttool_event: Some("PostToolUse"),
            config_paths: &["~/.claude/settings.json"],
            config_format: ConfigFormat::Json,
            plugin_create_dirs: &[],
            approval_timeout_supported: true,
            binary_names: &["claude", "claude.exe"],
            sentinel_paths: &["~/.claude/settings.json"],
        },
        CliMeta {
            id: "codex",
            name: "Codex",
            _vendor: "OpenAI",
            stop_event: Some("Stop"),
            notification_event: None,
            pretool_event: Some("PreToolUse"),
            posttool_event: Some("PostToolUse"),
            config_paths: &["~/.codex/config.toml"],
            config_format: ConfigFormat::Toml,
            plugin_create_dirs: &[],
            approval_timeout_supported: true,
            binary_names: &["codex", "codex.exe"],
            sentinel_paths: &["~/.codex/"],
        },
        CliMeta {
            id: "opencode",
            name: "OpenCode",
            _vendor: "SST",
            stop_event: Some("session.idle"),
            notification_event: Some("tui.toast.show"),
            pretool_event: Some("tool.execute.before"),
            posttool_event: Some("tool.execute.after"),
            config_paths: &["opencode.json"],
            config_format: ConfigFormat::Json,
            plugin_create_dirs: &[".opencode/plugins/deepnotifier"],
            approval_timeout_supported: true,
            binary_names: &["opencode", "opencode.exe"],
            sentinel_paths: &["~/.config/opencode/"],
        },
        CliMeta {
            id: "gemini",
            name: "Gemini CLI",
            _vendor: "Google",
            stop_event: Some("SessionEnd"),
            notification_event: Some("Notification"),
            pretool_event: Some("BeforeTool"),
            posttool_event: Some("AfterTool"),
            config_paths: &["~/.gemini/settings.json"],
            config_format: ConfigFormat::Json,
            plugin_create_dirs: &[],
            approval_timeout_supported: true,
            binary_names: &["gemini", "gemini.exe"],
            sentinel_paths: &["~/.gemini/"],
        },
        CliMeta {
            id: "kiro",
            name: "Kiro",
            _vendor: "Amazon",
            stop_event: Some("stop"),
            notification_event: None,
            pretool_event: Some("preToolUse"),
            posttool_event: Some("postToolUse"),
            config_paths: &["kiro.json"],
            config_format: ConfigFormat::Json,
            plugin_create_dirs: &[],
            approval_timeout_supported: true,
            binary_names: &["kiro", "kiro.exe"],
            sentinel_paths: &["~/.kiro/"],
        },
        CliMeta {
            id: "codebuddy",
            name: "CodeBuddy",
            _vendor: "Tencent",
            stop_event: Some("Stop"),
            notification_event: Some("Notification"),
            pretool_event: Some("PreToolUse"),
            posttool_event: Some("PostToolUse"),
            config_paths: &["hooks/hooks.json"],
            config_format: ConfigFormat::Json,
            plugin_create_dirs: &["hooks"],
            approval_timeout_supported: true,
            binary_names: &["codebuddy", "codebuddy.exe", "cbc"],
            sentinel_paths: &["~/.codebuddy/"],
        },
        CliMeta {
            id: "qoder",
            name: "Qoder",
            _vendor: "QoderAI",
            stop_event: Some("Stop"),
            notification_event: Some("Notification"),
            pretool_event: Some("PreToolUse"),
            posttool_event: Some("PostToolUse"),
            config_paths: &["~/.qoder/settings.json"],
            config_format: ConfigFormat::Json,
            plugin_create_dirs: &[],
            approval_timeout_supported: true,
            binary_names: &["qoder", "qoder.exe"],
            sentinel_paths: &["~/.qoder/"],
        },
    ]
}

/// Check if a CLI tool appears to be installed.
/// Checks both the binary on PATH and sentinel config paths.
pub fn check_cli_installed(meta: &CliMeta, project_dir: &std::path::Path) -> bool {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // Check binaries on PATH
    for bin in meta.binary_names {
        let result = if cfg!(windows) {
            Command::new("where").arg(bin).output()
        } else {
            Command::new("which").arg(bin).output()
        };
        if let Ok(out) = result {
            if out.status.success() {
                return true;
            }
        }
    }

    // Check sentinel config paths
    for path in meta.sentinel_paths {
        let resolved = if let Some(rest) = path.strip_prefix("~/") {
            home.join(rest)
        } else if path.starts_with('~') {
            home.join(&path[1..])
        } else {
            project_dir.join(path)
        };
        if resolved.exists() {
            return true;
        }
    }

    false
}
