//! Enhanced Tactical Reset System
//! 
//! Provides a two-level emergency recovery system:
//! 
//! ## Level 1 - Sector Reset
//! - Trigger: `Super+Backspace` (configurable) or `tos sector reset`
//! - Action: Sends SIGTERM to all processes in the current sector, closes all viewports,
//!   returns to a fresh Level 2 Command Hub
//! - Confirmation: None by default; optional undo button (5s) can be enabled
//! 
//! ## Level 2 - System Reset
//! - Trigger: `Super+Alt+Backspace` or `tos system reset`
//! - Dialog: Presents three options:
//!   - **Restart Compositor**: Terminates all sectors, restarts TOS compositor, returns to Global Overview
//!   - **Log Out**: Ends TOS session, returns to login manager
//!   - **Cancel**
//! - Confirmation: Tactile confirmation required (hold, slider, voice, multi-button)
//! - Countdown with cancel option

use crate::{TosState, HierarchyLevel, Sector, CommandHub};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Type of reset operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResetLevel {
    /// Level 1: Reset current sector only
    Sector,
    /// Level 2: System-wide reset (compositor or logout)
    System,
}

/// System reset option
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemResetOption {
    /// Restart the TOS compositor
    RestartCompositor,
    /// Log out and return to login manager
    LogOut,
    /// Cancel the reset operation
    Cancel,
}

/// Tactile confirmation method
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TactileMethod {
    /// Hold a button/key for specified duration
    Hold { duration_ms: u64 },
    /// Slide a control across the screen
    Slider,
    /// Voice confirmation phrase
    Voice(String),
    /// Press multiple buttons simultaneously
    MultiButton { buttons: usize },
}

/// Configuration for reset behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetConfig {
    /// Enable undo button for sector reset
    pub enable_sector_undo: bool,
    /// Undo button duration in seconds
    pub undo_duration_secs: u64,
    /// Default tactile confirmation method for system reset
    pub default_tactile_method: TactileMethod,
    /// Countdown duration before reset executes
    pub countdown_secs: u64,
    /// Whether to show confirmation dialog for sector reset
    pub confirm_sector_reset: bool,
    /// Whether to save sector state before reset
    pub save_state_before_reset: bool,
}

impl Default for ResetConfig {
    fn default() -> Self {
        Self {
            enable_sector_undo: true,
            undo_duration_secs: 5,
            default_tactile_method: TactileMethod::Hold { duration_ms: 2000 },
            countdown_secs: 3,
            confirm_sector_reset: false,
            save_state_before_reset: true,
        }
    }
}

/// State of a reset operation
#[derive(Debug, Clone, PartialEq)]
pub enum ResetOperationState {
    /// No reset in progress
    Idle,
    /// Sector reset in progress with undo available
    SectorResetting { sector_id: Uuid, start_time: Instant },
    /// System reset dialog shown, waiting for option selection
    SystemDialogShown,
    /// Tactile confirmation in progress
    TactileConfirming {
        option: SystemResetOption,
        method: TactileMethod,
        progress: f32, // 0.0 to 1.0
    },
    /// Countdown before executing
    Countdown {
        option: SystemResetOption,
        remaining_secs: u64,
    },
    /// Executing the reset
    Executing,
}

/// The Tactical Reset Manager
pub struct TacticalReset {
    /// Configuration
    pub config: ResetConfig,
    /// Current operation state
    pub state: ResetOperationState,
    /// Saved sector state for potential restoration
    pub saved_sector: Option<Sector>,
    /// Callback for executing system commands
    system_executor: Option<Box<dyn Fn(&str) -> Result<(), String> + Send>>,
}

impl std::fmt::Debug for TacticalReset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TacticalReset")
            .field("config", &self.config)
            .field("state", &self.state)
            .field("saved_sector", &self.saved_sector)
            .field("system_executor", &self.system_executor.as_ref().map(|_| "Some(Fn)"))
            .finish()
    }
}

impl Default for TacticalReset {
    fn default() -> Self {
        Self::new()
    }
}

impl TacticalReset {
    /// Create a new tactical reset manager with default config
    pub fn new() -> Self {
        Self::with_config(ResetConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: ResetConfig) -> Self {
        Self {
            config,
            state: ResetOperationState::Idle,
            saved_sector: None,
            system_executor: None,
        }
    }

    /// Set a custom system command executor
    pub fn set_system_executor<F>(&mut self, executor: F)
    where
        F: Fn(&str) -> Result<(), String> + Send + 'static,
    {
        self.system_executor = Some(Box::new(executor));
    }

    /// Check if a reset is currently in progress
    pub fn is_resetting(&self) -> bool {
        !matches!(self.state, ResetOperationState::Idle)
    }

    /// Initiate a Level 1 Sector Reset
    pub fn initiate_sector_reset(&mut self, state: &mut TosState) -> Result<(), ResetError> {
        if self.is_resetting() {
            return Err(ResetError::ResetInProgress);
        }

        let viewport = &state.viewports[state.active_viewport_index];
        let sector_id = state.sectors[viewport.sector_index].id;

        // Save sector state if enabled
        if self.config.save_state_before_reset {
            self.saved_sector = Some(state.sectors[viewport.sector_index].clone());
        }

        // Start the reset operation
        self.state = ResetOperationState::SectorResetting {
            sector_id,
            start_time: Instant::now(),
        };

        // Trigger auditory indicator (§11)
        state.earcon_player.play(crate::system::audio::earcons::EarconEvent::TacticalAlert);

        // Execute the actual reset
        self.execute_sector_reset(state, viewport.sector_index)?;

        Ok(())
    }

    /// Execute the sector reset logic
    fn execute_sector_reset(&mut self, state: &mut TosState, sector_index: usize) -> Result<(), ResetError> {
        // Get the sector
        let sector = &mut state.sectors[sector_index];
        
        // Send SIGTERM to all application processes in the sector
        for hub in &mut sector.hubs {
            for app in &hub.applications {
                if let Some(pid) = app.pid {
                    unsafe {
                        libc::kill(pid as i32, libc::SIGTERM);
                    }
                }
            }
            hub.applications.clear();
            hub.active_app_index = None;
            hub.prompt.clear();
            hub.confirmation_required = None;
        }

        // Reset to single hub
        sector.hubs = vec![CommandHub {
            id: Uuid::new_v4(),
            mode: crate::CommandHubMode::Command,
            prompt: String::new(),
            applications: vec![],
            active_app_index: None,
            terminal_output: vec![],
            confirmation_required: None,
            current_directory: dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/")),
            show_hidden_files: false,
            selected_files: std::collections::HashSet::new(),
            context_menu: None,
            shell_listing: None,
        }];
        sector.active_hub_index = 0;

        // Reset viewports for this sector
        for viewport in &mut state.viewports {
            if viewport.sector_index == sector_index {
                viewport.hub_index = 0;
                viewport.current_level = HierarchyLevel::CommandHub;
                viewport.active_app_index = None;
                viewport.bezel_expanded = false;
            }
        }

        // Update global state
        state.current_level = HierarchyLevel::CommandHub;
        state.escape_count = 0;

        // If undo is not enabled, complete immediately
        if !self.config.enable_sector_undo {
            self.complete_reset();
        }

        Ok(())
    }

    /// Undo a sector reset (if within time window)
    pub fn undo_sector_reset(&mut self, state: &mut TosState) -> Result<(), ResetError> {
        match self.state {
            ResetOperationState::SectorResetting { start_time, .. } => {
                let elapsed = start_time.elapsed();
                if elapsed > Duration::from_secs(self.config.undo_duration_secs) {
                    return Err(ResetError::UndoExpired);
                }

                // Restore the saved sector
                if let Some(saved) = self.saved_sector.take() {
                    let viewport = &state.viewports[state.active_viewport_index];
                    let sector_index = viewport.sector_index;
                    state.sectors[sector_index] = saved;
                    
                    self.complete_reset();
                    Ok(())
                } else {
                    Err(ResetError::NoSavedState)
                }
            }
            _ => Err(ResetError::NoResetInProgress),
        }
    }

    /// Initiate a Level 2 System Reset
    pub fn initiate_system_reset(&mut self) -> Result<(), ResetError> {
        if self.is_resetting() {
            return Err(ResetError::ResetInProgress);
        }

        self.state = ResetOperationState::SystemDialogShown;
        Ok(())
    }

    /// Select a system reset option
    pub fn select_system_option(&mut self, option: SystemResetOption) -> Result<(), ResetError> {
        match self.state {
            ResetOperationState::SystemDialogShown => {
                match option {
                    SystemResetOption::Cancel => {
                        self.cancel_reset();
                        Ok(())
                    }
                    _ => {
                        // Start tactile confirmation
                        self.state = ResetOperationState::TactileConfirming {
                            option,
                            method: self.config.default_tactile_method.clone(),
                            progress: 0.0,
                        };
                        Ok(())
                    }
                }
            }
            _ => Err(ResetError::InvalidState),
        }
    }

    /// Update tactile confirmation progress
    pub fn update_tactile_progress(&mut self, progress: f32) -> Result<Option<SystemResetOption>, ResetError> {
        match &mut self.state {
            ResetOperationState::TactileConfirming { option, method, progress: ref mut p } => {
                *p = progress.clamp(0.0, 1.0);
                
                let required = match method {
                    TactileMethod::Hold { .. } => 1.0,
                    TactileMethod::Slider => 1.0,
                    TactileMethod::Voice(_) => 1.0,
                    TactileMethod::MultiButton { buttons } => *buttons as f32,
                };

                if *p >= required {
                    let opt = *option;
                    // Start countdown
                    self.state = ResetOperationState::Countdown {
                        option: opt,
                        remaining_secs: self.config.countdown_secs,
                    };
                    Ok(Some(opt))
                } else {
                    Ok(None)
                }
            }
            _ => Err(ResetError::InvalidState),
        }
    }

    /// Cancel the current reset operation
    pub fn cancel_reset(&mut self) {
        self.state = ResetOperationState::Idle;
        self.saved_sector = None;
    }

    /// Complete the reset operation
    fn complete_reset(&mut self) {
        self.state = ResetOperationState::Idle;
        self.saved_sector = None;
    }

    /// Tick the countdown timer (call periodically)
    pub fn tick_countdown(&mut self) -> Option<SystemResetOption> {
        match &mut self.state {
            ResetOperationState::Countdown { option, remaining_secs } => {
                if *remaining_secs == 0 {
                    let opt = *option;
                    self.state = ResetOperationState::Executing;
                    Some(opt)
                } else {
                    *remaining_secs -= 1;
                    None
                }
            }
            _ => None,
        }
    }

    /// Execute the selected system reset option
    pub fn execute_system_reset(&mut self, option: SystemResetOption) -> Result<(), ResetError> {
        match option {
            SystemResetOption::RestartCompositor => {
                self.restart_compositor()
            }
            SystemResetOption::LogOut => {
                self.log_out()
            }
            SystemResetOption::Cancel => {
                self.cancel_reset();
                Ok(())
            }
        }
    }

    /// Restart the TOS compositor
    fn restart_compositor(&self) -> Result<(), ResetError> {
        if let Some(ref executor) = self.system_executor {
            executor("systemctl restart tos-compositor")
                .map_err(|e| ResetError::ExecutionFailed(e))
        } else {
            // Default implementation - in real system would restart
            println!("TACTICAL RESET: Restarting TOS compositor...");
            Ok(())
        }
    }

    /// Log out and return to login manager
    fn log_out(&self) -> Result<(), ResetError> {
        if let Some(ref executor) = self.system_executor {
            executor("pkill -u $USER tos")
                .map_err(|e| ResetError::ExecutionFailed(e))
        } else {
            // Default implementation
            println!("TACTICAL RESET: Logging out...");
            Ok(())
        }
    }

    /// Check if undo is still available for sector reset
    pub fn can_undo(&self) -> bool {
        match self.state {
            ResetOperationState::SectorResetting { start_time, .. } => {
                start_time.elapsed() < Duration::from_secs(self.config.undo_duration_secs)
            }
            _ => false,
        }
    }

    /// Get remaining undo time in seconds
    pub fn undo_remaining_secs(&self) -> Option<u64> {
        match self.state {
            ResetOperationState::SectorResetting { start_time, .. } => {
                let elapsed = start_time.elapsed().as_secs();
                if elapsed < self.config.undo_duration_secs {
                    Some(self.config.undo_duration_secs - elapsed)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get remaining countdown seconds
    pub fn countdown_remaining(&self) -> Option<u64> {
        match self.state {
            ResetOperationState::Countdown { remaining_secs, .. } => Some(remaining_secs),
            _ => None,
        }
    }

    /// Render the reset dialog/status as HTML
    pub fn render(&self) -> String {
        match &self.state {
            ResetOperationState::Idle => String::new(),
            
            ResetOperationState::SectorResetting { sector_id, .. } => {
                let undo_button = if self.config.enable_sector_undo && self.can_undo() {
                    let remaining = self.undo_remaining_secs().unwrap_or(0);
                    format!(
                        r#"<button class="reset-undo" onclick="undoSectorReset()">
                            UNDO RESET ({}s)
                        </button>"#,
                        remaining
                    )
                } else {
                    String::new()
                };

                format!(
                    r#"<div class="tactical-reset-overlay sector-reset">
                        <div class="reset-message">SECTOR RESET COMPLETE</div>
                        <div class="reset-details">Sector {} has been reset to default state</div>
                        {}
                    </div>"#,
                    &sector_id.to_string()[..8],
                    undo_button
                )
            }
            
            ResetOperationState::SystemDialogShown => {
                r#"<div class="tactical-reset-overlay system-dialog">
                    <div class="reset-title">SYSTEM RESET</div>
                    <div class="reset-options">
                        <button class="reset-option" onclick="selectResetOption('restart')">
                            <div class="option-icon">↻</div>
                            <div class="option-label">Restart Compositor</div>
                            <div class="option-desc">Restart TOS, keep session</div>
                        </button>
                        <button class="reset-option" onclick="selectResetOption('logout')">
                            <div class="option-icon">⏻</div>
                            <div class="option-label">Log Out</div>
                            <div class="option-desc">End session, return to login</div>
                        </button>
                        <button class="reset-option cancel" onclick="selectResetOption('cancel')">
                            <div class="option-icon">✕</div>
                            <div class="option-label">Cancel</div>
                            <div class="option-desc">Return to normal operation</div>
                        </button>
                    </div>
                </div>"#.to_string()
            }
            
            ResetOperationState::TactileConfirming { option, method, progress } => {
                let (method_name, method_desc) = match method {
                    TactileMethod::Hold { duration_ms } => (
                        "HOLD TO CONFIRM",
                        format!("Hold for {} seconds", duration_ms / 1000)
                    ),
                    TactileMethod::Slider => (
                        "SLIDE TO CONFIRM",
                        "Drag slider to the right".to_string()
                    ),
                    TactileMethod::Voice(ref phrase) => (
                        "VOICE CONFIRMATION",
                        format!("Say: \"{}\"", phrase)
                    ),
                    TactileMethod::MultiButton { buttons } => (
                        "MULTI-BUTTON CONFIRM",
                        format!("Press {} buttons simultaneously", buttons)
                    ),
                };

                let option_name = match option {
                    SystemResetOption::RestartCompositor => "Restart Compositor",
                    SystemResetOption::LogOut => "Log Out",
                    SystemResetOption::Cancel => "Cancel",
                };

                format!(
                    r#"<div class="tactical-reset-overlay tactile-confirm">
                        <div class="reset-title">CONFIRM {}</div>
                        <div class="tactile-method">{}</div>
                        <div class="tactile-desc">{}</div>
                        <div class="tactile-progress">
                            <div class="progress-bar" style="width: {}%"></div>
                        </div>
                        <button class="reset-cancel" onclick="cancelReset()">CANCEL</button>
                    </div>"#,
                    option_name,
                    method_name,
                    method_desc,
                    (progress * 100.0) as u32
                )
            }
            
            ResetOperationState::Countdown { option, remaining_secs } => {
                let option_name = match option {
                    SystemResetOption::RestartCompositor => "Restarting Compositor",
                    SystemResetOption::LogOut => "Logging Out",
                    SystemResetOption::Cancel => "Cancelling",
                };

                format!(
                    r#"<div class="tactical-reset-overlay countdown">
                        <div class="reset-title">{} IN...</div>
                        <div class="countdown-number">{}</div>
                        <button class="reset-cancel" onclick="cancelReset()">CANCEL</button>
                    </div>"#,
                    option_name,
                    remaining_secs
                )
            }
            
            ResetOperationState::Executing => {
                r#"<div class="tactical-reset-overlay executing">
                    <div class="reset-spinner">◐</div>
                    <div class="reset-message">EXECUTING RESET...</div>
                </div>"#.to_string()
            }
        }
    }
}

/// Errors that can occur during reset operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResetError {
    /// A reset is already in progress
    ResetInProgress,
    /// No reset in progress to undo
    NoResetInProgress,
    /// Undo time has expired
    UndoExpired,
    /// No saved state to restore
    NoSavedState,
    /// Invalid operation for current state
    InvalidState,
    /// System command execution failed
    ExecutionFailed(String),
}

impl std::fmt::Display for ResetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResetError::ResetInProgress => write!(f, "A reset is already in progress"),
            ResetError::NoResetInProgress => write!(f, "No reset in progress"),
            ResetError::UndoExpired => write!(f, "Undo time has expired"),
            ResetError::NoSavedState => write!(f, "No saved state available"),
            ResetError::InvalidState => write!(f, "Invalid operation for current state"),
            ResetError::ExecutionFailed(e) => write!(f, "Execution failed: {}", e),
        }
    }
}

impl std::error::Error for ResetError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_config_default() {
        let config = ResetConfig::default();
        assert!(config.enable_sector_undo);
        assert_eq!(config.undo_duration_secs, 5);
        assert_eq!(config.countdown_secs, 3);
        assert!(config.save_state_before_reset);
    }

    #[test]
    fn test_tactical_reset_new() {
        let reset = TacticalReset::new();
        assert!(!reset.is_resetting());
        assert!(matches!(reset.state, ResetOperationState::Idle));
    }

    #[test]
    fn test_sector_reset_lifecycle() {
        let mut reset = TacticalReset::new();
        let mut state = crate::TosState::new();
        
        // Initially not resetting
        assert!(!reset.is_resetting());
        
        // Initiate sector reset
        reset.initiate_sector_reset(&mut state).unwrap();
        assert!(reset.is_resetting());
        assert!(matches!(reset.state, ResetOperationState::SectorResetting { .. }));
        
        // Can undo initially
        assert!(reset.can_undo());
        assert!(reset.undo_remaining_secs().is_some());
        
        // Complete reset
        reset.complete_reset();
        assert!(!reset.is_resetting());
    }

    #[test]
    fn test_system_reset_dialog() {
        let mut reset = TacticalReset::new();
        
        // Show dialog
        reset.initiate_system_reset().unwrap();
        assert!(matches!(reset.state, ResetOperationState::SystemDialogShown));
        
        // Select restart option
        reset.select_system_option(SystemResetOption::RestartCompositor).unwrap();
        assert!(matches!(reset.state, ResetOperationState::TactileConfirming { .. }));
        
        // Cancel
        reset.cancel_reset();
        assert!(matches!(reset.state, ResetOperationState::Idle));
    }

    #[test]
    fn test_tactile_progress() {
        let mut reset = TacticalReset::new();
        
        reset.initiate_system_reset().unwrap();
        reset.select_system_option(SystemResetOption::RestartCompositor).unwrap();
        
        // Update progress
        let result = reset.update_tactile_progress(0.5).unwrap();
        assert!(result.is_none()); // Not complete yet
        
        // Complete progress
        let result = reset.update_tactile_progress(1.0).unwrap();
        assert!(result.is_some());
        assert!(matches!(reset.state, ResetOperationState::Countdown { .. }));
    }

    #[test]
    fn test_countdown_tick() {
        let mut reset = TacticalReset::new();
        reset.config.countdown_secs = 3;
        
        reset.initiate_system_reset().unwrap();
        reset.select_system_option(SystemResetOption::RestartCompositor).unwrap();
        reset.update_tactile_progress(1.0).unwrap();
        
        // Tick countdown
        assert_eq!(reset.countdown_remaining(), Some(3));
        assert!(reset.tick_countdown().is_none());
        
        assert_eq!(reset.countdown_remaining(), Some(2));
        assert!(reset.tick_countdown().is_none());
        
        assert_eq!(reset.countdown_remaining(), Some(1));
        assert!(reset.tick_countdown().is_none());
        
        assert_eq!(reset.countdown_remaining(), Some(0));
        assert!(reset.tick_countdown().is_some()); // Executes at 0
    }

    #[test]
    fn test_reset_error_display() {
        assert_eq!(
            ResetError::ResetInProgress.to_string(),
            "A reset is already in progress"
        );
        assert_eq!(
            ResetError::UndoExpired.to_string(),
            "Undo time has expired"
        );
    }

    #[test]
    fn test_render_states() {
        let mut reset = TacticalReset::new();
        
        // Idle renders empty
        assert!(reset.render().is_empty());
        
        // System dialog
        reset.initiate_system_reset().unwrap();
        let html = reset.render();
        assert!(html.contains("SYSTEM RESET"));
        assert!(html.contains("Restart Compositor"));
        assert!(html.contains("Log Out"));
        
        // Tactile confirmation
        reset.select_system_option(SystemResetOption::LogOut).unwrap();
        let html = reset.render();
        assert!(html.contains("CONFIRM"));
        assert!(html.contains("progress-bar"));
        
        // Countdown
        reset.update_tactile_progress(1.0).unwrap();
        let html = reset.render();
        assert!(html.contains("countdown-number"));
    }
}
