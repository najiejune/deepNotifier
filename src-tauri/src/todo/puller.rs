use std::sync::Arc;
use std::collections::HashMap;
use tauri::Emitter;
use tokio::sync::RwLock;
use tracing;

use crate::config::schema::{HttpMethod, TodoPullEndpoint};
use crate::todo::models::{PullTodoResponse, TodoItem, TodoSource};
use crate::todo::store::TodoStore;

pub async fn fetch_and_merge(
    endpoint: &TodoPullEndpoint,
    store: &Arc<RwLock<TodoStore>>,
) -> Result<usize, String> {
    let client = reqwest::Client::new();
    let url = endpoint.url.trim();
    if url.is_empty() {
        return Err("Pull URL is empty".into());
    }

    let mut request = match endpoint.method {
        HttpMethod::GET => client.get(url),
        HttpMethod::POST => {
            let body = endpoint.body.clone().unwrap_or_default();
            client.post(url).body(body)
        }
    };

    for (key, value) in &endpoint.headers {
        request = request.header(key.as_str(), value.as_str());
    }
    request = request
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(30));

    let resp = request.send().await.map_err(|e| format!("Pull request failed: {}", e))?;
    let body = resp.text().await.map_err(|e| format!("Read body failed: {}", e))?;

    let parsed: PullTodoResponse =
        serde_json::from_str(&body).map_err(|e| format!("Parse failed: {} — body: {}", e, &body[..body.len().min(200)]))?;

    let incoming: Vec<TodoItem> = parsed
        .todos
        .into_iter()
        .map(|mut t| {
            if t.id.is_empty() {
                t.id = uuid::Uuid::new_v4().to_string();
            }
            if t.created_at.is_empty() {
                t.created_at = chrono::Local::now().to_rfc3339();
            }
            t.source = TodoSource::Pull;
            t
        })
        .collect();

    let count = store.write().await.merge_pulled(incoming);
    Ok(count)
}

pub fn start_scheduler(
    app_handle: tauri::AppHandle,
    config: Arc<RwLock<crate::config::schema::AppConfig>>,
    store: Arc<RwLock<TodoStore>>,
) {
    tauri::async_runtime::spawn(async move {
        // Track the last pull time per endpoint ID
        let mut last_pull: HashMap<String, tokio::time::Instant> = HashMap::new();

        // Skip first immediate tick
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        loop {
            let cfg = config.read().await;
            let enabled = cfg.todo.pull_enabled;
            let endpoints: Vec<TodoPullEndpoint> = cfg
                .todo
                .pull_endpoints
                .iter()
                .filter(|ep| ep.enabled && !ep.url.trim().is_empty())
                .cloned()
                .collect();
            drop(cfg);

            if !enabled || endpoints.is_empty() {
                last_pull.clear();
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                continue;
            }

            let now = tokio::time::Instant::now();
            for ep in &endpoints {
                let interval = std::time::Duration::from_secs(ep.interval_secs.max(30));
                let should_pull = last_pull
                    .get(&ep.id)
                    .map(|last| now.duration_since(*last) >= interval)
                    .unwrap_or(true);

                if should_pull {
                    last_pull.insert(ep.id.clone(), now);
                    match fetch_and_merge(ep, &store).await {
                        Ok(count) if count > 0 => {
                            tracing::info!("Pulled {} new todo(s) from {}", count, ep.name);
                            let _ = app_handle.emit("todos-updated", ());
                        }
                        Ok(_) => {}
                        Err(e) => {
                            tracing::warn!("Todo pull '{}' failed: {}", ep.name, e);
                        }
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    });
}
