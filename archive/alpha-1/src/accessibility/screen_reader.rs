//! Screen Reader Integration - AT-SPI and Orca Support
//! 
//! Provides integration with Linux accessibility infrastructure
//! including AT-SPI for screen reader support.

use super::{AccessibilityAnnouncement, AccessibilityConfig, AccessibilityError, AlertSeverity, VerbosityLevel};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Screen reader interface using AT-SPI
#[derive(Debug)]
pub struct ScreenReader {
    config: Arc<RwLock<AccessibilityConfig>>,
    connection: Option<AtspiConnection>,
    enabled: bool,
}

/// AT-SPI connection wrapper
#[derive(Debug)]
struct AtspiConnection {
    _bus_address: String,
    _app_name: String,
}

/// Screen reader announcement priority
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnouncementPriority {
    Immediate,  // Interrupt current speech
    Important,  // Queue after current
    Normal,     // Standard queue
    Background, // Only if idle
}

impl ScreenReader {
    /// Create a new screen reader interface
    pub async fn new(config: Arc<RwLock<AccessibilityConfig>>) -> Result<Self, AccessibilityError> {
        let screen_reader_enabled = config.read().await.screen_reader_enabled;
        
        if !screen_reader_enabled {
            return Ok(Self {
                config,
                connection: None,
                enabled: false,
            });
        }
        
        // Try to establish AT-SPI connection
        let connection = match Self::connect_atspi().await {
            Ok(conn) => {
                tracing::info!("AT-SPI connection established");
                Some(conn)
            }
            Err(e) => {
                tracing::warn!("Failed to connect to AT-SPI: {}", e);
                None
            }
        };
        
        let enabled = connection.is_some();
        Ok(Self {
            config,
            connection,
            enabled,
        })
    }
    
    /// Announce a message to the screen reader
    pub async fn announce(&self, announcement: &AccessibilityAnnouncement) -> Result<(), AccessibilityError> {
        if !self.enabled {
            return Ok(());
        }
        
        let config = self.config.read().await;
        let text = self.format_for_screen_reader(announcement, config.verbosity);
        let priority = self.announcement_to_priority(announcement);
        
        // Send to AT-SPI
        if let Some(ref conn) = self.connection {
            self.send_atspi_announcement(conn, &text, priority).await?;
        }
        
        // Send to Braille if enabled
        self.send_braille(&text).await?;
        
        // Also log for debugging
        tracing::info!("Screen reader: {}", text);
        
        Ok(())
    }
    
    /// Announce raw text
    pub async fn speak(&self, text: &str, priority: AnnouncementPriority) -> Result<(), AccessibilityError> {
        if !self.enabled {
            return Ok(());
        }
        
        if let Some(ref conn) = self.connection {
            self.send_atspi_announcement(conn, text, priority).await?;
        }
        
        tracing::info!("Screen reader: {}", text);
        Ok(())
    }
    
    /// Update the accessible name of a UI element
    pub async fn set_accessible_name(&self, element_id: &str, name: &str) -> Result<(), AccessibilityError> {
        if !self.enabled {
            return Ok(());
        }
        
        tracing::debug!("Set accessible name for {}: {}", element_id, name);
        
        // In a full implementation, this would update the AT-SPI object
        // For now, just log
        Ok(())
    }
    
    /// Update the accessible description of a UI element
    pub async fn set_accessible_description(
        &self,
        element_id: &str,
        description: &str,
    ) -> Result<(), AccessibilityError> {
        if !self.enabled {
            return Ok(());
        }
        
        tracing::debug!("Set accessible description for {}: {}", element_id, description);
        Ok(())
    }
    
    /// Notify that focus has changed
    pub async fn focus_changed(&self, element_id: &str, element_type: &str, label: &str) -> Result<(), AccessibilityError> {
        if !self.enabled {
            return Ok(());
        }
        
        let announcement = format!("{} {}: {}", element_type, element_id, label);
        self.speak(&announcement, AnnouncementPriority::Immediate).await?;
        
        Ok(())
    }
    
    /// Notify that a value has changed
    pub async fn value_changed(&self, element_id: &str, value: &str) -> Result<(), AccessibilityError> {
        if !self.enabled {
            return Ok(());
        }
        
        let announcement = format!("{}: {}", element_id, value);
        self.speak(&announcement, AnnouncementPriority::Normal).await?;
        
        Ok(())
    }
    
    /// Shutdown the screen reader connection
    pub async fn shutdown(&self) -> Result<(), AccessibilityError> {
        tracing::info!("Screen reader shutdown");
        Ok(())
    }

    /// Send text to Braille display
    pub async fn send_braille(&self, text: &str) -> Result<(), AccessibilityError> {
        let config = self.config.read().await;
        if !config.braille_output_enabled {
            return Ok(());
        }
        
        tracing::info!("Braille output: {}", text);
        // In a real implementation on Linux, this might:
        // 1. Connect to brltty via its API
        // 2. Or send to AT-SPI Braille interface
        
        Ok(())
    }
    
    /// Check if screen reader is active
    pub fn is_active(&self) -> bool {
        self.enabled
    }
    
    /// Format announcement for screen reader based on verbosity
    fn format_for_screen_reader(&self, announcement: &AccessibilityAnnouncement, verbosity: VerbosityLevel) -> String {
        match announcement {
            AccessibilityAnnouncement::Navigation { from_level, to_level, description } => {
                match verbosity {
                    VerbosityLevel::Minimal => format!("{}", to_level),
                    VerbosityLevel::Normal => format!("{}: {}", to_level, description),
                    VerbosityLevel::Verbose => {
                        format!("Navigated from {} to {}. {}", from_level, to_level, description)
                    }
                    VerbosityLevel::Debug => {
                        format!("Navigation: {} -> {} | {}", from_level, to_level, description)
                    }
                }
            }
            AccessibilityAnnouncement::Action { action, result } => {
                match verbosity {
                    VerbosityLevel::Minimal => action.clone(),
                    VerbosityLevel::Normal => format!("{}: {}", action, result),
                    VerbosityLevel::Verbose => format!("Action {} completed. {}", action, result),
                    VerbosityLevel::Debug => format!("Action: {} = {}", action, result),
                }
            }
            AccessibilityAnnouncement::Status { component, state } => {
                match verbosity {
                    VerbosityLevel::Minimal => state.clone(),
                    VerbosityLevel::Normal => format!("{} {}", component, state),
                    VerbosityLevel::Verbose => format!("{} is now {}", component, state),
                    VerbosityLevel::Debug => format!("Status: {} = {}", component, state),
                }
            }
            AccessibilityAnnouncement::Alert { severity, message } => {
                let prefix = match severity {
                    AlertSeverity::Info => "",
                    AlertSeverity::Warning => "Warning: ",
                    AlertSeverity::Error => "Error: ",
                    AlertSeverity::Critical => "Critical: ",
                };
                format!("{}{}", prefix, message)
            }
            AccessibilityAnnouncement::Collaboration { event_type, participant } => {
                match verbosity {
                    VerbosityLevel::Minimal => participant.clone(),
                    VerbosityLevel::Normal => format!("{} {}", participant, event_type),
                    VerbosityLevel::Verbose => format!("{} has {}", participant, event_type),
                    VerbosityLevel::Debug => format!("Collaboration: {} {}", participant, event_type),
                }
            }
        }
    }
    
    /// Convert announcement to priority
    fn announcement_to_priority(&self, announcement: &AccessibilityAnnouncement) -> AnnouncementPriority {
        match announcement {
            AccessibilityAnnouncement::Alert { severity, .. } => match severity {
                AlertSeverity::Critical => AnnouncementPriority::Immediate,
                AlertSeverity::Error => AnnouncementPriority::Important,
                AlertSeverity::Warning => AnnouncementPriority::Normal,
                AlertSeverity::Info => AnnouncementPriority::Background,
            },
            AccessibilityAnnouncement::Navigation { .. } => AnnouncementPriority::Normal,
            AccessibilityAnnouncement::Action { .. } => AnnouncementPriority::Normal,
            AccessibilityAnnouncement::Status { .. } => AnnouncementPriority::Background,
            AccessibilityAnnouncement::Collaboration { .. } => AnnouncementPriority::Normal,
        }
    }
    
    /// Connect to AT-SPI bus
    async fn connect_atspi() -> Result<AtspiConnection, AccessibilityError> {
        // Check for AT_SPI_BUS_ADDRESS environment variable
        let bus_address = std::env::var("AT_SPI_BUS")
            .or_else(|_| std::env::var("DBUS_SESSION_BUS_ADDRESS"))
            .unwrap_or_else(|_| "unix:path=/run/user/1000/bus".to_string());
        
        // In a full implementation, this would:
        // 1. Connect to the D-Bus session bus
        // 2. Query the AT-SPI registry
        // 3. Register the application
        
        // For now, check if accessibility is available
        if !Self::check_accessibility_available().await {
            return Err(AccessibilityError::ScreenReaderError(
                "AT-SPI not available. Is a screen reader running?".to_string()
            ));
        }
        
        Ok(AtspiConnection {
            _bus_address: bus_address,
            _app_name: "TOS".to_string(),
        })
    }
    
    /// Check if accessibility infrastructure is available
    async fn check_accessibility_available() -> bool {
        // Check for common screen reader processes
        let screen_readers = ["orca", "espeak", "speech-dispatcher"];
        
        for reader in &screen_readers {
            if Self::is_process_running(reader).await {
                return true;
            }
        }
        
        // Check for AT-SPI bus
        if std::env::var("AT_SPI_BUS").is_ok() {
            return true;
        }
        
        false
    }
    
    /// Check if a process is running
    async fn is_process_running(name: &str) -> bool {
        // Simple check using pgrep
        match tokio::process::Command::new("pgrep")
            .arg("-x")
            .arg(name)
            .output()
            .await
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
    
    /// Send announcement via AT-SPI
    async fn send_atspi_announcement(
        &self,
        _conn: &AtspiConnection,
        text: &str,
        priority: AnnouncementPriority,
    ) -> Result<(), AccessibilityError> {
        // In a full implementation using the atspi crate:
        // 1. Get the accessible object for the application
        // 2. Emit a notification event
        // 3. Or use the speech interface directly
        
        // For now, try using speech-dispatcher directly as fallback
        let priority_flag = match priority {
            AnnouncementPriority::Immediate => "-p",
            AnnouncementPriority::Important => "-p",
            AnnouncementPriority::Normal => "",
            AnnouncementPriority::Background => "-x",
        };
        
        if !priority_flag.is_empty() {
            let _ = tokio::process::Command::new("spd-say")
                .arg(priority_flag)
                .arg(text)
                .output()
                .await;
        } else {
            let _ = tokio::process::Command::new("spd-say")
                .arg(text)
                .output()
                .await;
        }
        
        Ok(())
    }
}

/// Generate ARIA live region HTML for dynamic content
pub fn generate_live_region(id: &str, priority: &str, content: &str) -> String {
    format!(
        r#"<div id="{}" aria-live="{}" aria-atomic="true" class="sr-only">{}</div>"#,
        id, priority, content
    )
}

/// Generate ARIA landmark regions
pub fn generate_landmark_regions() -> String {
    r#"
<nav aria-label="Global Navigation" role="navigation"></nav>
<main role="main"></main>
<aside aria-label="Mini Map" role="complementary"></aside>
<footer role="contentinfo"></footer>
"#.to_string()
}

/// Check if running in an accessible environment
pub async fn detect_accessibility_needs() -> AccessibilityNeeds {
    let mut needs = AccessibilityNeeds::default();
    
    // Check for high contrast preference
    if let Ok(output) = tokio::process::Command::new("gsettings")
        .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
        .await
    {
        let theme = String::from_utf8_lossy(&output.stdout);
        if theme.contains("contrast") || theme.contains("High") {
            needs.high_contrast = true;
        }
    }
    
    // Check for reduced motion preference
    if let Ok(output) = tokio::process::Command::new("gsettings")
        .args(&["get", "org.gnome.desktop.interface", "enable-animations"])
        .output()
        .await
    {
        let animations = String::from_utf8_lossy(&output.stdout);
        if animations.contains("false") {
            needs.reduced_motion = true;
        }
    }
    
    // Check if screen reader is active
    needs.screen_reader = ScreenReader::check_accessibility_available().await;
    
    needs
}

/// Detected accessibility needs
#[derive(Debug, Default)]
pub struct AccessibilityNeeds {
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub screen_reader: bool,
    pub large_text: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_announcement_priority() {
        let sr = ScreenReader {
            config: Arc::new(RwLock::new(AccessibilityConfig::default())),
            connection: None,
            enabled: true,
        };
        
        let nav = AccessibilityAnnouncement::Navigation {
            from_level: "L1".to_string(),
            to_level: "L2".to_string(),
            description: "Test".to_string(),
        };
        
        assert_eq!(sr.announcement_to_priority(&nav), AnnouncementPriority::Normal);
        
        let alert = AccessibilityAnnouncement::Alert {
            severity: AlertSeverity::Critical,
            message: "Test".to_string(),
        };
        
        assert_eq!(sr.announcement_to_priority(&alert), AnnouncementPriority::Immediate);
    }

    #[test]
    fn test_format_for_screen_reader() {
        let sr = ScreenReader {
            config: Arc::new(RwLock::new(AccessibilityConfig::default())),
            connection: None,
            enabled: true,
        };
        
        let announcement = AccessibilityAnnouncement::Navigation {
            from_level: "Global".to_string(),
            to_level: "Hub".to_string(),
            description: "Sector 1".to_string(),
        };
        
        let minimal = sr.format_for_screen_reader(&announcement, VerbosityLevel::Minimal);
        assert_eq!(minimal, "Hub");
        
        let verbose = sr.format_for_screen_reader(&announcement, VerbosityLevel::Verbose);
        assert!(verbose.contains("Navigated"));
        assert!(verbose.contains("Sector 1"));
    }

    #[test]
    fn test_live_region_generation() {
        let html = generate_live_region("status", "polite", "Operation complete");
        assert!(html.contains(r#"id="status""#));
        assert!(html.contains(r#"aria-live="polite""#));
        assert!(html.contains("Operation complete"));
    }
}
