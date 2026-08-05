use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub webhook: WebhookConfig,
    pub poll: PollConfig,
    pub notification: NotificationConfig,
    pub dnd: DndConfig,
    pub timer: TimerConfig,
    pub marquee: MarqueeConfig,
    pub todo: TodoConfig,
    pub hook: HookConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            webhook: WebhookConfig::default(),
            poll: PollConfig::default(),
            notification: NotificationConfig::default(),
            dnd: DndConfig::default(),
            timer: TimerConfig::default(),
            marquee: MarqueeConfig::default(),
            todo: TodoConfig::default(),
            hook: HookConfig::default(),
        }
    }
}

fn default_language() -> String {
    "zh".into()
}

fn default_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    #[serde(default = "default_language")]
    pub language: String,
    pub mode: NotificationMode,
    pub run_on_startup: bool,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
            mode: NotificationMode::Push,
            run_on_startup: false,
            minimize_to_tray: true,
            close_to_tray: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationMode {
    Push,
    Pull,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub port: u16,
    pub secret: String,
    pub github_events: Vec<String>,
    pub gitlab_events: Vec<String>,
    pub bitbucket_events: Vec<String>,
    pub custom_enabled: bool,
    pub custom_title_path: String,
    pub custom_body_path: String,
    pub custom_severity: String,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 3927,
            secret: String::new(),
            github_events: vec![
                "push".into(),
                "pull_request".into(),
                "issues".into(),
                "issue_comment".into(),
                "release".into(),
            ],
            gitlab_events: vec![
                "Push Hook".into(),
                "Merge Request Hook".into(),
                "Issue Hook".into(),
            ],
            bitbucket_events: vec![
                "repo:push".into(),
                "pullrequest:created".into(),
                "pullrequest:updated".into(),
                "pullrequest:approved".into(),
                "pullrequest:merged".into(),
                "repo:refs_changed".into(),
                "pr:opened".into(),
                "pr:modified".into(),
                "pr:reviewer_approved".into(),
                "pr:merged".into(),
                "pr:declined".into(),
            ],
            custom_enabled: false,
            custom_title_path: "title".into(),
            custom_body_path: "body".into(),
            custom_severity: "Info".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PollConfig {
    pub enabled: bool,
    pub endpoints: Vec<PollEndpoint>,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoints: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub interval_secs: u64,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationConfig {
    pub sound_enabled: bool,
    pub sound_file: String,
    pub sound_volume: f32,
    pub marquee_enabled: bool,
    pub tray_enabled: bool,
    pub max_history: usize,
    /// Toast card on-screen seconds per severity. 0 = sticky (manual close).
    pub toast_info_secs: u32,
    pub toast_warning_secs: u32,
    pub toast_critical_secs: u32,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            sound_file: "ping".into(),
            sound_volume: 0.7,
            marquee_enabled: true,
            tray_enabled: true,
            max_history: 500,
            toast_info_secs: 10,
            toast_warning_secs: 10,
            toast_critical_secs: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DndConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub schedules: Vec<DndSchedule>,
}

impl Default for DndConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            schedules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DndSchedule {
    pub id: String,
    pub name: String,
    pub start_time: String,
    pub end_time: String,
    pub days: Vec<WeekDay>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeekDay {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TimerConfig {
    pub pomodoro_work_mins: u32,
    pub pomodoro_short_break_mins: u32,
    pub pomodoro_long_break_mins: u32,
    pub pomodoro_rounds: u32,
    #[serde(default = "default_pomodoro_sound")]
    pub pomodoro_sound_file: String,
    pub auto_start_break: bool,
    pub auto_start_work: bool,
}

fn default_pomodoro_sound() -> String {
    "chime".into()
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            pomodoro_work_mins: 25,
            pomodoro_short_break_mins: 1,
            pomodoro_long_break_mins: 0,
            pomodoro_rounds: 4,
            pomodoro_sound_file: "chime".into(),
            auto_start_break: false,
            auto_start_work: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MarqueeConfig {
    pub position: MarqueePosition,
    pub speed: u32,
    pub height: u32,
    pub font_size: u32,
    pub font_family: String,
    pub icon_before: String,
    pub icon_after: String,
    pub bg_color: String,
    pub text_color: String,
    pub opacity: f32,
    pub duration_secs: u32,
    /// Number of danmaku tracks (1..=3). Defaults to 2; the serde default keeps
    /// config files written before this field existed parseable.
    #[serde(default = "default_marquee_tracks")]
    pub tracks: u32,
}

fn default_marquee_tracks() -> u32 {
    2
}

impl MarqueeConfig {
    /// Track count clamped to the supported range.
    pub fn track_count(&self) -> usize {
        (self.tracks as usize).clamp(1, 3)
    }
}

impl Default for MarqueeConfig {
    fn default() -> Self {
        Self {
            position: MarqueePosition::Top,
            speed: 100,
            height: 40,
            font_size: 16,
            font_family: "sans-serif".into(),
            icon_before: String::new(),
            icon_after: String::new(),
            bg_color: "#1e3a5f".into(),
            text_color: "#ffffff".into(),
            opacity: 0.9,
            duration_secs: 30,
            tracks: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarqueePosition {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoPullEndpoint {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub url: String,
    #[serde(default = "default_pull_interval")]
    pub interval_secs: u64,
    #[serde(default)]
    pub method: HttpMethod,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
}

fn default_true() -> bool { true }
fn default_pull_interval() -> u64 { 86400 }

fn default_stop_sound() -> String { "klaudio-minimal-zen-stop".into() }
fn default_notification_sound() -> String { "klaudio-sci-fi-terminal-notification".into() }
fn default_approval_sound() -> String { "klaudio-retro-8bit-notification".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HookConfig {
    pub enabled: bool,
    pub cli_tools: Vec<CliToolConfig>,
    pub approval_timeout_secs: u32,
    // Shared hook notification settings (applied to all CLI tools)
    pub on_stop_sound: bool,
    #[serde(default = "default_stop_sound")]
    pub stop_sound_file: String,
    pub on_stop_marquee: bool,
    pub on_notification_sound: bool,
    #[serde(default = "default_notification_sound")]
    pub notification_sound_file: String,
    pub on_notification_marquee: bool,
    pub approval_timeout_enabled: bool,
    #[serde(default)]
    pub approval_timeout_sound_enabled: bool,
    #[serde(default = "default_approval_sound")]
    pub approval_timeout_sound_file: String,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cli_tools: default_cli_tools(),
            approval_timeout_secs: 120,
            on_stop_sound: true,
            stop_sound_file: default_stop_sound(),
            on_stop_marquee: true,
            on_notification_sound: true,
            notification_sound_file: default_notification_sound(),
            on_notification_marquee: true,
            approval_timeout_enabled: false,
            approval_timeout_sound_enabled: true,
            approval_timeout_sound_file: default_approval_sound(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliToolConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub install_status: HookStatus,
    pub config_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum HookStatus {
    NotInstalled,
    Installed,
    Error(String),
}

pub(crate) fn default_cli_tools() -> Vec<CliToolConfig> {
    fn base(id: &str, name: &str) -> CliToolConfig {
        CliToolConfig {
            id: id.into(),
            name: name.into(),
            enabled: false,
            install_status: HookStatus::NotInstalled,
            config_path: None,
        }
    }
    vec![
        base("claude", "Claude Code"),
        base("opencode", "OpenCode"),
        base("codex", "Codex"),
        base("kiro", "Kiro"),
        base("codebuddy", "CodeBuddy"),
        base("qoder", "Qoder"),
        base("gemini", "Gemini CLI"),
        base("kimi", "Kimi Code"),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TodoConfig {
    pub pull_enabled: bool,
    #[serde(default)]
    pub pull_endpoints: Vec<TodoPullEndpoint>,
    pub push_enabled: bool,
    pub push_port: u16,
}

impl Default for TodoConfig {
    fn default() -> Self {
        Self {
            pull_enabled: false,
            pull_endpoints: vec![],
            push_enabled: false,
            push_port: 3928,
        }
    }
}
