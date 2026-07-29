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

/// If the body string is JSON, extract human-readable fields.
/// Otherwise return the original string unchanged.
fn clean_json_body(raw: &str) -> String {
    let v: serde_json::Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(_) => return raw.to_string(),
    };
    match v {
        serde_json::Value::Object(ref map) => {
            // Try common human-readable keys in priority order
            for key in &["message", "text", "content", "title", "name"] {
                if let Some(val) = map.get(*key) {
                    if let Some(s) = val.as_str() {
                        if !s.is_empty() {
                            return s.to_string();
                        }
                    }
                }
            }
            // If no standard key found, show a summary instead of raw JSON
            let keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
            if keys.len() == 1 {
                // Single-key object: show its value
                if let Some(val) = map.values().next() {
                    return clean_json_body(&val.to_string());
                }
            }
            format!("({} fields: {})", keys.len(), keys.join(", "))
        }
        serde_json::Value::String(s) => s,
        serde_json::Value::Array(ref arr) => {
            if arr.is_empty() {
                return "(empty)".to_string();
            }
            if arr.len() == 1 {
                return clean_json_body(&arr[0].to_string());
            }
            format!("[{} items]", arr.len())
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "(no data)".to_string(),
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
    // Detect CLI hook events by payload fields (cli_id + event_type)
    let cli_id = payload.get("cli_id").and_then(|v| v.as_str()).map(String::from);
    let hook_event = payload.get("event_type").and_then(|v| v.as_str()).map(String::from);
    let pid = payload.get("pid").and_then(|v| v.as_u64()).map(|n| n as u32);

    let (source, event_type) = match (cli_id, hook_event) {
        (Some(name), Some(ev)) => (NotificationSource::Hook { cli_name: name }, ev),
        _ => (NotificationSource::Custom, "custom.webhook".into()),
    };

    let raw_title = extract_by_path(payload, title_path)
        .or_else(|| payload.get("title").and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_else(|| "(no title)".into());

    let raw_body = extract_by_path(payload, body_path)
        .or_else(|| payload.get("body").and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_else(|| "(no body)".into());

    let (title, body) = match &source {
        NotificationSource::Hook { .. } => {
            (clean_json_body(&raw_title), clean_json_body(&raw_body))
        }
        _ => (raw_title, raw_body),
    };

    let severity = parse_severity(severity_str);

    NotificationEvent {
        id: Uuid::new_v4().to_string(),
        source,
        event_type,
        title,
        body,
        severity,
        timestamp: Local::now(),
        raw_payload: Some(payload.clone()),
        url: None,
        pid,
    }
}
