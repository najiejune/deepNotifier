use crate::notifier::dispatcher::{NotificationEvent, NotificationSource, Severity};
use chrono::Local;
use serde_json::Value;
use uuid::Uuid;

fn get_repo(payload: &Value) -> &str {
    // Bitbucket Cloud
    if let Some(name) = payload["repository"]["full_name"].as_str() {
        return name;
    }
    // Bitbucket Server
    payload["repository"]["name"]
        .as_str()
        .unwrap_or("unknown")
}

fn get_actor(payload: &Value) -> &str {
    // Bitbucket Cloud
    if let Some(name) = payload["actor"]["display_name"].as_str() {
        return name;
    }
    if let Some(name) = payload["actor"]["nickname"].as_str() {
        return name;
    }
    // Bitbucket Server (camelCase)
    if let Some(name) = payload["actor"]["displayName"].as_str() {
        return name;
    }
    payload["actor"]["name"].as_str().unwrap_or("unknown")
}

fn get_url(payload: &Value) -> Option<String> {
    // Bitbucket Cloud
    if let Some(url) = payload["repository"]["links"]["html"]["href"].as_str() {
        return Some(url.to_string());
    }
    // Bitbucket Server
    if let Some(base) = payload["repository"]["links"]["self"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|l| l["href"].as_str())
    {
        return Some(base.to_string());
    }
    None
}

pub fn parse_bitbucket_event(event_key: &str, payload: &Value) -> Option<NotificationEvent> {
    let repo = get_repo(payload);
    let actor = get_actor(payload);
    let url = get_url(payload);

    // Normalize event key for matching
    let (title, body, severity) = match event_key {
        // --- Cloud: push ---
        "repo:push" => {
            let changes = payload["push"]["changes"].as_array();
            let branch = changes
                .and_then(|c| c.first())
                .and_then(|c| c["new"]["name"].as_str())
                .unwrap_or("unknown");
            let commit_count: usize = changes
                .map(|c| {
                    c.iter()
                        .filter_map(|ch| ch["commits"].as_array())
                        .flatten()
                        .count()
                })
                .unwrap_or(0);
            (
                format!("Push to {} ({})", repo, branch),
                format!("{} pushed {} commit(s) to {}", actor, commit_count, branch),
                Severity::Info,
            )
        }

        // --- Server: push ---
        "repo:refs_changed" => {
            let changes = payload["changes"].as_array();
            let branch = changes
                .and_then(|c| c.first())
                .and_then(|c| {
                    c["ref"]["displayId"]
                        .as_str()
                        .or(c["ref"]["id"].as_str().and_then(|r| {
                            r.strip_prefix("refs/heads/")
                        }))
                })
                .unwrap_or("unknown");
            let commit_count: usize = changes
                .map(|c| c.len())
                .unwrap_or(0);
            (
                format!("Push to {} ({})", repo, branch),
                format!("{} pushed {} change(s) to {}", actor, commit_count, branch),
                Severity::Info,
            )
        }

        // --- Cloud: PR ---
        e @ ("pullrequest:created" | "pullrequest:updated" | "pullrequest:approved") => {
            let pr = &payload["pullrequest"];
            let pr_id = pr["id"].as_u64().map(|i| i.to_string()).unwrap_or_else(|| "?".into());
            let pr_title = pr["title"].as_str().unwrap_or("(no title)");
            let action = match e {
                "pullrequest:created" => "created",
                "pullrequest:updated" => "updated",
                "pullrequest:approved" => "approved",
                _ => unreachable!(),
            };
            (
                format!("PR #{} {} - {}", pr_id, action, repo),
                format!("{} {} PR #{}: {}", actor, action, pr_id, pr_title),
                Severity::Info,
            )
        }

        "pullrequest:merged" => {
            let pr = &payload["pullrequest"];
            let pr_id = pr["id"].as_u64().map(|i| i.to_string()).unwrap_or_else(|| "?".into());
            let pr_title = pr["title"].as_str().unwrap_or("(no title)");
            (
                format!("PR #{} merged - {}", pr_id, repo),
                format!("{} merged PR #{}: {}", actor, pr_id, pr_title),
                Severity::Warning,
            )
        }

        // --- Server: PR ---
        e @ ("pr:opened" | "pr:modified" | "pr:reviewer_approved" | "pr:from_ref_updated") => {
            let pr = &payload["pullRequest"];
            let pr_id = pr["id"].as_u64().map(|i| i.to_string()).unwrap_or_else(|| "?".into());
            let pr_title = pr["title"].as_str().unwrap_or("(no title)");
            let action = match e {
                "pr:opened" => "opened",
                "pr:modified" => "modified",
                "pr:reviewer_approved" => "approved",
                "pr:from_ref_updated" => "updated",
                _ => unreachable!(),
            };
            (
                format!("PR #{} {} - {}", pr_id, action, repo),
                format!("{} {} PR #{}: {}", actor, action, pr_id, pr_title),
                Severity::Info,
            )
        }

        "pr:merged" => {
            let pr = &payload["pullRequest"];
            let pr_id = pr["id"].as_u64().map(|i| i.to_string()).unwrap_or_else(|| "?".into());
            let pr_title = pr["title"].as_str().unwrap_or("(no title)");
            (
                format!("PR #{} merged - {}", pr_id, repo),
                format!("{} merged PR #{}: {}", actor, pr_id, pr_title),
                Severity::Warning,
            )
        }

        "pr:declined" => {
            let pr = &payload["pullRequest"];
            let pr_id = pr["id"].as_u64().map(|i| i.to_string()).unwrap_or_else(|| "?".into());
            let pr_title = pr["title"].as_str().unwrap_or("(no title)");
            (
                format!("PR #{} declined - {}", pr_id, repo),
                format!("{} declined PR #{}: {}", actor, pr_id, pr_title),
                Severity::Warning,
            )
        }

        // --- Catch-all ---
        _ => (
            format!("Bitbucket {} - {}", event_key, repo),
            format!("{} triggered {} on {}", actor, event_key, repo),
            Severity::Info,
        ),
    };

    Some(NotificationEvent {
        id: Uuid::new_v4().to_string(),
        source: NotificationSource::Bitbucket,
        event_type: format!("bitbucket.{}", event_key.replace(':', ".")),
        title,
        body,
        severity,
        timestamp: Local::now(),
        raw_payload: Some(payload.clone()),
        url,
        pid: None,
    })
}
