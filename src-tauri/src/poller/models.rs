use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResponse {
    pub status_code: u16,
    pub content_hash: u64,
    pub body: String,
    pub timestamp: String,
}
