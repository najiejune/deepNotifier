//! Shared notification event model.
//!
//! `NotificationEvent` is the single contract between webhook sources, the
//! desktop dispatcher, the alert server, and the mobile client. The serde
//! shape must stay stable — the TS mirror lives in `src/types/index.ts`.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub id: String,
    pub source: NotificationSource,
    pub event_type: String,
    pub title: String,
    pub body: String,
    pub severity: Severity,
    pub timestamp: DateTime<Local>,
    pub raw_payload: Option<serde_json::Value>,
    pub url: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum NotificationSource {
    GitHub,
    GitLab,
    Bitbucket,
    Custom,
    Poll { endpoint_name: String },
    Timer,
    Pomodoro,
    System,
    Hook { cli_name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}
