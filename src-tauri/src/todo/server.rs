use std::sync::Arc;
use axum::{extract::State, routing::post, Json, Router};
use tauri::Emitter;
use tokio::sync::RwLock;
use tracing;

use crate::todo::models::{PushTodoRequest, TodoItem, TodoSource};
use crate::todo::store::TodoStore;

struct PushState {
    store: Arc<RwLock<TodoStore>>,
    app_handle: tauri::AppHandle,
}

pub async fn start(
    app_handle: tauri::AppHandle,
    store: Arc<RwLock<TodoStore>>,
    port: u16,
) {
    let state = Arc::new(PushState { store, app_handle });

    let app = Router::new()
        .route("/todos", post(handle_push))
        .route("/todos/", post(handle_push))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    tracing::info!("Todo push server listening on {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("Failed to bind todo push server on {}: {}", addr, e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        tracing::warn!("Todo push server stopped: {}", e);
    }
}

async fn handle_push(
    State(state): State<Arc<PushState>>,
    Json(req): Json<PushTodoRequest>,
) -> Json<TodoItem> {
    let item = TodoItem {
        id: uuid::Uuid::new_v4().to_string(),
        text: req.text,
        completed: false,
        due_date: req.due_date,
        created_at: chrono::Local::now().to_rfc3339(),
        source: TodoSource::Push {
            remote_addr: "http".into(),
        },
    };

    state.store.write().await.add(item.clone());

    let _ = state.app_handle.emit("todos-updated", ());

    tracing::info!("Received pushed todo: {}", item.text);
    Json(item)
}
