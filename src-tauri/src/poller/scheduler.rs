use crate::poller::client;
use crate::state::AppState;
use tracing::error;

pub fn start(state: AppState) {
    tauri::async_runtime::spawn(async move {
        let endpoints = state.config.read().await.poll.endpoints.clone();

        for endpoint in endpoints {
            if !endpoint.enabled {
                continue;
            }
            let interval_secs = endpoint.interval_secs.max(10);

            let tx = state.notification_tx.clone();

            tauri::async_runtime::spawn(async move {
                let mut last_hash: u64 = 0;
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
                loop {
                    interval.tick().await;
                    match client::poll_endpoint(&endpoint).await {
                        Ok(Some(event)) => {
                            let new_hash = client::compute_hash(&event.body);
                            if new_hash != last_hash {
                                last_hash = new_hash;
                                let _ = tx.send(event).await;
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            error!("Poll error for {}: {}", endpoint.name, e);
                        }
                    }
                }
            });
        }
    });
}
