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
use std::collections::HashMap;
use serde_json::json;
use async_trait::async_trait;

/// AI Assistant Manager implementation (§3.5, §11)
#[derive(Default)]
pub struct AiManager {
    pub active_backend: Option<String>,
    pub backends: HashMap<String, Box<dyn AiBackend>>,
    pub history: Vec<AiMessage>,
    pub is_generating: bool,
}

impl Debug for AiManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiManager")
            .field("active_backend", &self.active_backend)
            .field("history_len", &self.history.len())
            .field("is_generating", &self.is_generating)
            .finish()
    }
}

impl AiManager {
    pub fn new() -> Self {
        let mut manager = Self {
            active_backend: None,
            backends: HashMap::new(),
            history: Vec::new(),
            is_generating: false,
        };
        
        // Register default Ollama backend
        let ollama = OllamaBackend::new("http://localhost:11434");
        manager.register_backend(Box::new(ollama));
        manager.active_backend = Some("ollama".to_string());
        
        manager
    }

    pub fn register_backend(&mut self, backend: Box<dyn AiBackend>) {
        let name = backend.metadata().name.to_lowercase();
        self.backends.insert(name, backend);
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
        
        tracing::info!("AI Query submitted: {}", query);
        
        // In a real async environment, we would spawn a task here to call process_query
        // For now, valid dispatch logic structure is in place
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
#[async_trait]
pub trait AiBackend: Debug + Send + Sync {
    fn metadata(&self) -> AiBackendMetadata;
    async fn process_query(&self, history: &[AiMessage]) -> Result<String, String>;
    async fn stream_query(&self, history: &[AiMessage], callback: Box<dyn Fn(String) + Send + Sync>) -> Result<(), String>;
}

/// Default Ollama Backend Implementation
#[derive(Debug, Clone)]
pub struct OllamaBackend {
    pub base_url: String,
    pub model: String,
}

impl OllamaBackend {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            model: "llama3".to_string(), // Default model
        }
    }
}

#[async_trait]
impl AiBackend for OllamaBackend {
    fn metadata(&self) -> AiBackendMetadata {
        AiBackendMetadata {
            name: "Ollama".to_string(),
            version: "0.1.0".to_string(),
            description: "Local AI backend via Ollama".to_string(),
            icon: "🧠".to_string(), // visual indicator
            capabilities: AiCapabilities {
                chat: true,
                function_calling: false,
                vision: false,
                streaming: true,
            },
            provider: "local".to_string(),
        }
    }

    async fn process_query(&self, history: &[AiMessage]) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/chat", self.base_url);
        
        let messages: Vec<_> = history.iter().map(|msg| {
            json!({
                "role": msg.role,
                "content": msg.content
            })
        }).collect();

        let body = json!({
            "model": self.model,
            "messages": messages,
            "stream": false
        });

        match client.post(&url).json(&body).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(content) = json["message"]["content"].as_str() {
                                Ok(content.to_string())
                            } else {
                                Err("Invalid response format from Ollama".to_string())
                            }
                        },
                        Err(e) => Err(format!("Failed to parse response: {}", e)),
                    }
                } else {
                    Err(format!("Ollama API error: {}", resp.status()))
                }
            },
            Err(e) => Err(format!("Request failed: {}", e)),
        }
    }

    async fn stream_query(&self, _history: &[AiMessage], _callback: Box<dyn Fn(String) + Send + Sync>) -> Result<(), String> {
        // Streaming implementation would go here using reqwest::Response::bytes_stream
        Err("Streaming not yet implemented for Ollama backend".to_string())
    }
}
