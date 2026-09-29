use chrono::Local;
use dn_core::*;

fn sample_event() -> NotificationEvent {
    NotificationEvent {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
        source: NotificationSource::GitHub,
        event_type: "pull_request.opened".into(),
        title: "PR #42 opened".into(),
        body: "feat: mobile".into(),
        severity: Severity::Warning,
        timestamp: Local::now(),
        raw_payload: None,
        url: Some("https://example.com/pr/42".into()),
        pid: None,
    }
}

#[test]
fn server_notification_roundtrip() {
    let msg = ServerMessage::Notification { v: 1, event: sample_event() };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
    match parsed {
        ServerMessage::Notification { v, event } => {
            assert_eq!(v, 1);
            assert_eq!(event.id, "550e8400-e29b-41d4-a716-446655440000");
            assert_eq!(event.severity, Severity::Warning);
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn wire_format_matches_protocol_doc() {
    // Exact shapes from docs/protocol-v1.md must parse.
    let hello: ServerMessage = serde_json::from_str(
        r#"{"v":1,"type":"hello","server_time":"2025-01-01T12:00:00+08:00","missed_events":3}"#,
    )
    .unwrap();
    assert!(matches!(hello, ServerMessage::Hello { missed_events: 3, .. }));

    let ping: ServerMessage = serde_json::from_str(r#"{"v":1,"type":"ping","ts":1700000000}"#).unwrap();
    assert!(matches!(ping, ServerMessage::Ping { ts: 1700000000, .. }));

    let pong: ClientMessage = serde_json::from_str(r#"{"v":1,"type":"pong","ts":1700000000}"#).unwrap();
    assert!(matches!(pong, ClientMessage::Pong { .. }));

    let ack: ClientMessage =
        serde_json::from_str(r#"{"v":1,"type":"ack","ids":["a","b"]}"#).unwrap();
    assert!(matches!(ack, ClientMessage::Ack { .. }));

    let hello_c: ClientMessage = serde_json::from_str(
        r#"{"v":1,"type":"hello","platform":"ios","app_version":"0.2.0"}"#,
    )
    .unwrap();
    assert!(matches!(hello_c, ClientMessage::Hello { platform: Platform::Ios, .. }));
}

#[test]
fn missing_v_field_defaults_to_one() {
    let ping: ServerMessage = serde_json::from_str(r#"{"type":"ping","ts":1}"#).unwrap();
    assert!(matches!(ping, ServerMessage::Ping { v: 1, .. }));
}

#[test]
fn severity_serialization_is_pascal_case() {
    assert_eq!(serde_json::to_string(&Severity::Critical).unwrap(), "\"Critical\"");
    assert_eq!(serde_json::to_string(&NotificationSource::GitLab).unwrap(), "\"GitLab\"");
}
