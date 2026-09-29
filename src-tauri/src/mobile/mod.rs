//! Alert-server connection manager (mobile client side of docs/protocol-v1.md).
//!
//! The module compiles on every platform so it stays type-checked during
//! desktop development; auto-connect at startup and on resume only happens
//! under `cfg(mobile)`.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Local};
use dn_core::{
    ClientMessage, EventsResponse, NotificationEvent, Platform, RegisterDeviceRequest,
    RegisterDeviceResponse, ServerMessage, UpdatePushTokenRequest,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::connect_async;

use crate::config::persistence;
use crate::state::AppState;

#[cfg(target_os = "android")]
const PLATFORM: Platform = Platform::Android;
#[cfg(target_os = "ios")]
const PLATFORM: Platform = Platform::Ios;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
const PLATFORM: Platform = Platform::Android;

const WS_PATH: &str = "/api/v1/ws";
/// Reconnect backoff: 1s, 2s, 4s, ... capped at 30s.
const MAX_BACKOFF_SECS: u64 = 30;
/// The server pings every 25s; assume the connection is dead after 3 misses.
const READ_TIMEOUT_SECS: u64 = 75;
/// Bound every outbound network call so a dead server can never park a task.
const CONNECT_TIMEOUT_SECS: u64 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnState {
    Offline,
    Connecting,
    Online,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub state: ConnState,
    pub server_url: String,
    pub device_id: String,
    pub last_error: Option<String>,
}

/// HTTP client with timeouts, shared by registration / push-token /
/// back-fill / unregister calls.
fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .build()
        .unwrap_or_default()
}

#[derive(Clone)]
pub struct MobileConn {
    app: tauri::AppHandle,
    state: AppState,
    client: reqwest::Client,
    status: Arc<RwLock<ServerStatus>>,
    stop_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    seen_ids: Arc<Mutex<HashSet<String>>>,
    /// Timestamp of the newest event ingested from the alert server. Kept
    /// separate from the shared history because the history also holds
    /// local events (timer/pomodoro), which must not advance the back-fill
    /// watermark.
    last_server_ts: Arc<RwLock<Option<DateTime<Local>>>>,
}

impl MobileConn {
    pub fn new(app: tauri::AppHandle, state: AppState) -> Self {
        let cfg = state.config.blocking_read().server.clone();
        Self {
            app,
            state,
            client: http_client(),
            status: Arc::new(RwLock::new(ServerStatus {
                state: ConnState::Offline,
                server_url: cfg.server_url.clone(),
                device_id: cfg.device_id.clone(),
                last_error: None,
            })),
            stop_tx: Arc::new(Mutex::new(None)),
            seen_ids: Arc::new(Mutex::new(HashSet::new())),
            last_server_ts: Arc::new(RwLock::new(None)),
        }
    }
}

pub fn managed(app: &tauri::App, state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let mgr = MobileConn::new(app.handle().clone(), state.clone());
    app.manage(mgr);
    #[cfg(mobile)]
    {
        let auto = state.config.blocking_read().server.auto_connect;
        if auto {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(c) = get_conn(&handle) {
                    connect(&c).await;
                }
            });
        }
    }
    Ok(())
}

fn get_conn(app: &tauri::AppHandle) -> Result<MobileConn, String> {
    app.try_state::<MobileConn>()
        .map(|s| s.inner().clone())
        .ok_or_else(|| "mobile connection manager not initialized".to_string())
}

async fn set_status(conn: &MobileConn, state: ConnState, last_error: Option<String>) {
    {
        let mut s = conn.status.write().await;
        s.state = state;
        s.last_error = last_error;
    }
    let snapshot = conn.status.read().await.clone();
    let _ = conn.app.emit("server-status", &snapshot);
}

fn http_base(server_url: &str) -> Result<String, String> {
    let base = server_url.trim_end_matches('/');
    if base.starts_with("http://") || base.starts_with("https://") {
        Ok(base.to_string())
    } else if base.starts_with("ws://") {
        Ok(base.replacen("ws://", "http://", 1))
    } else if base.starts_with("wss://") {
        Ok(base.replacen("wss://", "https://", 1))
    } else {
        Err(format!("unsupported server_url scheme: {base}"))
    }
}

fn ws_url(server_url: &str, ws_token: &str) -> Result<String, String> {
    let http = http_base(server_url)?;
    let ws = if let Some(rest) = http.strip_prefix("https://") {
        format!("wss://{rest}")
    } else {
        http.replacen("http://", "ws://", 1)
    };
    let token: String = url::form_urlencoded::byte_serialize(ws_token.as_bytes()).collect();
    Ok(format!("{ws}{WS_PATH}?token={token}"))
}

async fn ensure_registered(conn: &MobileConn) -> Result<(), String> {
    {
        let cfg = conn.state.config.read().await;
        if !cfg.server.device_id.is_empty() {
            return Ok(());
        }
        if cfg.server.api_token.is_empty() || cfg.server.server_url.is_empty() {
            return Err("server_url and api_token not configured".to_string());
        }
    }
    // Snapshot what we need, then drop the lock before the network call —
    // never hold the config write lock across an await.
    let (base, api_token, push_token) = {
        let cfg = conn.state.config.read().await;
        (
            http_base(&cfg.server.server_url)?,
            cfg.server.api_token.clone(),
            cfg.server.push_token.clone(),
        )
    };
    let resp = conn
        .client
        .post(format!("{base}/api/v1/devices"))
        .bearer_auth(&api_token)
        .json(&RegisterDeviceRequest {
            platform: PLATFORM,
            push_token: if push_token.is_empty() {
                None
            } else {
                Some(push_token)
            },
            name: None,
        })
        .send()
        .await
        .map_err(|e| format!("device registration request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("device registration rejected: HTTP {}", resp.status()));
    }
    let body: RegisterDeviceResponse = resp
        .json()
        .await
        .map_err(|e| format!("bad registration response: {e}"))?;
    {
        let mut cfg = conn.state.config.write().await;
        // Another caller may have registered while we waited on the network.
        if !cfg.server.device_id.is_empty() {
            return Ok(());
        }
        cfg.server.device_id = body.device_id.clone();
        cfg.server.ws_token = body.ws_token;
        persistence::save(&conn.state.config_dir, &cfg);
    }
    let mut status = conn.status.write().await;
    status.device_id = body.device_id;
    drop(status);
    tracing::info!("registered device with alert server");
    Ok(())
}

/// Upload a fresh FCM/APNs push token to the server (called by the native
/// bridge on token refresh, and after registration).
pub async fn upload_push_token(app: tauri::AppHandle, token: String) -> Result<(), String> {
    let conn = get_conn(&app)?;
    {
        let mut cfg = conn.state.config.write().await;
        cfg.server.push_token = token.clone();
        persistence::save(&conn.state.config_dir, &cfg);
    }
    let cfg = conn.state.config.read().await.server.clone();
    if cfg.device_id.is_empty() {
        return Ok(()); // registration will pick it up later
    }
    let base = http_base(&cfg.server_url)?;
    let resp = conn
        .client
        .post(format!("{base}/api/v1/devices/{}/push-token", cfg.device_id))
        .bearer_auth(&cfg.api_token)
        .json(&UpdatePushTokenRequest { push_token: token })
        .send()
        .await
        .map_err(|e| format!("push-token upload failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("push-token upload rejected: HTTP {}", resp.status()));
    }
    Ok(())
}

/// Ingest an event arriving from any channel (WS live or back-fill).
/// Deduplicates by event id, stores history, emits to the frontend, and
/// raises an OS notification unless DND is active.
async fn ingest(conn: &MobileConn, event: NotificationEvent) {
    {
        let mut seen = conn.seen_ids.lock().await;
        if seen.contains(&event.id) {
            return;
        }
        if seen.len() > 1000 {
            seen.clear();
        }
        seen.insert(event.id.clone());
    }
    {
        let mut hist = conn.state.history.write().await;
        if hist.contains_id(&event.id) {
            return;
        }
        hist.push(event.clone());
    }
    {
        // Advance the back-fill watermark (server events only; ingest() is
        // never called for local timer/pomodoro events).
        let mut ts = conn.last_server_ts.write().await;
        if ts.is_none_or(|t| event.timestamp > t) {
            *ts = Some(event.timestamp);
        }
    }
    let _ = conn.app.emit("notification", &event);

    if *conn.state.dnd_active.read().await {
        tracing::info!("DND active, suppressing OS notification: {}", event.title);
        return;
    }
    let mut builder = conn
        .app
        .notification()
        .builder()
        .title(&event.title)
        .body(&event.body);
    if let Some(url) = &event.url {
        builder = builder.extra("url", url.clone());
    }
    if let Err(e) = builder.show() {
        tracing::warn!("failed to show OS notification: {e}");
    }
}

/// Back-fill events missed while offline (docs/protocol-v1.md §2).
/// The watermark is the newest server event we have; subtract one second so
/// boundary events sharing a truncated timestamp are re-fetched (dedup by id
/// absorbs the overlap).
async fn backfill(conn: &MobileConn) -> Result<(), String> {
    let cfg = conn.state.config.read().await.server.clone();
    let base = http_base(&cfg.server_url)?;
    let mut url = format!("{base}/api/v1/events");
    let since = {
        let ts = conn.last_server_ts.read().await;
        ts.map(|t| (t - chrono::Duration::seconds(1)).to_rfc3339())
    };
    if let Some(ts) = since {
        url = format!("{url}?since={ts}");
    }
    let resp = conn
        .client
        .get(&url)
        .bearer_auth(&cfg.api_token)
        .send()
        .await
        .map_err(|e| format!("back-fill request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("back-fill rejected: HTTP {}", resp.status()));
    }
    let body: EventsResponse = resp
        .json()
        .await
        .map_err(|e| format!("bad back-fill response: {e}"))?;
    for event in body.events {
        ingest(conn, event).await;
    }
    Ok(())
}

async fn run_connection(conn: MobileConn, mut stop: tokio::sync::oneshot::Receiver<()>) {
    let mut backoff = 1u64;
    loop {
        set_status(&conn, ConnState::Connecting, None).await;
        let result = connect_once(&conn, &mut stop).await;
        match result {
            Ok(()) => backoff = 1,
            Err(e) => {
                tracing::warn!("alert server connection ended: {e}");
                set_status(&conn, ConnState::Offline, Some(e)).await;
            }
        }
        // Stop quickly if disconnect was requested; otherwise back off.
        tokio::select! {
            _ = &mut stop => {
                set_status(&conn, ConnState::Offline, None).await;
                return;
            }
            _ = tokio::time::sleep(Duration::from_secs(backoff)) => {}
        }
        backoff = (backoff * 2).min(MAX_BACKOFF_SECS);
    }
}

async fn connect_once(
    conn: &MobileConn,
    stop: &mut tokio::sync::oneshot::Receiver<()>,
) -> Result<(), String> {
    ensure_registered(conn).await?;
    let (server_url, ws_token) = {
        let cfg = conn.state.config.read().await.server.clone();
        (cfg.server_url.clone(), cfg.ws_token.clone())
    };
    if ws_token.is_empty() {
        return Err("device not registered (no ws_token)".to_string());
    }
    let url = ws_url(&server_url, &ws_token)?;
    // Never log the URL itself — it carries the ws_token credential.
    tracing::info!("connecting to alert server: {server_url}{WS_PATH}");

    let (stream, _response) = tokio::time::timeout(
        Duration::from_secs(CONNECT_TIMEOUT_SECS),
        connect_async(&url),
    )
    .await
    .map_err(|_| "ws connect timed out".to_string())?
    .map_err(|e| format!("ws connect failed: {e}"))?;
    let (mut write, mut read) = stream.split();

    // The server speaks first (hello); answer with our own hello.
    let hello = ClientMessage::Hello {
        v: 1,
        platform: PLATFORM,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&hello)
                .map_err(|e| e.to_string())?
                .into(),
        ))
        .await
        .map_err(|e| format!("ws send failed: {e}"))?;

    set_status(conn, ConnState::Online, None).await;

    loop {
        // Graceful disconnect: a stop request wins over waiting for traffic.
        let next = tokio::select! {
            biased;
            _ = &mut *stop => return Ok(()),
            msg = tokio::time::timeout(Duration::from_secs(READ_TIMEOUT_SECS), read.next()) => msg,
        };
        let msg = match next {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(e))) => return Err(format!("ws error: {e}")),
            Ok(None) => return Err("ws closed by server".to_string()),
            Err(_) => return Err("ws read timeout (no pings)".to_string()),
        };

        match msg {
            tokio_tungstenite::tungstenite::Message::Text(text) => {
                let parsed: ServerMessage =
                    serde_json::from_str(&text).map_err(|e| format!("bad server message: {e}"))?;
                match parsed {
                    ServerMessage::Hello {
                        missed_events,
                        server_time,
                        ..
                    } => {
                        tracing::info!("server hello at {server_time}, missed={missed_events}");
                        if missed_events > 0 {
                            if let Err(e) = backfill(conn).await {
                                tracing::warn!("back-fill failed: {e}");
                            }
                        }
                    }
                    ServerMessage::Notification { event, .. } => {
                        let id = event.id.clone();
                        ingest(conn, event).await;
                        let ack = ClientMessage::Ack { v: 1, ids: vec![id] };
                        let _ = write
                            .send(tokio_tungstenite::tungstenite::Message::Text(
                                serde_json::to_string(&ack).unwrap_or_default().into(),
                            ))
                            .await;
                    }
                    ServerMessage::Ping { ts, .. } => {
                        let pong = ClientMessage::Pong { v: 1, ts };
                        write
                            .send(tokio_tungstenite::tungstenite::Message::Text(
                                serde_json::to_string(&pong).unwrap_or_default().into(),
                            ))
                            .await
                            .map_err(|e| format!("ws send failed: {e}"))?;
                    }
                }
            }
            tokio_tungstenite::tungstenite::Message::Close(_) => {
                return Err("ws close frame received".to_string());
            }
            _ => {}
        }
    }
}

/// Start (or restart) the connection task. Async: `tokio::sync::Mutex` must
/// never be `blocking_lock`ed from an async context — it panics.
async fn connect(conn: &MobileConn) {
    // Nothing to do until the user has configured a server.
    {
        let cfg = conn.state.config.read().await;
        if cfg.server.server_url.is_empty() {
            set_status(conn, ConnState::Offline, None).await;
            return;
        }
    }
    // If a task is already running, stop it cleanly first. Take and replace
    // the stop sender under a single guard so two overlapping connect calls
    // can't both spawn a live connection task.
    let (tx, rx) = tokio::sync::oneshot::channel();
    {
        let mut guard = conn.stop_tx.lock().await;
        if let Some(old) = guard.take() {
            let _ = old.send(());
        }
        *guard = Some(tx);
    }
    tauri::async_runtime::spawn(run_connection(conn.clone(), rx));
}

async fn disconnect(conn: &MobileConn) {
    if let Some(tx) = conn.stop_tx.lock().await.take() {
        let _ = tx.send(());
    }
}

/// Called from `RunEvent::Resumed` on mobile: reconnect when the app returns
/// to the foreground (the OS suspends sockets while backgrounded).
#[cfg(mobile)]
pub fn on_resumed(app: &tauri::AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let auto = match get_conn(&handle) {
            Ok(c) => c.state.config.read().await.server.auto_connect,
            Err(_) => false,
        };
        if auto {
            if let Ok(c) = get_conn(&handle) {
                connect(&c).await;
            }
        }
    });
}

// --- Tauri commands ---

#[tauri::command]
pub async fn mobile_server_status(app: tauri::AppHandle) -> Result<ServerStatus, String> {
    Ok(get_conn(&app)?.status.read().await.clone())
}

#[tauri::command]
pub async fn mobile_server_register(
    app: tauri::AppHandle,
    server_url: String,
    api_token: String,
) -> Result<(), String> {
    let conn = get_conn(&app)?;
    {
        let mut cfg = conn.state.config.write().await;
        cfg.server.server_url = server_url.trim_end_matches('/').to_string();
        cfg.server.api_token = api_token;
        // New credentials invalidate any previous registration.
        cfg.server.device_id.clear();
        cfg.server.ws_token.clear();
        persistence::save(&conn.state.config_dir, &cfg);
    }
    {
        let mut status = conn.status.write().await;
        status.server_url = server_url.trim_end_matches('/').to_string();
        status.device_id.clear();
    }
    // Register immediately so status reflects the device id.
    ensure_registered(&conn).await
}

#[tauri::command]
pub async fn mobile_server_connect(app: tauri::AppHandle) -> Result<(), String> {
    let c = get_conn(&app)?;
    connect(&c).await;
    Ok(())
}

#[tauri::command]
pub async fn mobile_server_disconnect(app: tauri::AppHandle) -> Result<(), String> {
    let c = get_conn(&app)?;
    disconnect(&c).await;
    set_status(&c, ConnState::Offline, None).await;
    Ok(())
}

#[tauri::command]
pub async fn mobile_server_unregister(app: tauri::AppHandle) -> Result<(), String> {
    let conn = get_conn(&app)?;
    disconnect(&conn).await;
    let (device_id, server_url, api_token) = {
        let mut cfg = conn.state.config.write().await;
        let triple = (
            cfg.server.device_id.clone(),
            cfg.server.server_url.clone(),
            cfg.server.api_token.clone(),
        );
        cfg.server.device_id.clear();
        cfg.server.ws_token.clear();
        cfg.server.push_token.clear();
        persistence::save(&conn.state.config_dir, &cfg);
        triple
    };
    if let (Ok(base), device_id) = (http_base(&server_url), device_id) {
        if !device_id.is_empty() {
            let resp = conn
                .client
                .delete(format!("{base}/api/v1/devices/{device_id}"))
                .bearer_auth(api_token)
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() => {}
                Ok(r) => tracing::warn!("device unregister rejected: HTTP {}", r.status()),
                Err(e) => tracing::warn!("device unregister request failed: {e}"),
            }
        }
    }
    conn.status.write().await.device_id.clear();
    Ok(())
}

#[tauri::command]
pub async fn mobile_push_token_update(
    app: tauri::AppHandle,
    token: String,
) -> Result<(), String> {
    upload_push_token(app, token).await
}
