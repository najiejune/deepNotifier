use crate::config::schema::{HttpMethod, PollEndpoint};
use crate::notifier::dispatcher::{NotificationEvent, NotificationSource, Severity};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub async fn poll_endpoint(
    endpoint: &PollEndpoint,
) -> Result<Option<NotificationEvent>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(endpoint.timeout_secs))
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;
    let mut request = match endpoint.method {
        HttpMethod::GET => client.get(&endpoint.url),
        HttpMethod::POST => client.post(&endpoint.url),
    };

    for (key, value) in &endpoint.headers {
        request = request.header(key.as_str(), value.as_str());
    }

    if let Some(body) = &endpoint.body {
        request = request.body(body.clone());
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }

    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Simple content hash for change detection
    let content_hash = compute_hash(&body);

    Ok(Some(NotificationEvent {
        id: uuid::Uuid::new_v4().to_string(),
        source: NotificationSource::Poll {
            endpoint_name: endpoint.name.clone(),
        },
        event_type: "poll.response".into(),
        title: format!("Poll: {}", endpoint.name),
        body: format!("Status: {} | Hash: {:016x}", status, content_hash),
        severity: Severity::Info,
        timestamp: chrono::Local::now(),
        raw_payload: None,
        url: Some(endpoint.url.clone()),
        pid: None,
    }))
}

pub fn compute_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}
