use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::schema::AppConfig;
use crate::history::store::NotificationHistory;
use crate::notifier::dispatcher::NotificationEvent;
use crate::timer::engine::TimerState;
use crate::todo::store::TodoStore;

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
}
