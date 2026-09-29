//! Minimal reference implementation of the deepNotifier protocol v1.
//!
//! This is a local-integration demo ONLY. It deliberately does NOT implement
//! device persistence, FCM/APNs push, or the `since` back-fill endpoint —
//! see `docs/protocol-v1.md` for the full contract.
//!
//! Run:  cargo run -p dn-core --example demo-server
//! Push:  curl -X POST localhost:3930/push \
//!          -H "Content-Type: application/json" \
//!          -d '{"title":"Deploy finished","body":"Build #42 passed","severity":"Info"}'

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::{get, post},
    Json, Router,
};
use chrono::Local;
use dn_core::{
    ClientMessage, NotificationEvent, NotificationSource, Platform, RegisterDeviceResponse,
    ServerMessage, Severity,
};
use futures_util::{sink::SinkExt, stream::StreamExt};

use tokio::sync::broadcast;

const PORT: u16 = 3930;
const PING_INTERVAL_SECS: u64 = 25;

#[derive(Clone)]
struct AppState {
    /// Serialized `ServerMessage::Notification` JSON broadcast to all clients.
    events_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    let (events_tx, _) = broadcast::channel(256);
    let state = AppState { events_tx };

    let app = Router::new()
        .route("/api/v1/ws", get(ws_handler))
        .route("/api/v1/devices", post(register_device))
        .route("/push", post(push))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(state);

    let addr = format!("0.0.0.0:{PORT}");
    println!("demo-server listening on {addr}");
    println!("demo: any ?token= accepted, any api_token accepted");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn register_device() -> Json<RegisterDeviceResponse> {
    Json(RegisterDeviceResponse {
        device_id: "demo-device".into(),
        ws_token: "demo-ws-token".into(),
    })
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut events_rx = state.events_tx.subscribe();

    // Server speaks first: hello.
    let hello = ServerMessage::Hello {
        v: 1,
        server_time: Local::now().to_rfc3339(),
        missed_events: 0,
    };
    if sender
        .send(Message::Text(serde_json::to_string(&hello).unwrap().into()))
        .await
        .is_err()
    {
        return;
    }

    let mut ping_timer = tokio::time::interval(std::time::Duration::from_secs(PING_INTERVAL_SECS));
    ping_timer.tick().await; // skip the immediate first tick

    loop {
        tokio::select! {
            // Forward broadcast notifications to this client.
            msg = events_rx.recv() => {
                match msg {
                    Ok(text) => {
                        if sender.send(Message::Text(text.into())).await.is_err() { break; }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        println!("client lagged, {n} events skipped");
                    }
                    Err(_) => break,
                }
            }
            // Heartbeat.
            _ = ping_timer.tick() => {
                let ping = ServerMessage::Ping {
                    v: 1,
                    ts: Local::now().timestamp() as u64,
                };
                if sender
                    .send(Message::Text(serde_json::to_string(&ping).unwrap().into()))
                    .await
                    .is_err()
                { break; }
            }
            // Client messages: hello / pong / ack.
            item = receiver.next() => {
                match item {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::Hello { platform, app_version, .. }) => {
                                let pf = match platform {
                                    Platform::Android => "android",
                                    Platform::Ios => "ios",
                                };
                                println!("client hello: {pf} v{app_version}");
                            }
                            Ok(ClientMessage::Pong { .. }) => {}
                            Ok(ClientMessage::Ack { ids, .. }) => {
                                println!("ack: {} event(s)", ids.len());
                            }
                            Err(e) => println!("bad client message: {e}"),
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
    println!("client disconnected");
}

async fn push(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let get = |k: &str| body.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let severity = match get("severity").to_lowercase().as_str() {
        "warning" => Severity::Warning,
        "critical" => Severity::Critical,
        _ => Severity::Info,
    };
    let event = NotificationEvent {
        id: uuid::Uuid::new_v4().to_string(),
        source: NotificationSource::Custom,
        event_type: get("event_type"),
        title: get("title"),
        body: get("body"),
        severity,
        timestamp: Local::now(),
        raw_payload: None,
        url: body.get("url").and_then(|v| v.as_str()).map(Into::into),
        pid: None,
    };
    let msg = ServerMessage::Notification { v: 1, event };
    let n = state
        .events_tx
        .send(serde_json::to_string(&msg).unwrap())
        .unwrap_or(0);
    println!("pushed to {n} client(s)");
    Json(serde_json::json!({ "delivered_to": n }))
}
