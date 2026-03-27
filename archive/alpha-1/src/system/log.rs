use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use uuid::Uuid;
use std::collections::VecDeque;

/// Types of events recorded in the TOS Log (§14.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogType {
    Lifecycle,      // Surface creation, focus, close
    Command,        // Shell commands executed
    Inspection,     // Level 4/5 access
    Telemetry,      // Resource snapshots
    Collaboration,  // Guest actions, joins/leaves
    System,         // Notifications, alerts
    Priority,       // Priority score updates
    Ai,            // AI interactions
    Security,       // Authentication, deeper inspection toggle
}

/// A single entry in the TOS Log (§14.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Local>,
    pub region: String, // "Global", "Sector:Name", "App:ID"
    pub event_type: LogType,
    pub message: String,
    pub details: Option<serde_json::Value>, // Structured data
    pub surface_id: Option<String>,
}

/// Centralized Log Manager (§14)
#[derive(Debug)]
pub struct LogManager {
    pub entries: VecDeque<LogEntry>,
    pub max_entries: usize,
    pub enabled: bool,
}

impl Default for LogManager {
    fn default() -> Self {
        Self {
            entries: VecDeque::with_capacity(1000),
            max_entries: 1000,
            enabled: true,
        }
    }
}

impl LogManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log(&mut self, event_type: LogType, region: &str, message: &str, details: Option<serde_json::Value>) {
        if !self.enabled && event_type != LogType::Security {
            return;
        }

        let entry = LogEntry {
            id: Uuid::new_v4(),
            timestamp: Local::now(),
            region: region.to_string(),
            event_type,
            message: message.to_string(),
            details,
            surface_id: None, // Can be populated if context known
        };

        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        
        // In a real implementation, we would also append to ~/.local/share/tos/logs/ (§14.3)
        self.entries.push_back(entry);
    }

    pub fn query(&self, filter: &str) -> Vec<&LogEntry> {
        // Basic contains search for now (§14.2)
        let filter_lower = filter.to_lowercase();
        self.entries.iter()
            .filter(|e| e.message.to_lowercase().contains(&filter_lower) || e.region.to_lowercase().contains(&filter_lower))
            .collect()
    }
}
