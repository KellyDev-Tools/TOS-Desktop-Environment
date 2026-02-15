//! Tests for Voice Command System (Phase 11)

use tos_core::system::voice::{
    VoiceCommandProcessor, VoiceConfig, VoiceState, ConfidenceLevel, 
    VoiceCommand, VoiceError
};
use tos_core::system::input::SemanticEvent;

#[test]
fn test_voice_config_default() {
    let config = VoiceConfig::default();
    
    assert_eq!(config.wake_word, "computer");
    assert_eq!(config.max_command_duration_secs, 5);
    assert!(config.visual_feedback);
    assert!(config.audio_feedback);
    assert_eq!(config.language, "en-US");
    assert!(config.microphone_index.is_none());
    assert_eq!(config.auto_execute_threshold, 0.7);
    
    // Check custom commands
    assert!(config.custom_commands.contains_key("beam me up"));
    assert!(config.custom_commands.contains_key("engage"));
    assert!(config.custom_commands.contains_key("red alert"));
}

#[test]
fn test_voice_processor_creation() {
    let processor = VoiceCommandProcessor::new();
    
    assert!(matches!(processor.state, VoiceState::Idle));
    assert!(processor.last_command.is_none());
    assert!(processor.command_history.is_empty());
    assert!(!processor.wake_word_active);
}

#[test]
fn test_confidence_level_from_score() {
    assert_eq!(ConfidenceLevel::from_score(0.9), ConfidenceLevel::High);
    assert_eq!(ConfidenceLevel::from_score(0.85), ConfidenceLevel::High);
    assert_eq!(ConfidenceLevel::from_score(0.7), ConfidenceLevel::Medium);
    assert_eq!(ConfidenceLevel::from_score(0.6), ConfidenceLevel::Medium);
    assert_eq!(ConfidenceLevel::from_score(0.5), ConfidenceLevel::Low);
    assert_eq!(ConfidenceLevel::from_score(0.3), ConfidenceLevel::Low);
}

#[test]
fn test_voice_state_transitions() {
    let mut processor = VoiceCommandProcessor::new();
    
    // Start
    processor.start().unwrap();
    assert!(processor.wake_word_active);
    assert!(matches!(processor.state, VoiceState::Idle));
    
    // Simulate wake word
    processor.simulate_wake_word();
    assert!(matches!(processor.state, VoiceState::Listening { .. }));
    assert!(processor.is_listening());
    
    // Process text (simulates end of listening)
    processor.process_text("zoom in");
    assert!(matches!(processor.state, VoiceState::Ready));
    assert!(!processor.is_listening());
    
    // Stop
    processor.stop();
    assert!(matches!(processor.state, VoiceState::Idle));
    assert!(!processor.wake_word_active);
}

#[test]
fn test_parse_navigation_commands() {
    let processor = VoiceCommandProcessor::new();
    
    // Zoom in variations
    let (event, confidence) = processor.parse_command("zoom in");
    assert_eq!(event, SemanticEvent::ZoomIn);
    assert!(confidence > 0.8);
    
    let (event, _) = processor.parse_command("enter");
    assert_eq!(event, SemanticEvent::ZoomIn);
    
    let (event, _) = processor.parse_command("go closer");
    assert_eq!(event, SemanticEvent::ZoomIn);
    
    // Zoom out variations
    let (event, confidence) = processor.parse_command("zoom out");
    assert_eq!(event, SemanticEvent::ZoomOut);
    assert!(confidence > 0.8);
    
    let (event, _) = processor.parse_command("go back");
    assert_eq!(event, SemanticEvent::ZoomOut);
    
    let (event, _) = processor.parse_command("exit");
    assert_eq!(event, SemanticEvent::ZoomOut);
    
    // Overview
    let (event, _) = processor.parse_command("show overview");
    assert_eq!(event, SemanticEvent::OpenGlobalOverview);
    
    let (event, _) = processor.parse_command("home");
    assert_eq!(event, SemanticEvent::OpenGlobalOverview);
}

#[test]
fn test_parse_mode_commands() {
    let processor = VoiceCommandProcessor::new();
    
    // Command mode
    let (event, _) = processor.parse_command("command mode");
    assert_eq!(event, SemanticEvent::ModeCommand);
    
    let (event, _) = processor.parse_command("terminal");
    assert_eq!(event, SemanticEvent::ModeCommand);
    
    // Directory mode
    let (event, _) = processor.parse_command("directory mode");
    assert_eq!(event, SemanticEvent::ModeDirectory);
    
    let (event, _) = processor.parse_command("files");
    assert_eq!(event, SemanticEvent::ModeDirectory);
    
    // Activity mode
    let (event, _) = processor.parse_command("activity mode");
    assert_eq!(event, SemanticEvent::ModeActivity);
    
    let (event, _) = processor.parse_command("running apps");
    assert_eq!(event, SemanticEvent::ModeActivity);
    
    // Cycle mode
    let (event, _) = processor.parse_command("cycle mode");
    assert_eq!(event, SemanticEvent::CycleMode);
    
    let (event, _) = processor.parse_command("switch mode");
    assert_eq!(event, SemanticEvent::CycleMode);
}

#[test]
fn test_parse_system_commands() {
    let processor = VoiceCommandProcessor::new();
    
    // Reset
    let (event, _) = processor.parse_command("reset");
    assert_eq!(event, SemanticEvent::TacticalReset);
    
    let (event, _) = processor.parse_command("emergency");
    assert_eq!(event, SemanticEvent::TacticalReset);
    
    // Bezel
    let (event, _) = processor.parse_command("toggle bezel");
    assert_eq!(event, SemanticEvent::ToggleBezel);
    
    let (event, _) = processor.parse_command("menu");
    assert_eq!(event, SemanticEvent::ToggleBezel);
    
    // Split
    let (event, _) = processor.parse_command("split view");
    assert_eq!(event, SemanticEvent::SplitViewport);
    
    // Close
    let (event, _) = processor.parse_command("close");
    assert_eq!(event, SemanticEvent::CloseViewport);
}

#[test]
fn test_parse_selection_commands() {
    let processor = VoiceCommandProcessor::new();
    
    // Select
    let (event, _) = processor.parse_command("select");
    assert_eq!(event, SemanticEvent::Select);
    
    let (event, _) = processor.parse_command("choose");
    assert_eq!(event, SemanticEvent::Select);
    
    // Next
    let (event, _) = processor.parse_command("next");
    assert_eq!(event, SemanticEvent::NextElement);
    
    let (event, _) = processor.parse_command("forward");
    assert_eq!(event, SemanticEvent::NextElement);
    
    // Previous
    let (event, _) = processor.parse_command("previous");
    assert_eq!(event, SemanticEvent::PrevElement);
}

#[test]
fn test_custom_commands() {
    let mut processor = VoiceCommandProcessor::new();
    
    // Add custom command
    processor.config.custom_commands.insert(
        "test zoom".to_string(), 
        "zoom_in".to_string()
    );
    
    let cmd = processor.process_text("test zoom").unwrap();
    assert_eq!(cmd.event, SemanticEvent::ZoomIn);
    assert_eq!(cmd.confidence, 0.95); // Custom commands have high confidence
    assert_eq!(cmd.confidence_level, ConfidenceLevel::High);
}

#[test]
fn test_builtin_custom_commands() {
    let processor = VoiceCommandProcessor::new();
    
    // "beam me up" -> zoom_in
    let (event, _) = processor.parse_command("beam me up");
    assert_eq!(event, SemanticEvent::ZoomIn);
    
    // "engage" -> execute (submit prompt)
    let (event, _) = processor.parse_command("engage");
    assert_eq!(event, SemanticEvent::SubmitPrompt);
    
    // "red alert" -> tactical_reset
    let (event, _) = processor.parse_command("red alert");
    assert_eq!(event, SemanticEvent::TacticalReset);
}

#[test]
fn test_command_processing() {
    let mut processor = VoiceCommandProcessor::new();
    
    let cmd = processor.process_text("zoom in").unwrap();
    
    assert_eq!(cmd.raw_text, "zoom in");
    assert_eq!(cmd.event, SemanticEvent::ZoomIn);
    assert!(cmd.confidence > 0.8);
    assert_eq!(cmd.confidence_level, ConfidenceLevel::High);
    assert!(cmd.processing_duration.as_millis() < 100); // Should be fast
}

#[test]
fn test_command_history() {
    let mut processor = VoiceCommandProcessor::new();
    
    // Process multiple commands
    processor.process_text("zoom in");
    processor.process_text("zoom out");
    processor.process_text("reset");
    processor.process_text("command mode");
    
    assert_eq!(processor.command_history.len(), 4);
    
    // Get recent history
    let recent = processor.get_history(2);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].raw_text, "command mode"); // Most recent first
    assert_eq!(recent[1].raw_text, "reset");
    
    // Clear history
    processor.clear_history();
    assert!(processor.command_history.is_empty());
}

#[test]
fn test_history_limit() {
    let mut processor = VoiceCommandProcessor::new();
    
    // Add more than max_history commands
    for i in 0..60 {
        processor.process_text(&format!("command {}", i));
    }
    
    // Should be limited to 50
    assert_eq!(processor.command_history.len(), 50);
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
fn test_low_confidence_command() {
    let mut processor = VoiceCommandProcessor::new();
    processor.config.auto_execute_threshold = 0.9; // High threshold
    
    // Process unclear command
    let cmd = processor.process_text("something unclear").unwrap();
    
    // Should have low confidence
    assert!(cmd.confidence < 0.9);
    assert_eq!(cmd.confidence_level, ConfidenceLevel::Low);
}

#[test]
fn test_status_text() {
    let mut processor = VoiceCommandProcessor::new();
    
    // Idle with wake word active
    processor.start().unwrap();
    let status = processor.get_status_text();
    assert!(status.contains("Listening"));
    assert!(status.contains("computer"));
    
    // Listening for command
    processor.simulate_wake_word();
    let status = processor.get_status_text();
    assert!(status.contains("Listening for command"));
    
    // Processing
    processor.state = VoiceState::Processing;
    let status = processor.get_status_text();
    assert!(status.contains("Processing"));
    
    // Ready
    processor.state = VoiceState::Ready;
    let status = processor.get_status_text();
    assert!(status.contains("Command ready"));
    
    // Executing
    processor.state = VoiceState::Executing;
    let status = processor.get_status_text();
    assert!(status.contains("Executing"));
    
    // Error
    processor.state = VoiceState::Error(VoiceError::MicrophoneUnavailable);
    let status = processor.get_status_text();
    assert!(status.contains("Error"));
    
    // Stopped
    processor.stop();
    let status = processor.get_status_text();
    assert!(status.contains("disabled"));
}

#[test]
fn test_render_indicator() {
    let mut processor = VoiceCommandProcessor::new();
    
    // Idle
    processor.start().unwrap();
    let html = processor.render_indicator();
    assert!(html.contains("voice-idle"));
    assert!(html.contains("🎤"));
    
    // Listening
    processor.simulate_wake_word();
    let html = processor.render_indicator();
    assert!(html.contains("voice-listening"));
    assert!(html.contains("🔴"));
    
    // Processing
    processor.state = VoiceState::Processing;
    let html = processor.render_indicator();
    assert!(html.contains("voice-processing"));
    assert!(html.contains("⚙"));
    
    // Ready
    processor.state = VoiceState::Ready;
    let html = processor.render_indicator();
    assert!(html.contains("voice-ready"));
    assert!(html.contains("✓"));
    
    // Executing
    processor.state = VoiceState::Executing;
    let html = processor.render_indicator();
    assert!(html.contains("voice-executing"));
    assert!(html.contains("▶"));
    
    // Error
    processor.state = VoiceState::Error(VoiceError::MicrophoneUnavailable);
    let html = processor.render_indicator();
    assert!(html.contains("voice-error"));
    assert!(html.contains("⚠"));
}

#[test]
fn test_render_indicator_with_last_command() {
    let mut processor = VoiceCommandProcessor::new();
    
    processor.process_text("zoom in");
    
    let html = processor.render_indicator();
    assert!(html.contains("zoom in"));
    assert!(html.contains("voice-last-command"));
    assert!(html.contains("%")); // Confidence percentage
}

#[test]
fn test_render_help() {
    let processor = VoiceCommandProcessor::new();
    
    let html = processor.render_help();
    assert!(html.contains("Voice Commands"));
    assert!(html.contains("Navigation"));
    assert!(html.contains("Modes"));
    assert!(html.contains("System"));
    assert!(html.contains("zoom in"));
    assert!(html.contains("command mode"));
    assert!(html.contains("reset"));
    assert!(html.contains("Computer")); // Wake word
}

#[test]
fn test_voice_error_display() {
    assert_eq!(
        VoiceError::MicrophoneUnavailable.to_string(),
        "Microphone not available"
    );
    assert_eq!(
        VoiceError::RecognitionFailed("test error".to_string()).to_string(),
        "Recognition failed: test error"
    );
    assert_eq!(
        VoiceError::NetworkError("timeout".to_string()).to_string(),
        "Network error: timeout"
    );
    assert_eq!(
        VoiceError::InvalidCommand("unknown".to_string()).to_string(),
        "Invalid command: unknown"
    );
    assert_eq!(
        VoiceError::CommandTimeout.to_string(),
        "Command timeout"
    );
    assert_eq!(
        VoiceError::PermissionDenied.to_string(),
        "Permission denied"
    );
}

#[test]
fn test_voice_config_customization() {
    let mut config = VoiceConfig::default();
    
    config.wake_word = "hey tos".to_string();
    config.max_command_duration_secs = 10;
    config.auto_execute_threshold = 0.5;
    config.visual_feedback = false;
    config.audio_feedback = false;
    config.language = "en-GB".to_string();
    config.microphone_index = Some(2);
    
    let processor = VoiceCommandProcessor::with_config(config);
    
    assert_eq!(processor.config.wake_word, "hey tos");
    assert_eq!(processor.config.max_command_duration_secs, 10);
    assert_eq!(processor.config.auto_execute_threshold, 0.5);
    assert!(!processor.config.visual_feedback);
    assert!(!processor.config.audio_feedback);
    assert_eq!(processor.config.language, "en-GB");
    assert_eq!(processor.config.microphone_index, Some(2));
}

#[test]
fn test_unknown_command() {
    let processor = VoiceCommandProcessor::new();
    
    // Unknown command should default to SubmitPrompt with low confidence
    let (event, confidence) = processor.parse_command("xyz abc 123");
    assert_eq!(event, SemanticEvent::SubmitPrompt);
    assert!(confidence < 0.5);
}

#[test]
fn test_command_with_extra_words() {
    let processor = VoiceCommandProcessor::new();
    
    // Should still recognize command with extra context
    let (event, _) = processor.parse_command("please zoom in now");
    assert_eq!(event, SemanticEvent::ZoomIn);
    
    let (event, _) = processor.parse_command("can you show the overview");
    assert_eq!(event, SemanticEvent::OpenGlobalOverview);
}
