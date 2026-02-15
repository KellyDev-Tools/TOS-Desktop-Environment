//! Voice Command System
//! 
//! Provides natural language command processing for TOS.
//! 
//! ## Features
//! - Wake word detection ("Computer")
//! - Natural language command parsing
//! - Confidence scoring
//! - Integration with semantic event system
//! 
//! ## Supported Commands
//! - Navigation: "zoom in", "zoom out", "go back", "show overview"
//! - Mode switching: "command mode", "directory mode", "activity mode"
//! - System: "reset", "toggle bezel", "split view"
//! - Mini-map: "activate mini-map", "close mini-map"
//! 
//! ## Architecture
//! The voice system operates in stages:
//! 1. **Listening**: Wake word detection (always on, low power)
//! 2. **Recording**: Capture audio after wake word
//! 3. **Processing**: Speech-to-text conversion
//! 4. **Parsing**: Extract semantic intent
//! 5. **Execution**: Map to semantic events

use crate::system::input::SemanticEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Voice command confidence level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Low confidence - may need confirmation
    Low,
    /// Medium confidence - execute with feedback
    Medium,
    /// High confidence - execute silently
    High,
}

impl ConfidenceLevel {
    /// Create from confidence score (0.0 - 1.0)
    pub fn from_score(score: f32) -> Self {
        if score >= 0.85 {
            ConfidenceLevel::High
        } else if score >= 0.6 {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        }
    }
}

/// State of the voice command system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceState {
    /// Idle, listening for wake word
    Idle,
    /// Wake word detected, listening for command
    Listening { start_time: Instant },
    /// Processing speech-to-text
    Processing,
    /// Command parsed, ready to execute
    Ready,
    /// Executing the command
    Executing,
    /// Error occurred
    Error(VoiceError),
}

/// Configuration for voice command system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// Wake word phrase
    pub wake_word: String,
    /// Maximum command duration in seconds
    pub max_command_duration_secs: u64,
    /// Confidence threshold for auto-execution
    pub auto_execute_threshold: f32,
    /// Whether to show visual feedback
    pub visual_feedback: bool,
    /// Whether to provide audio feedback
    pub audio_feedback: bool,
    /// Microphone device index (None for default)
    pub microphone_index: Option<usize>,
    /// Language code (e.g., "en-US")
    pub language: String,
    /// Custom command mappings
    pub custom_commands: HashMap<String, String>,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        let mut custom_commands = HashMap::new();
        custom_commands.insert("beam me up".to_string(), "zoom_in".to_string());
        custom_commands.insert("engage".to_string(), "execute".to_string());
        custom_commands.insert("red alert".to_string(), "tactical_reset".to_string());
        
        Self {
            wake_word: "computer".to_string(),
            max_command_duration_secs: 5,
            auto_execute_threshold: 0.7,
            visual_feedback: true,
            audio_feedback: true,
            microphone_index: None,
            language: "en-US".to_string(),
            custom_commands,
        }
    }
}

/// A recognized voice command
#[derive(Debug, Clone)]
pub struct VoiceCommand {
    /// Raw text from speech recognition
    pub raw_text: String,
    /// Parsed semantic event
    pub event: SemanticEvent,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Confidence level
    pub confidence_level: ConfidenceLevel,
    /// Processing duration
    pub processing_duration: Duration,
}

/// The Voice Command Processor
#[derive(Debug)]
pub struct VoiceCommandProcessor {
    /// Configuration
    pub config: VoiceConfig,
    /// Current state
    pub state: VoiceState,
    /// Last recognized command
    pub last_command: Option<VoiceCommand>,
    /// Command history
    pub command_history: Vec<VoiceCommand>,
    /// Maximum history size
    max_history: usize,
    /// Wake word detector (placeholder for actual implementation)
    pub wake_word_active: bool,
    /// Audio buffer for command recording
    audio_buffer: Vec<f32>,
}

impl Default for VoiceCommandProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceCommandProcessor {
    /// Create a new voice command processor with default config
    pub fn new() -> Self {
        Self::with_config(VoiceConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: VoiceConfig) -> Self {
        Self {
            config,
            state: VoiceState::Idle,
            last_command: None,
            command_history: Vec::new(),
            max_history: 50,
            wake_word_active: false,
            audio_buffer: Vec::new(),
        }
    }

    /// Start listening for wake word
    pub fn start(&mut self) -> Result<(), VoiceError> {
        self.state = VoiceState::Idle;
        self.wake_word_active = true;
        // In real implementation, this would initialize microphone
        Ok(())
    }

    /// Stop listening
    pub fn stop(&mut self) {
        self.state = VoiceState::Idle;
        self.wake_word_active = false;
        self.audio_buffer.clear();
    }

    /// Check if currently listening for commands
    pub fn is_listening(&self) -> bool {
        matches!(self.state, VoiceState::Listening { .. })
    }

    /// Check if processing a command
    pub fn is_processing(&self) -> bool {
        matches!(self.state, VoiceState::Processing)
    }

    /// Simulate wake word detection (for testing)
    pub fn simulate_wake_word(&mut self) {
        if matches!(self.state, VoiceState::Idle) {
            self.state = VoiceState::Listening {
                start_time: Instant::now(),
            };
        }
    }

    /// Process audio input (placeholder for actual STT)
    /// In a real implementation, this would use whisper-rs or similar
    pub fn process_audio(&mut self, audio_samples: &[f32]) -> Option<VoiceCommand> {
        match self.state {
            VoiceState::Listening { start_time } => {
                // Check for timeout
                if start_time.elapsed().as_secs() > self.config.max_command_duration_secs {
                    self.state = VoiceState::Idle;
                    return None;
                }

                // Accumulate audio
                self.audio_buffer.extend_from_slice(audio_samples);
                
                // In real implementation, would feed to STT engine
                // For now, return None to continue listening
                None
            }
            _ => None,
        }
    }

    /// Process text command directly (for testing or text input)
    pub fn process_text(&mut self, text: &str) -> Option<VoiceCommand> {
        self.state = VoiceState::Processing;
        
        let start = Instant::now();
        let (event, confidence) = self.parse_command(text);
        let duration = start.elapsed();
        
        let command = VoiceCommand {
            raw_text: text.to_string(),
            event,
            confidence,
            confidence_level: ConfidenceLevel::from_score(confidence),
            processing_duration: duration,
        };

        self.last_command = Some(command.clone());
        self.add_to_history(command.clone());
        
        self.state = VoiceState::Ready;
        Some(command)
    }

    /// Parse text into semantic event and confidence
    pub fn parse_command(&self, text: &str) -> (SemanticEvent, f32) {
        let text = text.to_lowercase();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        // Check custom commands first
        for (phrase, action) in &self.config.custom_commands {
            if text.contains(phrase) {
                if let Some(event) = self.action_to_event(action) {
                    return (event, 0.95); // High confidence for custom commands
                }
            }
        }

        // Navigation commands
        if self.matches_any(&text, &words, &["zoom in", "closer", "deeper", "enter"]) {
            return (SemanticEvent::ZoomIn, 0.9);
        }
        if self.matches_any(&text, &words, &["zoom out", "back", "up", "higher", "exit", "away"]) {
            return (SemanticEvent::ZoomOut, 0.9);
        }
        if self.matches_any(&text, &words, &["overview", "global", "home", "show all"]) {
            return (SemanticEvent::OpenGlobalOverview, 0.85);
        }

        // Mode switching
        if self.matches_any(&text, &words, &["command", "terminal", "cli"]) {
            return (SemanticEvent::ModeCommand, 0.9);
        }
        if self.matches_any(&text, &words, &["directory", "files", "folder", "browse"]) {
            return (SemanticEvent::ModeDirectory, 0.9);
        }
        if self.matches_any(&text, &words, &["activity", "processes", "apps", "running"]) {
            return (SemanticEvent::ModeActivity, 0.9);
        }
        if self.matches_any(&text, &words, &["next mode", "cycle", "switch mode"]) {
            return (SemanticEvent::CycleMode, 0.85);
        }

        // System commands
        if self.matches_any(&text, &words, &["reset", "emergency", "abort", "panic", "red alert"]) {
            return (SemanticEvent::TacticalReset, 0.9);
        }
        if self.matches_any(&text, &words, &["bezel", "controls", "menu", "options"]) {
            return (SemanticEvent::ToggleBezel, 0.85);
        }
        if self.matches_any(&text, &words, &["split", "divide", "tile", "split view"]) {
            return (SemanticEvent::SplitViewport, 0.85);
        }
        if self.matches_any(&text, &words, &["close", "exit", "quit", "kill"]) {
            return (SemanticEvent::CloseViewport, 0.85);
        }

        // Selection
        if self.matches_any(&text, &words, &["select", "choose", "open", "activate"]) {
            return (SemanticEvent::Select, 0.8);
        }
        if self.matches_any(&text, &words, &["next", "forward", "right", "down"]) {
            return (SemanticEvent::NextElement, 0.8);
        }
        if self.matches_any(&text, &words, &["previous", "back", "left", "up"]) {
            return (SemanticEvent::PrevElement, 0.8);
        }

        // Mini-map
        if self.matches_any(&text, &words, &["mini-map", "map", "where am i", "location"]) {
            // This would need a custom event, for now use toggle
            return (SemanticEvent::ToggleBezel, 0.7);
        }

        // Unknown command
        (SemanticEvent::SubmitPrompt, 0.3)
    }

    /// Check if any of the target words appear in the input
    fn matches_any(&self, text: &str, words: &[&str], targets: &[&str]) -> bool {
        for target in targets {
            if target.contains(' ') {
                if text.contains(target) {
                    return true;
                }
            } else {
                if words.contains(target) {
                    return true;
                }
            }
        }
        false
    }

    /// Convert action string to semantic event
    fn action_to_event(&self, action: &str) -> Option<SemanticEvent> {
        match action {
            "zoom_in" => Some(SemanticEvent::ZoomIn),
            "zoom_out" => Some(SemanticEvent::ZoomOut),
            "tactical_reset" => Some(SemanticEvent::TacticalReset),
            "toggle_bezel" => Some(SemanticEvent::ToggleBezel),
            "mode_command" => Some(SemanticEvent::ModeCommand),
            "mode_directory" => Some(SemanticEvent::ModeDirectory),
            "mode_activity" => Some(SemanticEvent::ModeActivity),
            "cycle_mode" => Some(SemanticEvent::CycleMode),
            "select" => Some(SemanticEvent::Select),
            "split_viewport" => Some(SemanticEvent::SplitViewport),
            "close_viewport" => Some(SemanticEvent::CloseViewport),
            "open_global_overview" => Some(SemanticEvent::OpenGlobalOverview),
            _ => None,
        }
    }

    /// Add command to history
    fn add_to_history(&mut self, command: VoiceCommand) {
        self.command_history.push(command);
        if self.command_history.len() > self.max_history {
            self.command_history.remove(0);
        }
    }

    /// Get recent command history
    pub fn get_history(&self, count: usize) -> Vec<&VoiceCommand> {
        self.command_history.iter().rev().take(count).collect()
    }

    /// Clear command history
    pub fn clear_history(&mut self) {
        self.command_history.clear();
    }

    /// Execute a voice command, returning the semantic event if successful
    pub fn execute_command(&mut self, command: VoiceCommand) -> Option<SemanticEvent> {
        self.state = VoiceState::Executing;
        
        // Check confidence threshold
        if command.confidence < self.config.auto_execute_threshold 
            && command.confidence_level == ConfidenceLevel::Low {
            // Would prompt for confirmation in real implementation
        }

        self.state = VoiceState::Idle;
        self.audio_buffer.clear();
        
        Some(command.event)
    }

    /// Get current status text for UI
    pub fn get_status_text(&self) -> String {
        match self.state {
            VoiceState::Idle => {
                if self.wake_word_active {
                    format!("Listening for \"{}\"...", self.config.wake_word)
                } else {
                    "Voice commands disabled".to_string()
                }
            }
            VoiceState::Listening { .. } => "Listening for command...".to_string(),
            VoiceState::Processing => "Processing...".to_string(),
            VoiceState::Ready => "Command ready".to_string(),
            VoiceState::Executing => "Executing...".to_string(),
            VoiceState::Error(ref e) => format!("Error: {}", e),
        }
    }

    /// Render voice status indicator as HTML
    pub fn render_indicator(&self) -> String {
        let state_class = match self.state {
            VoiceState::Idle => "voice-idle",
            VoiceState::Listening { .. } => "voice-listening",
            VoiceState::Processing => "voice-processing",
            VoiceState::Ready => "voice-ready",
            VoiceState::Executing => "voice-executing",
            VoiceState::Error(_) => "voice-error",
        };

        let icon = match self.state {
            VoiceState::Idle => "🎤",
            VoiceState::Listening { .. } => "🔴",
            VoiceState::Processing => "⚙",
            VoiceState::Ready => "✓",
            VoiceState::Executing => "▶",
            VoiceState::Error(_) => "⚠",
        };

        let status = self.get_status_text();

        format!(
            r#"<div class="voice-indicator {}">
                <div class="voice-icon">{}</div>
                <div class="voice-status">{}</div>
                {}
            </div>"#,
            state_class,
            icon,
            status,
            if let Some(ref cmd) = self.last_command {
                format!(
                    r#"<div class="voice-last-command">
                        <span class="command-text">"{}"</span>
                        <span class="command-confidence">{}%</span>
                    </div>"#,
                    cmd.raw_text,
                    (cmd.confidence * 100.0) as u32
                )
            } else {
                String::new()
            }
        )
    }

    /// Render voice command help/available commands
    pub fn render_help(&self) -> String {
        r#"<div class="voice-help">
            <h3>Voice Commands</h3>
            <div class="command-category">
                <h4>Navigation</h4>
                <ul>
                    <li><code>"zoom in"</code> or <code>"enter"</code> - Zoom deeper</li>
                    <li><code>"zoom out"</code> or <code>"back"</code> - Zoom out</li>
                    <li><code>"overview"</code> or <code>"home"</code> - Global overview</li>
                </ul>
            </div>
            <div class="command-category">
                <h4>Modes</h4>
                <ul>
                    <li><code>"command mode"</code> - Switch to command mode</li>
                    <li><code>"directory mode"</code> - Switch to directory mode</li>
                    <li><code>"activity mode"</code> - Switch to activity mode</li>
                    <li><code>"cycle mode"</code> - Next mode</li>
                </ul>
            </div>
            <div class="command-category">
                <h4>System</h4>
                <ul>
                    <li><code>"reset"</code> or <code>"emergency"</code> - Tactical reset</li>
                    <li><code>"bezel"</code> or <code>"menu"</code> - Toggle bezel</li>
                    <li><code>"split"</code> - Split viewport</li>
                </ul>
            </div>
            <div class="voice-wake-word">
                Say <strong>"Computer"</strong> to activate voice commands
            </div>
        </div>"#.to_string()
    }
}

/// Errors that can occur in voice processing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceError {
    /// Microphone not available
    MicrophoneUnavailable,
    /// Speech recognition failed
    RecognitionFailed(String),
    /// Network error (for cloud STT)
    NetworkError(String),
    /// Invalid command
    InvalidCommand(String),
    /// Timeout waiting for command
    CommandTimeout,
    /// Permission denied
    PermissionDenied,
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceError::MicrophoneUnavailable => write!(f, "Microphone not available"),
            VoiceError::RecognitionFailed(e) => write!(f, "Recognition failed: {}", e),
            VoiceError::NetworkError(e) => write!(f, "Network error: {}", e),
            VoiceError::InvalidCommand(e) => write!(f, "Invalid command: {}", e),
            VoiceError::CommandTimeout => write!(f, "Command timeout"),
            VoiceError::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}

impl std::error::Error for VoiceError {}

/// Start voice command polling (for integration with main loop)
pub fn start_voice_polling(_processor: std::sync::Arc<std::sync::Mutex<VoiceCommandProcessor>>) {
    std::thread::spawn(move || {
        loop {
            // In real implementation, this would:
            // 1. Check for wake word continuously
            // 2. When detected, record audio
            // 3. Process with STT engine
            // 4. Parse and execute commands
            
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_config_default() {
        let config = VoiceConfig::default();
        assert_eq!(config.wake_word, "computer");
        assert_eq!(config.max_command_duration_secs, 5);
        assert!(config.visual_feedback);
        assert!(config.audio_feedback);
    }

    #[test]
    fn test_confidence_level() {
        assert_eq!(ConfidenceLevel::from_score(0.9), ConfidenceLevel::High);
        assert_eq!(ConfidenceLevel::from_score(0.7), ConfidenceLevel::Medium);
        assert_eq!(ConfidenceLevel::from_score(0.5), ConfidenceLevel::Low);
    }

    #[test]
    fn test_voice_processor_new() {
        let processor = VoiceCommandProcessor::new();
        assert!(matches!(processor.state, VoiceState::Idle));
        assert!(processor.last_command.is_none());
        assert!(processor.command_history.is_empty());
    }

    #[test]
    fn test_parse_navigation_commands() {
        let processor = VoiceCommandProcessor::new();
        
        let (event, confidence) = processor.parse_command("zoom in");
        assert_eq!(event, SemanticEvent::ZoomIn);
        assert!(confidence > 0.8);
        
        let (event, confidence) = processor.parse_command("go back");
        assert_eq!(event, SemanticEvent::ZoomOut);
        
        let (event, _) = processor.parse_command("show overview");
        assert_eq!(event, SemanticEvent::OpenGlobalOverview);
    }

    #[test]
    fn test_parse_mode_commands() {
        let processor = VoiceCommandProcessor::new();
        
        let (event, _) = processor.parse_command("command mode");
        assert_eq!(event, SemanticEvent::ModeCommand);
        
        let (event, _) = processor.parse_command("directory mode");
        assert_eq!(event, SemanticEvent::ModeDirectory);
        
        let (event, _) = processor.parse_command("activity mode");
        assert_eq!(event, SemanticEvent::ModeActivity);
        
        let (event, _) = processor.parse_command("cycle mode");
        assert_eq!(event, SemanticEvent::CycleMode);
    }

    #[test]
    fn test_parse_system_commands() {
        let processor = VoiceCommandProcessor::new();
        
        let (event, _) = processor.parse_command("reset");
        assert_eq!(event, SemanticEvent::TacticalReset);
        
        let (event, _) = processor.parse_command("toggle bezel");
        assert_eq!(event, SemanticEvent::ToggleBezel);
        
        let (event, _) = processor.parse_command("split view");
        assert_eq!(event, SemanticEvent::SplitViewport);
    }

    #[test]
    fn test_custom_commands() {
        let mut processor = VoiceCommandProcessor::new();
        processor.config.custom_commands.insert("test command".to_string(), "zoom_in".to_string());
        
        let cmd = processor.process_text("test command").unwrap();
        assert_eq!(cmd.event, SemanticEvent::ZoomIn);
        assert_eq!(cmd.confidence, 0.95);
    }

    #[test]
    fn test_command_history() {
        let mut processor = VoiceCommandProcessor::new();
        
        processor.process_text("zoom in");
        processor.process_text("zoom out");
        processor.process_text("reset");
        
        assert_eq!(processor.command_history.len(), 3);
        
        let history = processor.get_history(2);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].raw_text, "reset");
        
        processor.clear_history();
        assert!(processor.command_history.is_empty());
    }

    #[test]
    fn test_state_transitions() {
        let mut processor = VoiceCommandProcessor::new();
        
        assert!(matches!(processor.state, VoiceState::Idle));
        
        processor.simulate_wake_word();
        assert!(matches!(processor.state, VoiceState::Listening { .. }));
        assert!(processor.is_listening());
        
        processor.process_text("zoom in");
        assert!(matches!(processor.state, VoiceState::Ready));
        assert!(!processor.is_listening());
    }

    #[test]
    fn test_execute_command() {
        let mut processor = VoiceCommandProcessor::new();
        
        let cmd = processor.process_text("zoom in").unwrap();
        let event = processor.execute_command(cmd);
        
        assert!(event.is_some());
        assert_eq!(event.unwrap(), SemanticEvent::ZoomIn);
        assert!(matches!(processor.state, VoiceState::Idle));
    }

    #[test]
    fn test_status_text() {
        let mut processor = VoiceCommandProcessor::new();
        
        processor.start().unwrap();
        assert!(processor.get_status_text().contains("Listening"));
        
        processor.simulate_wake_word();
        assert!(processor.get_status_text().contains("Listening for command"));
        
        processor.stop();
        assert!(processor.get_status_text().contains("disabled"));
    }

    #[test]
    fn test_render_indicator() {
        let mut processor = VoiceCommandProcessor::new();
        
        let html = processor.render_indicator();
        assert!(html.contains("voice-idle"));
        assert!(html.contains("🎤"));
        
        processor.simulate_wake_word();
        let html = processor.render_indicator();
        assert!(html.contains("voice-listening"));
        assert!(html.contains("🔴"));
    }

    #[test]
    fn test_voice_error_display() {
        assert_eq!(
            VoiceError::MicrophoneUnavailable.to_string(),
            "Microphone not available"
        );
        assert_eq!(
            VoiceError::CommandTimeout.to_string(),
            "Command timeout"
        );
    }
}
