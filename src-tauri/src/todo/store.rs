use std::fs;
use std::path::PathBuf;

use crate::todo::models::TodoItem;

pub struct TodoStore {
    path: PathBuf,
    items: Vec<TodoItem>,
}

impl TodoStore {
    pub fn new(config_dir: &PathBuf) -> Self {
        let path = config_dir.join("todos.json");
        let items = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { path, items }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.items) {
            let _ = fs::write(&self.path, json);
        }
    }

    pub fn list(&self) -> Vec<TodoItem> {
        self.items.clone()
    }

    pub fn add(&mut self, item: TodoItem) {
        self.items.push(item);
        self.save();
    }

    pub fn toggle(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|t| t.id == id) {
            item.completed = !item.completed;
            self.save();
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.items.retain(|t| t.id != id);
        self.save();
    }

    pub fn merge_pulled(&mut self, incoming: Vec<TodoItem>) -> usize {
        let mut count = 0;
        for item in incoming {
            if !self.items.iter().any(|e| e.text == item.text && e.due_date == item.due_date) {
                self.items.push(item);
                count += 1;
            }
        }
        if count > 0 {
            self.save();
        }
        count
    }
}
