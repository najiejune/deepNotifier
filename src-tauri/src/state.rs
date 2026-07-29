use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::config::schema::AppConfig;
use crate::history::store::NotificationHistory;
use crate::notifier::dispatcher::NotificationEvent;
use crate::timer::engine::TimerState;
use crate::todo::store::TodoStore;

/// A snapshot of CLI installation status for a given project directory.
pub type CliStatusCache = Arc<RwLock<std::collections::HashMap<String, bool>>>;

/// Tracks an in-flight CLI tool approval for timeout detection.
#[derive(Debug, Clone)]
pub struct ApprovalSession {
    pub start: Instant,
    pub pid: u32,
    pub cli_id: String,
    pub cli_name: String,
}

pub type ApprovalSessions = Arc<RwLock<std::collections::HashMap<String, ApprovalSession>>>;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub config_dir: PathBuf,
    pub notification_tx: tokio::sync::mpsc::Sender<NotificationEvent>,
    pub notification_rx: Arc<RwLock<Option<tokio::sync::mpsc::Receiver<NotificationEvent>>>>,
    pub dnd_active: Arc<RwLock<bool>>,
    pub history: Arc<RwLock<NotificationHistory>>,
    pub timer_state: Arc<RwLock<TimerState>>,
    pub timer_cancel: Arc<RwLock<Option<tokio::sync::oneshot::Sender<()>>>>,
    pub todo_store: Arc<RwLock<TodoStore>>,
    pub cli_installed_cache: CliStatusCache,
    /// Last notification's source process PID — used for click-to-focus
    pub pending_pid: Arc<RwLock<Option<u32>>>,
    /// In-flight approval sessions keyed by session_id
    pub approval_sessions: ApprovalSessions,
}
