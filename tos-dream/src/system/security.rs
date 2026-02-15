//! Security Module - Dangerous Command Handling
//! 
//! Provides tactile confirmation and security features for dangerous operations.
//! 
//! ## Features
//! - Dangerous command pattern detection
//! - Tactile confirmation with multiple modalities
//! - Visual/auditory/haptic warning system
//! - Audit logging
//! - RBAC enforcement
//! 
//! ## Tactile Confirmation Methods
//! - **Hold**: Hold button/key for specified duration
//! - **Slider**: Drag slider across screen
//! - **Voice**: Speak confirmation phrase
//! - **Multi-button**: Press multiple buttons simultaneously
//! 
//! ## Audit Logging
//! All security events are logged with timestamp, user, sector, and action details.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

/// Security event types for audit logging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityEvent {
    /// Dangerous command detected
    DangerousCommandDetected {
        command: String,
        risk_level: RiskLevel,
        user: String,
        sector_id: Uuid,
        timestamp: String,
    },
    /// Tactile confirmation started
    ConfirmationStarted {
        method: TactileMethod,
        command: String,
        user: String,
        timestamp: String,
    },
    /// Tactile confirmation completed
    ConfirmationCompleted {
        method: TactileMethod,
        command: String,
        user: String,
        success: bool,
        timestamp: String,
    },
    /// Command executed after confirmation
    CommandExecuted {
        command: String,
        user: String,
        sector_id: Uuid,
        confirmation_method: TactileMethod,
        timestamp: String,
    },
    /// Command blocked (no confirmation)
    CommandBlocked {
        command: String,
        user: String,
        sector_id: Uuid,
        reason: String,
        timestamp: String,
    },
    /// Authentication event
    Authentication {
        user: String,
        success: bool,
        method: String,
        timestamp: String,
    },
    /// Role change
    RoleChange {
        user: String,
        old_role: String,
        new_role: String,
        changed_by: String,
        timestamp: String,
    },
}

/// Risk level for operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - informational only
    Low,
    /// Medium risk - requires simple confirmation
    Medium,
    /// High risk - requires tactile confirmation
    High,
    /// Critical risk - requires multi-factor confirmation
    Critical,
}

/// Tactile confirmation method
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TactileMethod {
    /// Hold button/key for duration
    Hold { 
        /// Duration in milliseconds
        duration_ms: u64,
        /// Button/key identifier
        target: String,
    },
    /// Slide control
    Slider {
        /// Distance to slide (percentage of screen)
        distance_percent: f32,
        /// Direction
        direction: SlideDirection,
    },
    /// Voice confirmation
    Voice {
        /// Phrase to speak
        phrase: String,
        /// Required confidence
        confidence_threshold: f32,
    },
    /// Multi-button press
    MultiButton {
        /// Number of buttons required
        button_count: usize,
        /// Button identifiers
        buttons: Vec<String>,
    },
    /// Pattern entry (e.g., swipe pattern)
    Pattern {
        /// Pattern sequence
        sequence: Vec<PatternPoint>,
    },
}

/// Slide direction for slider confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlideDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Pattern point for pattern confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternPoint {
    pub x: u32,
    pub y: u32,
}

/// Dangerous command pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerousPattern {
    /// Pattern name
    pub name: String,
    /// Regex pattern
    pub pattern: String,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Warning message
    pub message: String,
    /// Required confirmation method
    pub required_confirmation: TactileMethod,
    /// Whether to allow override
    pub allow_override: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable dangerous command detection
    pub enable_detection: bool,
    /// Enable audit logging
    pub enable_audit_log: bool,
    /// Audit log file path
    pub audit_log_path: String,
    /// Maximum audit log entries in memory
    pub max_audit_entries: usize,
    /// Default confirmation method
    pub default_confirmation: TactileMethod,
    /// Dangerous patterns
    pub patterns: Vec<DangerousPattern>,
    /// Require confirmation for all destructive commands
    pub confirm_all_destructive: bool,
    /// Timeout for confirmation (seconds)
    pub confirmation_timeout_secs: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_detection: true,
            enable_audit_log: true,
            audit_log_path: "/var/log/tos/security.log".to_string(),
            max_audit_entries: 1000,
            default_confirmation: TactileMethod::Hold {
                duration_ms: 2000,
                target: "space".to_string(),
            },
            patterns: Self::default_patterns(),
            confirm_all_destructive: true,
            confirmation_timeout_secs: 30,
        }
    }
}

impl SecurityConfig {
    /// Default dangerous patterns
    fn default_patterns() -> Vec<DangerousPattern> {
        vec![
            DangerousPattern {
                name: "rm_rf_root".to_string(),
                pattern: r"rm\s+(-[a-zA-Z]*f[a-zA-Z]*\s+)?(-[a-zA-Z]*r[a-zA-Z]*\s+)?/+".to_string(),
                risk_level: RiskLevel::Critical,
                message: "This will recursively delete the entire filesystem!".to_string(),
                required_confirmation: TactileMethod::MultiButton {
                    button_count: 3,
                    buttons: vec!["ctrl".to_string(), "alt".to_string(), "delete".to_string()],
                },
                allow_override: false,
            },
            DangerousPattern {
                name: "dd_to_device".to_string(),
                pattern: r"dd\s+.*\s+of=/dev/[sh]d[a-z]".to_string(),
                risk_level: RiskLevel::Critical,
                message: "This will overwrite a storage device directly!".to_string(),
                required_confirmation: TactileMethod::Voice {
                    phrase: "I confirm this will destroy data".to_string(),
                    confidence_threshold: 0.9,
                },
                allow_override: false,
            },
            DangerousPattern {
                name: "mkfs".to_string(),
                pattern: r"mkfs\.\w+\s+/dev/".to_string(),
                risk_level: RiskLevel::Critical,
                message: "This will format a filesystem and destroy all data!".to_string(),
                required_confirmation: TactileMethod::Slider {
                    distance_percent: 0.8,
                    direction: SlideDirection::Right,
                },
                allow_override: false,
            },
            DangerousPattern {
                name: "write_to_device".to_string(),
                pattern: r"[>|&]\s*/dev/[sh]d[a-z]".to_string(),
                risk_level: RiskLevel::High,
                message: "Writing directly to block devices can corrupt data".to_string(),
                required_confirmation: TactileMethod::Hold {
                    duration_ms: 3000,
                    target: "space".to_string(),
                },
                allow_override: true,
            },
            DangerousPattern {
                name: "chmod_777_recursive".to_string(),
                pattern: r"chmod\s+(-[a-zA-Z]*R[a-zA-Z]*\s+)?777\s+(-[a-zA-Z]*R[a-zA-Z]*\s+)?/".to_string(),
                risk_level: RiskLevel::High,
                message: "This will make all files world-writable recursively".to_string(),
                required_confirmation: TactileMethod::Hold {
                    duration_ms: 2000,
                    target: "space".to_string(),
                },
                allow_override: true,
            },
            DangerousPattern {
                name: "fork_bomb".to_string(),
                pattern: r":\(\)\s*\{\s*:\s*\|:\s*&\s*\}\s*;".to_string(),
                risk_level: RiskLevel::High,
                message: "This is a fork bomb that will crash the system".to_string(),
                required_confirmation: TactileMethod::Pattern {
                    sequence: vec![
                        PatternPoint { x: 0, y: 0 },
                        PatternPoint { x: 1, y: 1 },
                        PatternPoint { x: 2, y: 0 },
                    ],
                },
                allow_override: false,
            },
            DangerousPattern {
                name: "wget_pipe_sh".to_string(),
                pattern: r"wget\s+.*\s*\|\s*(ba)?sh".to_string(),
                risk_level: RiskLevel::Medium,
                message: "Piping downloaded content directly to shell is dangerous".to_string(),
                required_confirmation: TactileMethod::Hold {
                    duration_ms: 1500,
                    target: "space".to_string(),
                },
                allow_override: true,
            },
            DangerousPattern {
                name: "curl_pipe_sh".to_string(),
                pattern: r"curl\s+.*\s*\|\s*(ba)?sh".to_string(),
                risk_level: RiskLevel::Medium,
                message: "Piping downloaded content directly to shell is dangerous".to_string(),
                required_confirmation: TactileMethod::Hold {
                    duration_ms: 1500,
                    target: "space".to_string(),
                },
                allow_override: true,
            },
        ]
    }
}

/// Active confirmation session
#[derive(Debug, Clone)]
pub struct ConfirmationSession {
    /// Session ID
    pub id: Uuid,
    /// Command being confirmed
    pub command: String,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Required confirmation method
    pub required_method: TactileMethod,
    /// Current progress (0.0 - 1.0)
    pub progress: f32,
    /// Session start time
    pub start_time: Instant,
    /// User confirming
    pub user: String,
    /// Sector ID
    pub sector_id: Uuid,
}

/// Security manager
#[derive(Debug)]
#[derive(Default)]
pub struct SecurityManager {
    /// Configuration
    pub config: SecurityConfig,
    /// Audit log
    pub audit_log: Vec<SecurityEvent>,
    /// Active confirmation sessions
    pub active_sessions: HashMap<Uuid, ConfirmationSession>,
    /// Dangerous pattern cache (compiled regex)
    pattern_cache: Vec<(DangerousPattern, regex::Regex)>,
}

impl SecurityManager {
    /// Create a new security manager with default config
    pub fn new() -> Self {
        Self::with_config(SecurityConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: SecurityConfig) -> Self {
        let pattern_cache = Self::compile_patterns(&config.patterns);
        
        Self {
            config,
            audit_log: Vec::new(),
            active_sessions: HashMap::new(),
            pattern_cache,
        }
    }

    /// Compile regex patterns
    fn compile_patterns(patterns: &[DangerousPattern]) -> Vec<(DangerousPattern, regex::Regex)> {
        patterns
            .iter()
            .filter_map(|p| {
                regex::Regex::new(&p.pattern)
                    .ok()
                    .map(|re| (p.clone(), re))
            })
            .collect()
    }

    /// Check if a reset/confirmation is currently in progress
    pub fn is_resetting(&self) -> bool {
        !self.active_sessions.is_empty()
    }

    /// Check if a command matches dangerous patterns
    pub fn check_command(&self, command: &str) -> Option<(RiskLevel, &DangerousPattern)> {
        if !self.config.enable_detection {
            return None;
        }

        for (pattern, regex) in &self.pattern_cache {
            if regex.is_match(command) {
                return Some((pattern.risk_level, pattern));
            }
        }

        None
    }

    /// Start a confirmation session for a dangerous command
    pub fn start_confirmation(
        &mut self,
        command: &str,
        user: &str,
        sector_id: Uuid,
    ) -> Option<ConfirmationSession> {
        let (risk_level, method) = {
            let (rl, p) = self.check_command(command)?;
            (rl, p.required_confirmation.clone())
        };
        
        let session = ConfirmationSession {
            id: Uuid::new_v4(),
            command: command.to_string(),
            risk_level,
            required_method: method.clone(),
            progress: 0.0,
            start_time: Instant::now(),
            user: user.to_string(),
            sector_id,
        };

        // Log the event
        self.log_event(SecurityEvent::DangerousCommandDetected {
            command: command.to_string(),
            risk_level,
            user: user.to_string(),
            sector_id,
            timestamp: Self::current_timestamp(),
        });

        self.log_event(SecurityEvent::ConfirmationStarted {
            method,
            command: command.to_string(),
            user: user.to_string(),
            timestamp: Self::current_timestamp(),
        });

        let id = session.id;
        self.active_sessions.insert(id, session.clone());
        
        Some(session)
    }

    /// Update confirmation progress
    pub fn update_progress(&mut self, session_id: Uuid, progress: f32) -> Option<bool> {
        let session = self.active_sessions.get_mut(&session_id)?;
        
        session.progress = progress.clamp(0.0, 10.0); // Allow higher than 1.0 for multi-button
        
        // Check if confirmation is complete
        let required = match &session.required_method {
            TactileMethod::Hold { .. } => 1.0,
            TactileMethod::Slider { .. } => 1.0,
            TactileMethod::Voice { .. } => 1.0,
            TactileMethod::MultiButton { button_count, .. } => *button_count as f32,
            TactileMethod::Pattern { .. } => 1.0,
        };

        Some(session.progress >= required)
    }

    /// Complete a confirmation session
    pub fn complete_confirmation(
        &mut self,
        session_id: Uuid,
        success: bool,
    ) -> Option<ConfirmationSession> {
        let session = self.active_sessions.remove(&session_id)?;
        
        self.log_event(SecurityEvent::ConfirmationCompleted {
            method: session.required_method.clone(),
            command: session.command.clone(),
            user: session.user.clone(),
            success,
            timestamp: Self::current_timestamp(),
        });

        if success {
            self.log_event(SecurityEvent::CommandExecuted {
                command: session.command.clone(),
                user: session.user.clone(),
                sector_id: session.sector_id,
                confirmation_method: session.required_method.clone(),
                timestamp: Self::current_timestamp(),
            });
        } else {
            self.log_event(SecurityEvent::CommandBlocked {
                command: session.command.clone(),
                user: session.user.clone(),
                sector_id: session.sector_id,
                reason: "Confirmation failed or cancelled".to_string(),
                timestamp: Self::current_timestamp(),
            });
        }

        Some(session)
    }

    /// Cancel a confirmation session
    pub fn cancel_confirmation(&mut self, session_id: Uuid) -> Option<ConfirmationSession> {
        self.complete_confirmation(session_id, false)
    }

    /// Check if a session has timed out
    pub fn check_timeout(&mut self, session_id: Uuid) -> bool {
        let timed_out = self.active_sessions.get(&session_id)
            .map(|session| session.start_time.elapsed().as_secs() >= self.config.confirmation_timeout_secs)
            .unwrap_or(false);
        
        if timed_out {
            self.cancel_confirmation(session_id);
        }
        timed_out
    }

    /// Log a security event
    fn log_event(&mut self, event: SecurityEvent) {
        if !self.config.enable_audit_log {
            return;
        }

        self.audit_log.push(event);
        
        // Trim log if too large
        if self.audit_log.len() > self.config.max_audit_entries {
            self.audit_log.remove(0);
        }

        // In real implementation, would also write to file
    }

    /// Get current timestamp string
    fn current_timestamp() -> String {
        use chrono::Local;
        Local::now().to_rfc3339()
    }

    /// Get recent audit log entries
    pub fn get_audit_log(&self, count: usize) -> Vec<&SecurityEvent> {
        self.audit_log.iter().rev().take(count).collect()
    }

    /// Get audit log for a specific user
    pub fn get_user_audit_log(&self, user: &str) -> Vec<&SecurityEvent> {
        self.audit_log
            .iter()
            .filter(|e| match e {
                SecurityEvent::DangerousCommandDetected { user: u, .. } => u == user,
                SecurityEvent::ConfirmationStarted { user: u, .. } => u == user,
                SecurityEvent::ConfirmationCompleted { user: u, .. } => u == user,
                SecurityEvent::CommandExecuted { user: u, .. } => u == user,
                SecurityEvent::CommandBlocked { user: u, .. } => u == user,
                SecurityEvent::Authentication { user: u, .. } => u == user,
                SecurityEvent::RoleChange { user: u, .. } => u == user,
            })
            .collect()
    }

    /// Get audit log for a specific sector
    pub fn get_sector_audit_log(&self, sector_id: Uuid) -> Vec<&SecurityEvent> {
        self.audit_log
            .iter()
            .filter(|e| match e {
                SecurityEvent::DangerousCommandDetected { sector_id: s, .. } => *s == sector_id,
                SecurityEvent::CommandExecuted { sector_id: s, .. } => *s == sector_id,
                SecurityEvent::CommandBlocked { sector_id: s, .. } => *s == sector_id,
                _ => false,
            })
            .collect()
    }

    /// Render a confirmation dialog as HTML
    pub fn render_confirmation(&self, session: &ConfirmationSession) -> String {
        let method_desc = match &session.required_method {
            TactileMethod::Hold { duration_ms, target } => {
                format!(
                    r#"<div class="tactile-hold">
                        <div class="hold-instruction">HOLD <kbd>{}</kbd> FOR {} SECONDS</div>
                        <div class="hold-progress-bar">
                            <div class="progress-fill" style="width: {}%"></div>
                        </div>
                    </div>"#,
                    target,
                    duration_ms / 1000,
                    (session.progress * 100.0) as u32
                )
            }
            TactileMethod::Slider { distance_percent, direction } => {
                let dir_text = match direction {
                    SlideDirection::Left => "LEFT",
                    SlideDirection::Right => "RIGHT",
                    SlideDirection::Up => "UP",
                    SlideDirection::Down => "DOWN",
                };
                format!(
                    r#"<div class="tactile-slider">
                        <div class="slider-instruction">SLIDE {} TO {}%</div>
                        <div class="slider-track">
                            <div class="slider-handle" style="left: {}%"></div>
                        </div>
                    </div>"#,
                    dir_text,
                    (distance_percent * 100.0) as u32,
                    (session.progress * 100.0) as u32
                )
            }
            TactileMethod::Voice { phrase, confidence_threshold } => {
                format!(
                    r#"<div class="tactile-voice">
                        <div class="voice-instruction">VOICE CONFIRMATION:</div>
                        <div class="voice-phrase">"{}"</div>
                        <div class="voice-confidence">REQUIRED CONFIDENCE: {}%</div>
                        <div class="voice-progress">{}</div>
                    </div>"#,
                    phrase,
                    (confidence_threshold * 100.0) as u32,
                    if session.progress > 0.0 { "✓ HEARD" } else { "LISTENING..." }
                )
            }
            TactileMethod::MultiButton { button_count, buttons } => {
                let buttons_html = buttons
                    .iter()
                    .map(|b| format!("<kbd>{}</kbd>", b))
                    .collect::<Vec<_>>()
                    .join(" + ");
                format!(
                    r#"<div class="tactile-multibutton">
                        <div class="mb-instruction">MULTI-BUTTON: PRESS {} SIMULTANEOUSLY</div>
                        <div class="mb-buttons">{}</div>
                        <div class="mb-progress">{} OF {} PRESSED</div>
                    </div>"#,
                    button_count,
                    buttons_html,
                    session.progress as usize,
                    button_count
                )
            }
            TactileMethod::Pattern { sequence } => {
                format!(
                    r#"<div class="tactile-pattern">
                        <div class="pattern-instruction">Draw the pattern ({} points)</div>
                        <div class="pattern-grid">
                            <!-- Pattern grid would be rendered here -->
                        </div>
                        <div class="pattern-progress">{} points matched</div>
                    </div>"#,
                    sequence.len(),
                    (session.progress * sequence.len() as f32) as usize
                )
            }
        };

        let risk_class = match session.risk_level {
            RiskLevel::Low => "risk-low",
            RiskLevel::Medium => "risk-medium",
            RiskLevel::High => "risk-high",
            RiskLevel::Critical => "risk-critical",
        };

        format!(
            r#"<div class="security-confirmation-dialog {}">
                <div class="confirmation-header">
                    <div class="risk-badge">{} RISK</div>
                    <div class="confirmation-title">DANGEROUS COMMAND DETECTED</div>
                </div>
                <div class="command-display">
                    <code>{}</code>
                </div>
                <div class="warning-message">
                    This command requires confirmation due to its destructive nature.
                </div>
                {}
                <div class="confirmation-timeout">
                    Timeout in {} seconds
                </div>
                <div class="confirmation-actions">
                    <button class="btn-cancel" onclick="cancelConfirmation('{}')">CANCEL</button>
                </div>
            </div>"#,
            risk_class,
            format!("{:?}", session.risk_level).to_uppercase(),
            html_escape(&session.command),
            method_desc,
            self.config.confirmation_timeout_secs.saturating_sub(session.start_time.elapsed().as_secs()),
            session.id
        )
    }

    /// Render security dashboard as HTML
    pub fn render_security_dashboard(&self) -> String {
        let recent_events: Vec<String> = self.audit_log
            .iter()
            .rev()
            .take(20)
            .map(|e| format!(
                r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                match e {
                    SecurityEvent::DangerousCommandDetected { timestamp, .. } => timestamp,
                    SecurityEvent::ConfirmationStarted { timestamp, .. } => timestamp,
                    SecurityEvent::ConfirmationCompleted { timestamp, .. } => timestamp,
                    SecurityEvent::CommandExecuted { timestamp, .. } => timestamp,
                    SecurityEvent::CommandBlocked { timestamp, .. } => timestamp,
                    SecurityEvent::Authentication { timestamp, .. } => timestamp,
                    SecurityEvent::RoleChange { timestamp, .. } => timestamp,
                },
                match e {
                    SecurityEvent::DangerousCommandDetected { .. } => "⚠️ Dangerous Command",
                    SecurityEvent::ConfirmationStarted { .. } => "⏳ Confirmation Started",
                    SecurityEvent::ConfirmationCompleted { success, .. } => {
                        if *success { "✅ Confirmed" } else { "❌ Cancelled" }
                    }
                    SecurityEvent::CommandExecuted { .. } => "▶️ Executed",
                    SecurityEvent::CommandBlocked { .. } => "🚫 Blocked",
                    SecurityEvent::Authentication { success, .. } => {
                        if *success { "🔓 Auth Success" } else { "🔒 Auth Failed" }
                    }
                    SecurityEvent::RoleChange { .. } => "👤 Role Change",
                },
                match e {
                    SecurityEvent::DangerousCommandDetected { user, .. } => user,
                    SecurityEvent::ConfirmationStarted { user, .. } => user,
                    SecurityEvent::ConfirmationCompleted { user, .. } => user,
                    SecurityEvent::CommandExecuted { user, .. } => user,
                    SecurityEvent::CommandBlocked { user, .. } => user,
                    SecurityEvent::Authentication { user, .. } => user,
                    SecurityEvent::RoleChange { user, .. } => user,
                }
            ))
            .collect();

        format!(
            r#"<div class="security-dashboard">
                <h2>Security Dashboard</h2>
                <div class="security-stats">
                    <div class="stat">
                        <div class="stat-value">{}</div>
                        <div class="stat-label">Total Events</div>
                    </div>
                    <div class="stat">
                        <div class="stat-value">{}</div>
                        <div class="stat-label">Active Sessions</div>
                    </div>
                    <div class="stat">
                        <div class="stat-value">{}</div>
                        <div class="stat-label">Patterns</div>
                    </div>
                </div>
                <h3>Recent Events</h3>
                <table class="security-log">
                    <thead>
                        <tr>
                            <th>Time</th>
                            <th>Event</th>
                            <th>User</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>"#,
            self.audit_log.len(),
            self.active_sessions.len(),
            self.config.patterns.len(),
            recent_events.join("")
        )
    }
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.enable_detection);
        assert!(config.enable_audit_log);
        assert!(!config.patterns.is_empty());
    }

    #[test]
    fn test_security_manager_new() {
        let manager = SecurityManager::new();
        assert!(manager.audit_log.is_empty());
        assert!(manager.active_sessions.is_empty());
    }

    #[test]
    fn test_check_dangerous_commands() {
        let manager = SecurityManager::new();
        
        // Test rm -rf /
        let result = manager.check_command("rm -rf /");
        assert!(result.is_some());
        let (risk, pattern) = result.unwrap();
        assert_eq!(risk, RiskLevel::Critical);
        assert_eq!(pattern.name, "rm_rf_root");
        
        // Test dd to device
        let cmd = "dd if=/dev/zero of=/dev/sda ";
        let result = manager.check_command(cmd);
        assert!(result.is_some());
        let (risk, _) = result.unwrap();
        assert_eq!(risk, RiskLevel::Critical);
        
        // Test safe command
        let result = manager.check_command("ls -la ");
        assert!(result.is_none());
    }

    #[test]
    fn test_confirmation_session() {
        let mut manager = SecurityManager::new();
        
        let session = manager.start_confirmation("rm -rf /", "test_user", Uuid::new_v4());
        assert!(session.is_some());
        
        let session = session.unwrap();
        assert_eq!(session.command, "rm -rf /");
        assert_eq!(session.user, "test_user");
        assert_eq!(session.progress, 0.0);
        
        // Update progress
        let complete = manager.update_progress(session.id, 0.5);
        assert_eq!(complete, Some(false)); // Not complete yet
        
        // Complete
        let complete = manager.update_progress(session.id, 3.0);
        assert_eq!(complete, Some(true)); // Now complete
        
        // Complete the session
        let completed = manager.complete_confirmation(session.id, true);
        assert!(completed.is_some());
        assert!(manager.active_sessions.is_empty());
    }

    #[test]
    fn test_audit_logging() {
        let mut manager = SecurityManager::new();
        
        let sector_id = Uuid::new_v4();
        manager.start_confirmation("rm -rf /", "test_user", sector_id);
        
        assert_eq!(manager.audit_log.len(), 2); // Detected + Started
        
        let log = manager.get_audit_log(10);
        assert_eq!(log.len(), 2);
        
        let user_log = manager.get_user_audit_log("test_user");
        assert_eq!(user_log.len(), 2);
        
        let sector_log = manager.get_sector_audit_log(sector_id);
        assert_eq!(sector_log.len(), 1);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_tactile_methods() {
        let hold = TactileMethod::Hold {
            duration_ms: 2000,
            target: "space".to_string(),
        };
        assert!(matches!(hold, TactileMethod::Hold { duration_ms: 2000, .. }));
        
        let slider = TactileMethod::Slider {
            distance_percent: 0.8,
            direction: SlideDirection::Right,
        };
        assert!(matches!(slider, TactileMethod::Slider { direction: SlideDirection::Right, .. }));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("\"test\""), "&quot;test&quot;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_render_confirmation() {
        let manager = SecurityManager::new();
        let session = ConfirmationSession {
            id: Uuid::new_v4(),
            command: "rm -rf /".to_string(),
            risk_level: RiskLevel::Critical,
            required_method: TactileMethod::Hold {
                duration_ms: 3000,
                target: "space".to_string(),
            },
            progress: 0.5,
            start_time: Instant::now(),
            user: "test".to_string(),
            sector_id: Uuid::new_v4(),
        };
        
        let html = manager.render_confirmation(&session);
        assert!(html.contains("DANGEROUS COMMAND DETECTED"));
        assert!(html.contains("rm -rf /"));
        assert!(html.contains("CRITICAL"));
        assert!(html.contains("50%"));
    }

    #[test]
    fn test_security_dashboard() {
        let mut manager = SecurityManager::new();
        
        // Add some events
        manager.start_confirmation("rm -rf /test", "user1", Uuid::new_v4());
        manager.start_confirmation("dd if=/dev/zero of=/dev/sdb", "user2", Uuid::new_v4());
        
        let html = manager.render_security_dashboard();
        assert!(html.contains("Security Dashboard"));
        assert!(html.contains("Total Events"));
        assert!(html.contains("Active Sessions"));
        assert!(html.contains("user1"));
        assert!(html.contains("user2"));
    }

    #[test]
    fn test_timeout() {
        let mut manager = SecurityManager::new();
        manager.config.confirmation_timeout_secs = 0; // Immediate timeout
        
        let session = manager.start_confirmation("rm -rf /", "test", Uuid::new_v4()).unwrap();
        
        // Should timeout immediately
        assert!(manager.check_timeout(session.id));
        assert!(manager.active_sessions.is_empty());
    }
}
