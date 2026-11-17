use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OscEvent {
    Priority(u8),
    /// Per-line priority set via OSC 9012 (§27.4).
    LinePriority(u8),
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

impl Default for OscParser {
    fn default() -> Self {
        Self::new()
    }
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
                if let Some(stripped) = url.strip_prefix("file://") {
                    if let Some(path_start) = stripped.find('/') {
                        let path = percent_decode(&stripped[path_start..]);
                        events.push(OscEvent::Cwd(path));
                    } else {
                        // case file://host (root)
                        events.push(OscEvent::Cwd("/".to_string()));
                    }
                }
                clean_text = format!("{}{}", &line[..pos], &line[actual_end + 1..]);
            }
        }

        // §27.4: OSC 9012 — Line-Level Priority
        if let Some(pos) = clean_text.find("\x1b]9012;") {
            if let Some(end) = clean_text[pos..].find('\x07') {
                let actual_end = pos + end;
                let payload = &clean_text[pos + 7..actual_end];
                if let Ok(p) = payload.trim().parse::<u8>() {
                    if (1..=3).contains(&p) {
                        events.push(OscEvent::LinePriority(p));
                    }
                }
                clean_text = format!("{}{}", &clean_text[..pos], &clean_text[actual_end + 1..]);
            }
        }

        // OSC 9002: Command Result
        if let Some(pos) = clean_text.find("\x1b]9002;") {
            if let Some(end) = clean_text[pos..].find('\x07') {
                let actual_end = pos + end;
                let payload = &clean_text[pos + 7..actual_end];
                let parts: Vec<&str> = payload.split(';').collect();
                if parts.len() >= 2 {
                    if let Ok(status) = parts[1].trim().parse::<i32>() {
                        events.push(OscEvent::CommandResult {
                            command: parts[0].to_string(),
                            status,
                            output: None,
                        });
                    }
                }
                clean_text = format!("{}{}", &clean_text[..pos], &clean_text[actual_end + 1..]);
            }
        }

        // OSC 9004: JSON Context
        if let Some(pos) = clean_text.find("\x1b]9004;") {
            if let Some(end) = clean_text[pos..].find('\x07') {
                let actual_end = pos + end;
                let payload = &clean_text[pos + 7..actual_end];
                if let Ok(decoded) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, payload) {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                        events.push(OscEvent::JsonContext(val));
                    }
                }
                clean_text = format!("{}{}", &clean_text[..pos], &clean_text[actual_end + 1..]);
            }
        }

        (clean_text, events)
    }
}

/// Percent-decode a URL path (OSC 7 file:// URLs encode spaces and
/// other characters, e.g. `/home/user/My%20Docs`). Invalid or
/// truncated escapes are left as-is.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osc7_cwd_is_percent_decoded() {
        let mut parser = OscParser::new();
        let (_clean, events) = parser.process("prompt\x1b]7;file://host/home/user/My%20Docs\x07");
        assert!(
            matches!(&events[..], [OscEvent::Cwd(p)] if p == "/home/user/My Docs"),
            "events: {events:?}"
        );
    }

    #[test]
    fn osc7_plain_path_unchanged() {
        let mut parser = OscParser::new();
        let (_clean, events) = parser.process("\x1b]7;file://host/tmp\x07");
        assert!(
            matches!(&events[..], [OscEvent::Cwd(p)] if p == "/tmp"),
            "events: {events:?}"
        );
    }
}
