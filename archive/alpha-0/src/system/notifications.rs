use std::collections::VecDeque;

// Based on "Notifications.md"

#[derive(Debug, Clone)]
pub enum Priority {
    Low,
    Normal,
    Critical, // "Red Alert" style
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub priority: Priority,
}

pub struct NotificationManager {
    pub queue: VecDeque<Notification>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, title: &str, message: &str, priority: Priority) {
        let n = Notification {
            title: title.to_string(),
            message: message.to_string(),
            priority,
        };
        self.queue.push_back(n);
        println!("[Notification] New alert: {}", title);
    }

    pub fn process_next(&mut self) -> Option<Notification> {
        if let Some(n) = self.queue.pop_front() {
            println!("\n*** INCOMING TRANSMISSION ***");
            println!("Priority: {:?}", n.priority);
            println!("Subject:  {}", n.title);
            println!("Message:  {}", n.message);
            println!("*****************************\n");
            Some(n)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_process() {
        let mut mgr = NotificationManager::new();
        mgr.push("Test Alert", "Something happened", Priority::Normal);
        
        assert_eq!(mgr.queue.len(), 1);
        
        let n = mgr.process_next().unwrap();
        assert_eq!(n.title, "Test Alert");
        assert_eq!(n.message, "Something happened");
        assert_eq!(mgr.queue.len(), 0);
    }

    #[test]
    fn test_fifo_order() {
        let mut mgr = NotificationManager::new();
        mgr.push("First", "1", Priority::Low);
        mgr.push("Second", "2", Priority::Normal);
        mgr.push("Third", "3", Priority::Critical);

        assert_eq!(mgr.process_next().unwrap().title, "First");
        assert_eq!(mgr.process_next().unwrap().title, "Second");
        assert_eq!(mgr.process_next().unwrap().title, "Third");
        assert!(mgr.process_next().is_none());
    }

    #[test]
    fn test_empty_queue_returns_none() {
        let mut mgr = NotificationManager::new();
        assert!(mgr.process_next().is_none());
    }
}
