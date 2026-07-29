use crate::state::AppState;
use crate::webhook::handlers;
use axum::Router;
use axum::routing::post;
use std::sync::Arc;
use tauri::Emitter;

pub fn start(app_handle: tauri::AppHandle, state: AppState) {
    tauri::async_runtime::spawn(async move {
        let port = state.config.read().await.webhook.port;
        let shared_state = Arc::new(state);

        let app = Router::new()
            .route("/webhook/github", post(handlers::github_handler))
            .route("/webhook/gitlab", post(handlers::gitlab_handler))
            .route("/webhook/bitbucket", post(handlers::bitbucket_handler))
            .route("/webhook/custom", post(handlers::custom_handler))
            .route("/hook/cli", post(handlers::cli_handler))
            .with_state(shared_state.clone());

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        tracing::info!("Webhook server listening on {}", addr);

        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind webhook server on port {}: {}", port, e);
                let _ = app_handle.emit("webhook-error", format!("Port {} bind failed: {}", port, e));
                return;
            }
        };

        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Webhook server error: {}", e);
        }
    });
}
