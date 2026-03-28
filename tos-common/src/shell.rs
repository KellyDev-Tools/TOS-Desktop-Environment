use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OscEvent {
    Priority(u8),
    Cwd(String),
    DirectoryListing(crate::state::DirectoryListing),
    CommandResult {
        command: String,
        status: i32,
        output: Option<String>,
    },
    JsonContext(serde_json::Value),
}

pub struct OscParser {
    pub current_priority: u8,
}

impl OscParser {
    pub fn new() -> Self {
        Self { current_priority: 0 }
    }

    pub fn process(&mut self, line: &str) -> (String, Vec<OscEvent>) {
        let mut events = Vec::new();
        let mut clean_text = line.to_string();

        if line.starts_with("\x1b]50;") {
            if let Some(end) = line.find('\x07') {
                let payload = &line[5..end];
                if let Ok(p) = payload.parse::<u8>() {
                    events.push(OscEvent::Priority(p));
                    clean_text = line[end+1..].to_string();
                }
            }
        } else if line.contains("\x1b]7;") {
             // Handle CWD etc.
        }

        (clean_text, events)
    }
}
