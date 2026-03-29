use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OscEvent {
    Priority(u8),
    Cwd(String),
    DirectoryListing(crate::DirectoryListing),
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
        Self {
            current_priority: 0,
        }
    }

    pub fn process(&mut self, line: &str) -> (String, Vec<OscEvent>) {
        let mut events = Vec::new();
        let mut clean_text = line.to_string();

        // §15.2: Metadata Interception (TOS OSC Extensions)
        // Format: OSC 50 ; <priority_digit> [; <optional_payload>] ST
        if let Some(pos) = line.find("\x1b]50;") {
            if let Some(end) = line[pos..].find('\x07') {
                let actual_end = pos + end;
                let payload = &line[pos + 5..actual_end];
                if let Some((p_str, rest)) = payload.split_once(';') {
                    if let Ok(p) = p_str.parse::<u8>() {
                        events.push(OscEvent::Priority(p));
                        self.current_priority = p;
                        clean_text = format!("{}{}", &line[..pos], rest);
                    }
                } else if let Ok(p) = payload.parse::<u8>() {
                    events.push(OscEvent::Priority(p));
                    self.current_priority = p;
                    clean_text = format!("{}{}", &line[..pos], &line[actual_end + 1..]);
                }
            }
        }

        // OSC 7: Current Working Directory
        // Format: OSC 7 ; file://hostname/path ST
        if let Some(pos) = line.find("\x1b]7;") {
            if let Some(end) = line[pos..].find('\x07') {
                let actual_end = pos + end;
                let url = &line[pos + 4..actual_end];
                if url.starts_with("file://") {
                    if let Some(path_start) = url[7..].find('/') {
                        let path = &url[7 + path_start..];
                        events.push(OscEvent::Cwd(path.to_string()));
                    } else {
                        // case file://host (root)
                        events.push(OscEvent::Cwd("/".to_string()));
                    }
                }
                clean_text = format!("{}{}", &line[..pos], &line[actual_end + 1..]);
            }
        }

        (clean_text, events)
    }
}
