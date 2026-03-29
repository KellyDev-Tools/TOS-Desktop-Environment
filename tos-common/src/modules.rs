//! Module trait contracts for AI, Shell, and Terminal Output modules.
//!
//! These traits define the runtime API that installable modules must
//! implement. The Brain loads modules dynamically and calls them through
//! these trait objects.

use crate::{TerminalContext, TerminalLine};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// AI Module Contract
// ---------------------------------------------------------------------------

/// Request payload for an AI backend query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiQuery {
    pub prompt: String,
    pub context: Vec<String>,
    pub stream: bool,
}

/// Response payload from an AI backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub id: Uuid,
    pub choice: AiChoice,
    pub usage: AiUsage,
    pub status: AiStatus,
}

/// The model's generated response content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChoice {
    pub role: String,
    pub content: String,
}

/// Token usage metadata for billing and debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsage {
    pub tokens: u32,
}

/// Stream or completion status for an AI response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiStatus {
    #[serde(rename = "streaming")]
    Streaming,
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "error")]
    Error(String),
}

/// Runtime contract for an installable AI backend module.
///
/// AI backends handle the actual LLM communication — model selection,
/// endpoint management, authentication, and streaming.
pub trait AiModule: Send + Sync {
    /// Submit a query and receive a synchronous response.
    fn query(&self, request: AiQuery) -> anyhow::Result<AiResponse>;
    /// Human-readable backend name.
    fn name(&self) -> &str;
    /// List of capabilities this backend supports (e.g. "chat", "function_calling").
    fn capabilities(&self) -> &[String];
}

// ---------------------------------------------------------------------------
// Shell Module Contract
// ---------------------------------------------------------------------------

/// Configuration for a shell module's OSC integration capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShellIntegration {
    /// Supports OSC 7 (current working directory reporting).
    pub osc_directory: bool,
    /// Supports OSC 9004 (command result / JSON context export).
    pub osc_command_result: bool,
    /// Supports OSC suggestion sequences.
    pub osc_suggestions: bool,
}

/// Runtime contract for an installable shell module.
///
/// Shell modules wrap a specific shell binary (fish, zsh, bash, etc.)
/// and declare which OSC integration features they support.
pub trait ShellModule: Send + Sync {
    /// Path to the shell executable.
    fn get_executable_path(&self) -> &std::path::Path;
    /// Default command-line arguments for the shell.
    fn get_default_args(&self) -> &[String];
    /// OSC integration capabilities.
    fn get_integration_config(&self) -> &ShellIntegration;
}

// ---------------------------------------------------------------------------
// Terminal Output Module Contract
// ---------------------------------------------------------------------------

/// Runtime contract for an installable terminal output rendering module.
///
/// Terminal output modules control how terminal lines are visually
/// presented — rectangular grids, cinematic triangular layouts, etc.
pub trait TerminalOutputModule: Send + Sync {
    /// Initialize the module with a rendering context and configuration.
    fn init(&mut self, context: TerminalContext, config: serde_json::Value);
    /// Push new lines into the module's output buffer.
    fn push_lines(&mut self, lines: Vec<TerminalLine>);
    /// Return the module's unique identifier.
    fn get_id(&self) -> &str;
}
pub mod sandbox;
