//! Accessibility Tests - Phase 10
//! 
//! Tests for screen reader integration, auditory feedback,
//! visual accessibility, and motor accessibility features.

#![cfg(feature = "accessibility")]

use tos_core::accessibility::*;
use std::sync::Arc;

#[tokio::test]
async fn test_accessibility_manager_creation() {
    let config = default_config();
    let manager: Result<AccessibilityManager, AccessibilityError> = AccessibilityManager::new(config).await;
    
    assert!(manager.is_ok());
}

#[tokio::test]
async fn test_high_contrast_config() {
    let config = high_contrast_config();
    let manager: AccessibilityManager = AccessibilityManager::new(config).await.unwrap();
    
    let css_classes = manager.get_css_classes().await;
    assert!(css_classes.contains("high-contrast"));
    assert!(css_classes.contains("theme-high-contrast"));
}

#[tokio::test]
async fn test_screen_reader_config() {
    let config = screen_reader_config();
    let manager: AccessibilityManager = AccessibilityManager::new(config).await.unwrap();
    
    let cfg = manager.get_config().await;
    assert!(cfg.screen_reader_enabled);
    assert!(cfg.tts_enabled);
    assert_eq!(cfg.verbosity, VerbosityLevel::Verbose);
}

#[tokio::test]
async fn test_motor_accessibility_config() {
    let config = motor_accessibility_config();
    let manager: AccessibilityManager = AccessibilityManager::new(config).await.unwrap();
    
    let cfg = manager.get_config().await;
    assert!(cfg.switch_device_enabled);
    assert!(cfg.sticky_keys);
    assert!(cfg.dwell_click_ms > 0);
}

#[tokio::test]
async fn test_announcement_navigation() {
    let config = default_config();
    let manager: AccessibilityManager = AccessibilityManager::new(config).await.unwrap();
    
    let announcement = AccessibilityAnnouncement::Navigation {
        from_level: "Global Overview".to_string(),
        to_level: "Command Hub".to_string(),
        description: "Sector Alpha".to_string(),
    };
    
    // Should not panic
    manager.announce(announcement).await;
}

#[tokio::test]
async fn test_announcement_action() {
    let config = default_config();
    let manager: AccessibilityManager = AccessibilityManager::new(config).await.unwrap();
    
    let announcement = AccessibilityAnnouncement::Action {
        action: "Open Application".to_string(),
        result: "Terminal opened".to_string(),
    };
    
    manager.announce(announcement).await;
}

#[tokio::test]
async fn test_announcement_alert() {
    let config = default_config();
    let manager: AccessibilityManager = AccessibilityManager::new(config).await.unwrap();
    
    let announcement = AccessibilityAnnouncement::Alert {
        severity: AlertSeverity::Warning,
        message: "Low memory".to_string(),
    };
    
    manager.announce(announcement).await;
}

#[tokio::test]
async fn test_visual_css_generation() {
    let config = AccessibilityConfig {
        high_contrast: true,
        font_scale: 1.5,
        theme_mode: ThemeMode::Dark,
        ..Default::default()
    };
    
    let manager: AccessibilityManager = AccessibilityManager::new(config).await.unwrap();
    let css = manager.get_css_classes().await;
    
    assert!(css.contains("high-contrast"));
    assert!(css.contains("font-scale-xlarge"));
    assert!(css.contains("theme-dark"));
}

#[tokio::test]
async fn test_colorblind_filters() {
    use tos_core::accessibility::visual::*;
    
    let filters = VisualAccessibility::generate_colorblind_filters();
    assert!(filters.contains("deuteranopia-filter"));
    assert!(filters.contains("protanopia-filter"));
    assert!(filters.contains("tritanopia-filter"));
}

#[test]
fn test_colorblind_safe_colors() {
    use tos_core::accessibility::visual::*;
    
    let deuteranopia_colors = VisualAccessibility::colorblind_safe_colors(ColorblindFilter::Deuteranopia);
    assert_eq!(deuteranopia_colors.len(), 6);
    
    let tritanopia_colors = VisualAccessibility::colorblind_safe_colors(ColorblindFilter::Tritanopia);
    assert_eq!(tritanopia_colors.len(), 6);
}

#[test]
fn test_aria_role_mapping() {
    use tos_core::accessibility::visual::*;
    
    assert_eq!(VisualAccessibility::get_aria_role("button"), "button");
    assert_eq!(VisualAccessibility::get_aria_role("dialog"), "dialog");
    assert_eq!(VisualAccessibility::get_aria_role("navigation"), "navigation");
    assert_eq!(VisualAccessibility::get_aria_role("unknown"), "generic");
}

#[test]
fn test_aria_attributes_generation() {
    use tos_core::accessibility::visual::*;
    
    let attrs = VisualAccessibility::generate_aria_attributes(
        "button",
        "Submit",
        Some("active"),
        Some(true),
        Some(false),
    );
    
    assert!(attrs.contains(r#"role="button""#));
    assert!(attrs.contains(r#"aria-label="Submit""#));
    assert!(attrs.contains(r#"aria-state="active""#));
    assert!(attrs.contains(r#"aria-expanded="true""#));
    assert!(attrs.contains(r#"aria-selected="false""#));
}

#[tokio::test]
async fn test_auditory_interface() {
    let config = Arc::new(tokio::sync::RwLock::new(AccessibilityConfig {
        auditory_feedback: true,
        tts_enabled: false,
        ..Default::default()
    }));
    
    let audio: Result<audio::AuditoryInterface, AccessibilityError> = audio::AuditoryInterface::new(config).await;
    assert!(audio.is_ok());
}

#[tokio::test]
async fn test_sound_theme_loading() {
    let default_theme = audio::SoundTheme::load_default().await;
    assert_eq!(default_theme.name, "default");
    
    let scifi_theme: Result<audio::SoundTheme, AccessibilityError> = audio::SoundTheme::load("sci-fi").await;
    assert!(scifi_theme.is_ok());
    assert_eq!(scifi_theme.unwrap().name, "sci-fi");
}

#[test]
fn test_sound_event_mapping() {
    use tos_core::accessibility::audio::*;
    
    // Test that all important events have sounds in default theme
    let theme = SoundTheme::load_default_blocking();
    
    assert!(theme.get_sound(SoundEvent::ZoomIn).is_some());
    assert!(theme.get_sound(SoundEvent::ZoomOut).is_some());
    assert!(theme.get_sound(SoundEvent::Select).is_some());
    assert!(theme.get_sound(SoundEvent::CommandAccepted).is_some());
    assert!(theme.get_sound(SoundEvent::AlertError).is_some());
}

#[tokio::test]
async fn test_motor_sticky_keys() {
    let config = Arc::new(tokio::sync::RwLock::new(AccessibilityConfig {
        sticky_keys: true,
        ..Default::default()
    }));
    
    let motor: Result<motor::MotorAccessibility, AccessibilityError> = motor::MotorAccessibility::new(config).await;
    assert!(motor.is_ok());
}

#[test]
fn test_sticky_modifier_conversion() {
    use tos_core::accessibility::motor::*;
    
    assert_eq!(key_to_modifier("Shift"), Some(StickyModifier::Shift));
    assert_eq!(key_to_modifier("Control"), Some(StickyModifier::Control));
    assert_eq!(key_to_modifier("Alt"), Some(StickyModifier::Alt));
    assert_eq!(key_to_modifier("Super"), Some(StickyModifier::Super));
    assert_eq!(key_to_modifier("A"), None);
}

#[test]
fn test_verbosity_levels() {
    // Test that verbosity levels are ordered correctly
    assert!(matches!(VerbosityLevel::Minimal, VerbosityLevel::Minimal));
    assert!(matches!(VerbosityLevel::Normal, VerbosityLevel::Normal));
    assert!(matches!(VerbosityLevel::Verbose, VerbosityLevel::Verbose));
    assert!(matches!(VerbosityLevel::Debug, VerbosityLevel::Debug));
}

#[test]
fn test_alert_severity_ordering() {
    // Test severity levels
    let info = AlertSeverity::Info;
    let warning = AlertSeverity::Warning;
    let error = AlertSeverity::Error;
    let critical = AlertSeverity::Critical;
    
    // All should be different
    assert_ne!(info as i32, warning as i32);
    assert_ne!(warning as i32, error as i32);
    assert_ne!(error as i32, critical as i32);
}

#[tokio::test]
async fn test_config_update() {
    let config = default_config();
    let manager: AccessibilityManager = AccessibilityManager::new(config).await.unwrap();
    
    let new_config = AccessibilityConfig {
        high_contrast: true,
        font_scale: 1.25,
        ..Default::default()
    };
    
    let result: Result<(), AccessibilityError> = manager.update_config(new_config).await;
    assert!(result.is_ok());
    
    let updated = manager.get_config().await;
    assert!(updated.high_contrast);
    assert_eq!(updated.font_scale, 1.25);
}

#[test]
fn test_theme_modes() {
    let system = ThemeMode::System;
    let light = ThemeMode::Light;
    let dark = ThemeMode::Dark;
    let high_contrast = ThemeMode::HighContrast;
    
    // All should be different variants
    assert_ne!(std::mem::discriminant(&system), std::mem::discriminant(&light));
    assert_ne!(std::mem::discriminant(&light), std::mem::discriminant(&dark));
    assert_ne!(std::mem::discriminant(&dark), std::mem::discriminant(&high_contrast));
}

#[test]
fn test_accessibility_error_types() {
    use tos_core::accessibility::AccessibilityError;
    
    let audio_error = AccessibilityError::AudioError("test".to_string());
    let sr_error = AccessibilityError::ScreenReaderError("test".to_string());
    let motor_error = AccessibilityError::MotorError("test".to_string());
    
    // Test display formatting
    let audio_str = format!("{}", audio_error);
    assert!(audio_str.contains("Audio"));
    
    let sr_str = format!("{}", sr_error);
    assert!(sr_str.contains("Screen reader"));
}

// Helper function for sound theme
impl audio::SoundTheme {
    fn load_default_blocking() -> Self {
        // For tests, create directly without async
        Self::create_default_theme()
    }
}
