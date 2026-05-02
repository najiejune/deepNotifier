use crate::notifier::dispatcher::NotificationEvent;

pub struct NotificationHistory {
    events: Vec<NotificationEvent>,
    max_size: usize,
}

impl NotificationHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, event: NotificationEvent) {
        if self.events.len() >= self.max_size {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    pub fn iter(&self) -> impl Iterator<Item = &NotificationEvent> {
        self.events.iter().rev()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
