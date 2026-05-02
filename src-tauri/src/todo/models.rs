use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub completed: bool,
    #[serde(default)]
    pub due_date: String,
    pub created_at: String,
    #[serde(default)]
    pub source: TodoSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TodoSource {
    #[default]
    Manual,
    Pull,
    Push {
        remote_addr: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullTodoResponse {
    pub todos: Vec<TodoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushTodoRequest {
    pub text: String,
    #[serde(default)]
    pub due_date: String,
}
