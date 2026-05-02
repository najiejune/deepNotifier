use crate::notifier::dispatcher::{NotificationEvent, NotificationSource, Severity};
use chrono::Local;
use serde_json::Value;
use uuid::Uuid;

fn extract_by_path(payload: &Value, path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    let mut current = payload;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    match current {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => None,
        _ => Some(current.to_string()),
    }
}

fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "warning" => Severity::Warning,
        "critical" => Severity::Critical,
        _ => Severity::Info,
    }
}

pub fn parse_custom_event(
    payload: &Value,
    title_path: &str,
    body_path: &str,
    severity_str: &str,
) -> NotificationEvent {
    let title = extract_by_path(payload, title_path)
        .or_else(|| payload.get("title").and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_else(|| "(no title)".into());

    let body = extract_by_path(payload, body_path)
        .or_else(|| payload.get("body").and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_else(|| "(no body)".into());

    let severity = parse_severity(severity_str);

    NotificationEvent {
        id: Uuid::new_v4().to_string(),
        source: NotificationSource::Custom,
        event_type: "custom.webhook".into(),
        title,
        body,
        severity,
        timestamp: Local::now(),
        raw_payload: Some(payload.clone()),
        url: None,
    }
}
