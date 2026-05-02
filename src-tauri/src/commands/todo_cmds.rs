use crate::state::AppState;
use crate::todo::models::{TodoItem, TodoSource};
use tauri::State;

#[tauri::command]
pub async fn get_todos(state: State<'_, AppState>) -> Result<Vec<TodoItem>, String> {
    let store = state.todo_store.read().await;
    Ok(store.list())
}

#[tauri::command]
pub async fn add_todo(
    state: State<'_, AppState>,
    text: String,
    due_date: Option<String>,
) -> Result<TodoItem, String> {
    let item = TodoItem {
        id: uuid::Uuid::new_v4().to_string(),
        text,
        completed: false,
        due_date: due_date.unwrap_or_default(),
        created_at: chrono::Local::now().to_rfc3339(),
        source: TodoSource::Manual,
    };
    state.todo_store.write().await.add(item.clone());
    Ok(item)
}

#[tauri::command]
pub async fn toggle_todo(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.todo_store.write().await.toggle(&id);
    Ok(())
}

#[tauri::command]
pub async fn delete_todo(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.todo_store.write().await.remove(&id);
    Ok(())
}
