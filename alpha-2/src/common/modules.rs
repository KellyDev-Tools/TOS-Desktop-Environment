use serde::{Deserialize, Serialize};
use crate::common::TerminalLine;
use uuid::Uuid;

/// §1.3.1: AI Module API Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiQuery {
    pub prompt: String,
    pub context: Vec<String>,
    pub stream: bool,
}

/// §1.3.1: AI Module API Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub id: Uuid,
    pub choice: AiChoice,
    pub usage: AiUsage,
    pub status: AiStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChoice {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsage {
    pub tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiStatus {
    #[serde(rename = "streaming")]
    Streaming,
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "error")]
    Error(String),
}

/// §1.7: Shell Module Configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShellIntegration {
    pub osc_directory: bool,
    pub osc_command_result: bool,
    pub osc_suggestions: bool,
}

/// §1.5.1: Terminal Output Module Logic Contract
pub trait TerminalOutputModule {
    fn init(&mut self, context: super::TerminalContext, config: serde_json::Value);
    fn push_lines(&mut self, lines: Vec<TerminalLine>);
    fn get_id(&self) -> &str;
}

/// §1.7: Shell Module Logic Contract
pub trait ShellModule {
    fn get_executable_path(&self) -> &std::path::Path;
    fn get_default_args(&self) -> &[String];
    fn get_integration_config(&self) -> &ShellIntegration;
}

/// §1.3: AI Backend Logic Contract
pub trait AiModule {
    fn query(&self, request: AiQuery) -> anyhow::Result<AiResponse>;
    fn name(&self) -> &str;
    fn capabilities(&self) -> &[String];
}
