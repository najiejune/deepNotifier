use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};

use crate::notifier::dispatcher::NotificationEvent;
use crate::notifier::dispatcher::NotificationSource;
use crate::notifier::dispatcher::Severity;
use crate::state::AppState;
use crate::webhook::bitbucket;
use crate::webhook::custom;
use crate::webhook::verify;

pub async fn github_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> &'static str {
    let config = state.config.read().await;
    let secret = config.webhook.secret.clone();
    let allowed_events = config.webhook.github_events.clone();
    drop(config);

    // Verify signature if secret is configured
    if !secret.is_empty() {
        let signature = headers
            .get("X-Hub-Signature-256")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !verify::verify_github_signature(&secret, &payload.to_string(), signature) {
            warn!("GitHub webhook signature verification failed");
            return "Invalid signature";
        }
    }

    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if !allowed_events.contains(&event_type.to_string()) {
        info!("GitHub event '{}' not in allowed list, ignoring", event_type);
        return "Event ignored";
    }

    let event = parse_github_event(event_type, &payload);
    info!("GitHub webhook: {} - {}", event_type, event.title);

    let _ = state.notification_tx.send(event).await;
    "OK"
}

pub async fn gitlab_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> &'static str {
    let config = state.config.read().await;
    let token = config.webhook.secret.clone();
    let allowed_events = config.webhook.gitlab_events.clone();
    drop(config);

    // Verify token if configured
    if !token.is_empty() {
        let received_token = headers
            .get("X-Gitlab-Token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if received_token != token {
            warn!("GitLab webhook token verification failed");
            return "Invalid token";
        }
    }

    let event_type = headers
        .get("X-Gitlab-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if !allowed_events.contains(&event_type.to_string()) {
        info!("GitLab event '{}' not in allowed list, ignoring", event_type);
        return "Event ignored";
    }

    let event = parse_gitlab_event(event_type, &payload);
    info!("GitLab webhook: {} - {}", event_type, event.title);

    let _ = state.notification_tx.send(event).await;
    "OK"
}

fn parse_github_event(event_type: &str, payload: &Value) -> NotificationEvent {
    let repo = payload["repository"]["full_name"]
        .as_str()
        .unwrap_or("unknown");
    let sender = payload["sender"]["login"].as_str().unwrap_or("unknown");

    let (title, body, severity) = match event_type {
        "push" => {
            let count = payload["commits"].as_array().map(|a| a.len()).unwrap_or(0);
            let branch = payload["ref"]
                .as_str()
                .unwrap_or("")
                .strip_prefix("refs/heads/")
                .unwrap_or("unknown");
            (
                format!("Push to {} ({})", repo, branch),
                format!("{} pushed {} commit(s) to {}", sender, count, branch),
                Severity::Info,
            )
        }
        "pull_request" => {
            let action = payload["action"].as_str().unwrap_or("unknown");
            let pr_title = payload["pull_request"]["title"].as_str().unwrap_or("PR");
            let pr_number = payload["number"].as_u64().unwrap_or(0);
            (
                format!("PR #{} {} - {}", pr_number, action, repo),
                format!("{} {} PR #{}: {}", sender, action, pr_number, pr_title),
                if action == "opened" { Severity::Info } else { Severity::Info },
            )
        }
        "issues" => {
            let action = payload["action"].as_str().unwrap_or("unknown");
            let issue_title = payload["issue"]["title"].as_str().unwrap_or("Issue");
            let issue_number = payload["number"].as_u64().unwrap_or(0);
            (
                format!("Issue #{} {} - {}", issue_number, action, repo),
                format!("{} {} issue #{}: {}", sender, action, issue_number, issue_title),
                Severity::Info,
            )
        }
        "release" => {
            let tag = payload["release"]["tag_name"].as_str().unwrap_or("unknown");
            (
                format!("Release {} - {}", tag, repo),
                format!("{} released {} on {}", sender, tag, repo),
                Severity::Warning,
            )
        }
        _ => (
            format!("GitHub {} - {}", event_type, repo),
            format!("{} triggered {} on {}", sender, event_type, repo),
            Severity::Info,
        ),
    };

    let html_url = payload["repository"]["html_url"]
        .as_str()
        .unwrap_or("")
        .to_string();

    NotificationEvent {
        id: uuid::Uuid::new_v4().to_string(),
        source: NotificationSource::GitHub,
        event_type: format!("github.{}", event_type),
        title,
        body,
        severity,
        timestamp: chrono::Local::now(),
        raw_payload: Some(payload.clone()),
        url: if html_url.is_empty() { None } else { Some(html_url) },
    }
}

fn parse_gitlab_event(event_type: &str, payload: &Value) -> NotificationEvent {
    let project = payload["project"]["path_with_namespace"]
        .as_str()
        .unwrap_or("unknown");
    let user = payload["user_name"].as_str().unwrap_or("unknown");

    let (title, body, severity) = match event_type {
        "Push Hook" => {
            let count = payload["total_commits_count"].as_u64().unwrap_or(0);
            let branch = payload["ref"].as_str().unwrap_or("unknown")
                .strip_prefix("refs/heads/")
                .unwrap_or("unknown");
            (
                format!("Push to {} ({})", project, branch),
                format!("{} pushed {} commit(s) to {}", user, count, branch),
                Severity::Info,
            )
        }
        "Merge Request Hook" => {
            let action = payload["object_attributes"]["action"].as_str().unwrap_or("unknown");
            let mr_title = payload["object_attributes"]["title"].as_str().unwrap_or("MR");
            let mr_iid = payload["object_attributes"]["iid"].as_u64().unwrap_or(0);
            (
                format!("MR !{} {} - {}", mr_iid, action, project),
                format!("{} {} MR !{}: {}", user, action, mr_iid, mr_title),
                Severity::Info,
            )
        }
        "Issue Hook" => {
            let action = payload["object_attributes"]["action"].as_str().unwrap_or("unknown");
            let issue_title = payload["object_attributes"]["title"].as_str().unwrap_or("Issue");
            let issue_iid = payload["object_attributes"]["iid"].as_u64().unwrap_or(0);
            (
                format!("Issue #{} {} - {}", issue_iid, action, project),
                format!("{} {} issue #{}: {}", user, action, issue_iid, issue_title),
                Severity::Info,
            )
        }
        _ => (
            format!("GitLab {} - {}", event_type, project),
            format!("{} triggered {} on {}", user, event_type, project),
            Severity::Info,
        ),
    };

    let web_url = payload["project"]["web_url"]
        .as_str()
        .unwrap_or("")
        .to_string();

    NotificationEvent {
        id: uuid::Uuid::new_v4().to_string(),
        source: NotificationSource::GitLab,
        event_type: format!("gitlab.{}", event_type.replace(' ', "_").to_lowercase()),
        title,
        body,
        severity,
        timestamp: chrono::Local::now(),
        raw_payload: Some(payload.clone()),
        url: if web_url.is_empty() { None } else { Some(web_url) },
    }
}

pub async fn bitbucket_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> &'static str {
    let config = state.config.read().await;
    let secret = config.webhook.secret.clone();
    let allowed_events = config.webhook.bitbucket_events.clone();
    drop(config);

    if !secret.is_empty() {
        let signature = headers
            .get("X-Hub-Signature-256")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !verify::verify_github_signature(&secret, &payload.to_string(), signature) {
            warn!("Bitbucket webhook signature verification failed");
            return "Invalid signature";
        }
    }

    let event_key = headers
        .get("X-Event-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if !allowed_events.contains(&event_key.to_string()) {
        info!("Bitbucket event '{}' not in allowed list, ignoring", event_key);
        return "Event ignored";
    }

    match bitbucket::parse_bitbucket_event(event_key, &payload) {
        Some(event) => {
            info!("Bitbucket webhook: {} - {}", event_key, event.title);
            let _ = state.notification_tx.send(event).await;
            "OK"
        }
        None => {
            warn!("Failed to parse Bitbucket event: {}", event_key);
            "Parse error"
        }
    }
}

pub async fn custom_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> &'static str {
    let config = state.config.read().await;
    if !config.webhook.custom_enabled {
        drop(config);
        return "Custom webhook disabled";
    }
    let title_path = config.webhook.custom_title_path.clone();
    let body_path = config.webhook.custom_body_path.clone();
    let severity = config.webhook.custom_severity.clone();
    drop(config);

    let event = custom::parse_custom_event(&payload, &title_path, &body_path, &severity);
    info!("Custom webhook: {}", event.title);

    let _ = state.notification_tx.send(event).await;
    "OK"
}
