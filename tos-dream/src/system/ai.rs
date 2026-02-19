use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Capabilities of an AI backend (§16.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCapabilities {
    pub chat: bool,
    pub function_calling: bool,
    pub vision: bool,
    pub streaming: bool,
}

/// Metadata for an AI backend (§16.4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBackendMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub icon: String,
    pub capabilities: AiCapabilities,
    pub provider: String,
}

/// A message in an AI conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub id: Uuid,
    pub role: String, // "user", "assistant", "system"
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// AI Assistant Manager implementation (§3.5, §11)
#[derive(Debug, Default)]
pub struct AiManager {
    pub active_backend: Option<String>,
    pub history: Vec<AiMessage>,
    pub is_generating: bool,
}

impl AiManager {
    pub fn new() -> Self {
        Self {
            active_backend: None,
            history: Vec::new(),
            is_generating: false,
        }
    }

    pub fn submit_query(&mut self, query: &str) {
        let msg = AiMessage {
            id: Uuid::new_v4(),
            role: "user".to_string(),
            content: query.to_string(),
            timestamp: chrono::Local::now(),
        };
        self.history.push(msg);
        self.is_generating = true;
        
        // Logic for backend dispatch will go here
        tracing::info!("AI Query submitted: {}", query);
    }

    pub fn stop_generation(&mut self) {
        self.is_generating = false;
        tracing::info!("AI Generation interrupted");
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// Trait for pluggable AI backends (§7.5, §14.4)
pub trait AiBackend: Debug + Send + Sync {
    fn metadata(&self) -> AiBackendMetadata;
    fn process_query(&self, history: &[AiMessage]) -> Result<String, String>;
    fn stream_query(&self, history: &[AiMessage], callback: Box<dyn Fn(String) + Send>) -> Result<(), String>;
}
