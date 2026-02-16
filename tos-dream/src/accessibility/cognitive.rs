//! Cognitive Accessibility - Simplified Mode and Tutorials
//! 
//! Provides cognitive accessibility features including tutorial systems
//! and context-sensitive help mapping.

use super::{AccessibilityConfig, AccessibilityError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cognitive accessibility manager
#[derive(Debug)]
pub struct CognitiveAccessibility {
    config: Arc<RwLock<AccessibilityConfig>>,
    help_map: HashMap<String, String>,
}

impl CognitiveAccessibility {
    /// Create a new cognitive accessibility manager
    pub async fn new(config: Arc<RwLock<AccessibilityConfig>>) -> Self {
        let mut help_map = HashMap::new();
        
        // Initialize default eval-help mapping
        help_map.insert("zoom_in".to_string(), "Moving deeper into the system hierarchy. From overview to hub, or hub to application.".to_string());
        help_map.insert("zoom_out".to_string(), "Moving higher in the system hierarchy. Return to the level above.".to_string());
        help_map.insert("tactical_reset".to_string(), "Quickly return to a known safe state in the current sector.".to_string());
        help_map.insert("mode_command".to_string(), "Switch to command mode to type and execute system commands.".to_string());
        
        Self {
            config,
            help_map,
        }
    }
    
    /// Get help text for a command/action
    pub async fn get_help(&self, action: &str) -> Option<String> {
        self.help_map.get(action).cloned()
    }
    
    /// Check if tutorial mode is active
    pub async fn is_tutorial_active(&self) -> bool {
        self.config.read().await.tutorial_mode
    }
    
    /// Register a new help mapping
    pub fn register_help(&mut self, action: String, description: String) {
        self.help_map.insert(action, description);
    }
    
    /// Provide tutorial guidance for a transition
    pub async fn get_guidance(&self, from_level: &str, to_level: &str) -> Option<String> {
        if !self.is_tutorial_active().await {
            return None;
        }
        
        match (from_level, to_level) {
            ("GlobalOverview", "CommandHub") => Some("You are now in a Command Hub. You can see applications and a command prompt here.".to_string()),
            ("CommandHub", "ApplicationFocus") => Some("You are now focusing on a single application. The side bezel provides application-specific controls.".to_string()),
            _ => None,
        }
    }
}
