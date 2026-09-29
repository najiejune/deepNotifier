//! Client-server protocol v1 types for the deepNotifier alert server.
//!
//! Documented in `docs/protocol-v1.md`. The server implementation lives
//! outside this repository; these types are the contract both sides code
//! against.

use serde::{Deserialize, Serialize};

use crate::models::NotificationEvent;

pub const PROTOCOL_VERSION: u8 = 1;

fn default_v() -> u8 {
    PROTOCOL_VERSION
}

/// Messages sent by the server over the WebSocket channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// First message after connect. `missed_events` tells the client how many
    /// events arrived while it was offline; the client then back-fills via
    /// `GET /api/v1/events?since=<last local timestamp>`.
    Hello {
        #[serde(default = "default_v")]
        v: u8,
        server_time: String,
        missed_events: u64,
    },
    /// A notification event. `event.raw_payload` is always null on this
    /// channel (size control); fetch details via the events endpoint.
    Notification {
        #[serde(default = "default_v")]
        v: u8,
        event: NotificationEvent,
    },
    /// Server heartbeat. The client must answer with `pong`; two unanswered
    /// pings and the server drops the connection.
    Ping {
        #[serde(default = "default_v")]
        v: u8,
        ts: u64,
    },
}

/// Messages sent by the client over the WebSocket channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Client greeting, answers the server's `hello`.
    Hello {
        #[serde(default = "default_v")]
        v: u8,
        platform: Platform,
        app_version: String,
    },
    /// Answers a server `ping`.
    Pong {
        #[serde(default = "default_v")]
        v: u8,
        ts: u64,
    },
    /// Delivery acknowledgement for received notification ids.
    Ack {
        #[serde(default = "default_v")]
        v: u8,
        ids: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Android,
    Ios,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceRequest {
    pub platform: Platform,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_token: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceResponse {
    pub device_id: String,
    pub ws_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePushTokenRequest {
    pub push_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub platform: Platform,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub push_token: Option<String>,
    #[serde(default)]
    pub last_seen: Option<String>,
    #[serde(default)]
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsResponse {
    pub events: Vec<NotificationEvent>,
}
