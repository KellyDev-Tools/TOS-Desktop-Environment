//! Tests for Security Module - Dangerous Command Handling (Phase 11)

use tos_core::system::security::{
    SecurityManager, SecurityConfig, SecurityEvent, RiskLevel, 
    TactileMethod, SlideDirection, PatternPoint, DangerousPattern,
    ConfirmationSession
};
use uuid::Uuid;
use std::time::Duration;

#[test]
fn test_security_config_default() {
    let config = SecurityConfig::default();
    
    assert!(config.enable_detection);
    assert!(config.enable_audit_log);
    assert_eq!(config.audit_log_path, "/var/log/tos/security.log");
    assert_eq!(config.max_audit_entries, 1000);
    assert!(config.confirm_all_destructive);
    assert_eq!(config.confirmation_timeout_secs, 30);
    assert!(!config.patterns.is_empty());
}

#[test]
fn test_security_manager_creation() {
    let manager = SecurityManager::new();
    
    assert!(manager.audit_log.is_empty());
    assert!(manager.active_sessions.is_empty());
    assert!(!manager.is_resetting());
}

#[test]
fn test_risk_level_ordering() {
    assert!(RiskLevel::Low < RiskLevel::Medium);
    assert!(RiskLevel::Medium < RiskLevel::High);
    assert!(RiskLevel::High < RiskLevel::Critical);
    
    // Test equality
    assert_eq!(RiskLevel::Low, RiskLevel::Low);
    assert_ne!(RiskLevel::Low, RiskLevel::High);
}

#[test]
fn test_check_dangerous_commands() {
    let manager = SecurityManager::new();
    
    // Critical: rm -rf /
    let result = manager.check_command("rm -rf /");
    assert!(result.is_some());
    let (risk, pattern) = result.unwrap();
    assert_eq!(risk, RiskLevel::Critical);
    assert_eq!(pattern.name, "rm_rf_root");
    assert!(!pattern.allow_override);
    
    // Critical: dd to device
    let cmd = "dd if=/dev/zero of=/dev/sda";
    let result = manager.check_command(cmd);
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::Critical);
    
    // Critical: mkfs
    let result = manager.check_command("mkfs.ext4 /dev/sdb1");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::Critical);
    
    // High: write to device
    let result = manager.check_command("cat file > /dev/sda");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::High);
    
    // High: chmod 777 recursive
    let result = manager.check_command("chmod -R 777 /");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::High);
    
    // High: fork bomb
    let result = manager.check_command(":(){ :|:& };:");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::High);
    
    // Medium: wget pipe
    let result = manager.check_command("wget http://example.com/script.sh | bash");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::Medium);
    
    // Medium: curl pipe
    let result = manager.check_command("curl -s http://example.com | sh");
    assert!(result.is_some());
    let (risk, _) = result.unwrap();
    assert_eq!(risk, RiskLevel::Medium);
    
    // Safe commands
    assert!(manager.check_command("ls -la").is_none());
    assert!(manager.check_command("cd /home").is_none());
    assert!(manager.check_command("cat file.txt").is_none());
    assert!(manager.check_command("echo hello").is_none());
}

#[test]
fn test_confirmation_session_lifecycle() {
    let mut manager = SecurityManager::new();
    let user = "test_user".to_string();
    let sector_id = Uuid::new_v4();
    
    // Start confirmation
    let session = manager.start_confirmation("rm -rf /", &user, sector_id);
    assert!(session.is_some());
    
    let session = session.unwrap();
    assert_eq!(session.command, "rm -rf /");
    assert_eq!(session.user, user);
    assert_eq!(session.sector_id, sector_id);
    assert_eq!(session.risk_level, RiskLevel::Critical);
    assert_eq!(session.progress, 0.0);
    assert!(manager.active_sessions.contains_key(&session.id));
    
    // Check audit log
    assert_eq!(manager.audit_log.len(), 2); // Detected + Started
    
    // Update progress
    let complete = manager.update_progress(session.id, 0.5);
    assert_eq!(complete, Some(false)); // Not complete yet
    
    let session_ref = manager.active_sessions.get(&session.id).unwrap();
    assert_eq!(session_ref.progress, 0.5);
    
    // Complete progress
    let complete = manager.update_progress(session.id, 3.0);
    assert_eq!(complete, Some(true)); // Now complete
    
    // Complete confirmation
    let completed = manager.complete_confirmation(session.id, true);
    assert!(completed.is_some());
    assert!(manager.active_sessions.is_empty());
    
    // Check audit log again
    assert_eq!(manager.audit_log.len(), 4); // + Completed + Executed
}

#[test]
fn test_cancel_confirmation() {
    let mut manager = SecurityManager::new();
    let sector_id = Uuid::new_v4();
    
    let session = manager.start_confirmation("rm -rf /", "user", sector_id).unwrap();
    let id = session.id;
    
    // Cancel
    let cancelled = manager.cancel_confirmation(id);
    assert!(cancelled.is_some());
    assert!(manager.active_sessions.is_empty());
    
    // Check blocked event in audit log
    let blocked_events: Vec<_> = manager.audit_log.iter()
        .filter(|e| matches!(e, SecurityEvent::CommandBlocked { .. }))
        .collect();
    assert_eq!(blocked_events.len(), 1);
}

#[test]
fn test_confirmation_timeout() {
    let mut manager = SecurityManager::new();
    manager.config.confirmation_timeout_secs = 0; // Immediate timeout
    let sector_id = Uuid::new_v4();
    
    let session = manager.start_confirmation("rm -rf /", "user", sector_id).unwrap();
    let id = session.id;
    
    // Should timeout immediately
    assert!(manager.check_timeout(id));
    assert!(manager.active_sessions.is_empty());
}

#[test]
fn test_tactile_methods() {
    // Hold
    let hold = TactileMethod::Hold {
        duration_ms: 3000,
        target: "space".to_string(),
    };
    assert!(matches!(hold, TactileMethod::Hold { duration_ms: 3000, target } if target == "space"));
    
    // Slider
    let slider = TactileMethod::Slider {
        distance_percent: 0.8,
        direction: SlideDirection::Right,
    };
    assert!(matches!(slider, TactileMethod::Slider { distance_percent: 0.8, direction: SlideDirection::Right }));
    
    // Voice
    let voice = TactileMethod::Voice {
        phrase: "I confirm".to_string(),
        confidence_threshold: 0.9,
    };
    assert!(matches!(voice, TactileMethod::Voice { phrase, confidence_threshold: 0.9 } if phrase == "I confirm"));
    
    // MultiButton
    let mb = TactileMethod::MultiButton {
        button_count: 3,
        buttons: vec!["ctrl".to_string(), "alt".to_string(), "delete".to_string()],
    };
    assert!(matches!(mb, TactileMethod::MultiButton { button_count: 3, .. }));
    
    // Pattern
    let pattern = TactileMethod::Pattern {
        sequence: vec![
            PatternPoint { x: 0, y: 0 },
            PatternPoint { x: 1, y: 1 },
            PatternPoint { x: 2, y: 0 },
        ],
    };
    assert!(matches!(pattern, TactileMethod::Pattern { sequence } if sequence.len() == 3));
}

#[test]
fn test_slide_direction_variants() {
    let directions = vec![
        SlideDirection::Left,
        SlideDirection::Right,
        SlideDirection::Up,
        SlideDirection::Down,
    ];
    
    for dir in directions {
        let _ = format!("{:?}", dir);
    }
}

#[test]
fn test_audit_logging() {
    let mut manager = SecurityManager::new();
    let sector_id = Uuid::new_v4();
    
    // Start confirmation
    manager.start_confirmation("rm -rf /test", "user1", sector_id);
    
    // Start another
    let sector_id2 = Uuid::new_v4();
    manager.start_confirmation("dd if=/dev/zero of=/dev/sdb", "user2", sector_id2);
    
    // Check total events
    assert_eq!(manager.audit_log.len(), 4); // 2 detected + 2 started
    
    // Get recent log
    let recent = manager.get_audit_log(3);
    assert_eq!(recent.len(), 3);
    
    // Get user log
    let user1_log = manager.get_user_audit_log("user1");
    assert_eq!(user1_log.len(), 2);
    
    // Get sector log
    let sector_log = manager.get_sector_audit_log(sector_id);
    assert_eq!(sector_log.len(), 1);
}

#[test]
fn test_audit_log_limit() {
    let mut manager = SecurityManager::new();
    manager.config.max_audit_entries = 5;
    let sector_id = Uuid::new_v4();
    
    // Add more events than limit
    for i in 0..10 {
        manager.start_confirmation("rm -rf /", "user", sector_id);
    }
    
    // Should be limited
    assert_eq!(manager.audit_log.len(), 5);
}

#[test]
fn test_security_event_variants() {
    let events = vec![
        SecurityEvent::DangerousCommandDetected {
            command: "rm -rf /".to_string(),
            risk_level: RiskLevel::Critical,
            user: "user".to_string(),
            sector_id: Uuid::new_v4(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
        SecurityEvent::ConfirmationStarted {
            method: TactileMethod::Hold { duration_ms: 2000, target: "space".to_string() },
            command: "cmd".to_string(),
            user: "user".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
        SecurityEvent::ConfirmationCompleted {
            method: TactileMethod::Hold { duration_ms: 2000, target: "space".to_string() },
            command: "cmd".to_string(),
            user: "user".to_string(),
            success: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
        SecurityEvent::CommandExecuted {
            command: "cmd".to_string(),
            user: "user".to_string(),
            sector_id: Uuid::new_v4(),
            confirmation_method: TactileMethod::Hold { duration_ms: 2000, target: "space".to_string() },
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
        SecurityEvent::CommandBlocked {
            command: "cmd".to_string(),
            user: "user".to_string(),
            sector_id: Uuid::new_v4(),
            reason: "timeout".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
        SecurityEvent::Authentication {
            user: "user".to_string(),
            success: true,
            method: "password".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
        SecurityEvent::RoleChange {
            user: "user".to_string(),
            old_role: "Viewer".to_string(),
            new_role: "Operator".to_string(),
            changed_by: "admin".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
    ];
    
    for event in events {
        let _ = format!("{:?}", event);
    }
}

#[test]
fn test_dangerous_pattern_creation() {
    let pattern = DangerousPattern {
        name: "test_pattern".to_string(),
        pattern: r"test.*command".to_string(),
        risk_level: RiskLevel::Medium,
        message: "Test warning".to_string(),
        required_confirmation: TactileMethod::Hold {
            duration_ms: 2000,
            target: "space".to_string(),
        },
        allow_override: true,
    };
    
    assert_eq!(pattern.name, "test_pattern");
    assert_eq!(pattern.risk_level, RiskLevel::Medium);
    assert!(pattern.allow_override);
}

#[test]
fn test_pattern_point() {
    let point = PatternPoint { x: 100, y: 200 };
    assert_eq!(point.x, 100);
    assert_eq!(point.y, 200);
}

#[test]
fn test_html_escape() {
    // Test the html_escape function through render_confirmation
    let manager = SecurityManager::new();
    let session = ConfirmationSession {
        id: Uuid::new_v4(),
        command: "<script>alert('xss')</script>".to_string(),
        risk_level: RiskLevel::High,
        required_method: TactileMethod::Hold {
            duration_ms: 2000,
            target: "space".to_string(),
        },
        progress: 0.5,
        start_time: std::time::Instant::now(),
        user: "user".to_string(),
        sector_id: Uuid::new_v4(),
    };
    
    let html = manager.render_confirmation(&session);
    
    // Should escape HTML
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn test_render_confirmation_variants() {
    let manager = SecurityManager::new();
    
    // Test with different risk levels
    for risk in [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High, RiskLevel::Critical] {
        let session = ConfirmationSession {
            id: Uuid::new_v4(),
            command: "test command".to_string(),
            risk_level: risk,
            required_method: TactileMethod::Hold {
                duration_ms: 2000,
                target: "space".to_string(),
            },
            progress: 0.5,
            start_time: std::time::Instant::now(),
            user: "user".to_string(),
            sector_id: Uuid::new_v4(),
        };
        
        let html = manager.render_confirmation(&session);
        
        let expected_class = match risk {
            RiskLevel::Low => "risk-low",
            RiskLevel::Medium => "risk-medium",
            RiskLevel::High => "risk-high",
            RiskLevel::Critical => "risk-critical",
        };
        
        assert!(html.contains(expected_class));
    }
}

#[test]
fn test_render_confirmation_methods() {
    let manager = SecurityManager::new();
    
    // Hold method
    let session = ConfirmationSession {
        id: Uuid::new_v4(),
        command: "cmd".to_string(),
        risk_level: RiskLevel::High,
        required_method: TactileMethod::Hold {
            duration_ms: 3000,
            target: "space".to_string(),
        },
        progress: 0.5,
        start_time: std::time::Instant::now(),
        user: "user".to_string(),
        sector_id: Uuid::new_v4(),
    };
    let html = manager.render_confirmation(&session);
    assert!(html.contains("HOLD"));
    assert!(html.contains("space"));
    assert!(html.contains("50%"));
    
    // Slider method
    let session = ConfirmationSession {
        id: Uuid::new_v4(),
        command: "cmd".to_string(),
        risk_level: RiskLevel::High,
        required_method: TactileMethod::Slider {
            distance_percent: 0.8,
            direction: SlideDirection::Right,
        },
        progress: 0.5,
        start_time: std::time::Instant::now(),
        user: "user".to_string(),
        sector_id: Uuid::new_v4(),
    };
    let html = manager.render_confirmation(&session);
    assert!(html.contains("SLIDE"));
    assert!(html.contains("80%"));
    
    // Voice method
    let session = ConfirmationSession {
        id: Uuid::new_v4(),
        command: "cmd".to_string(),
        risk_level: RiskLevel::High,
        required_method: TactileMethod::Voice {
            phrase: "I confirm".to_string(),
            confidence_threshold: 0.9,
        },
        progress: 0.0,
        start_time: std::time::Instant::now(),
        user: "user".to_string(),
        sector_id: Uuid::new_v4(),
    };
    let html = manager.render_confirmation(&session);
    assert!(html.contains("VOICE"));
    assert!(html.contains("I confirm"));
    
    // MultiButton method
    let session = ConfirmationSession {
        id: Uuid::new_v4(),
        command: "cmd".to_string(),
        risk_level: RiskLevel::Critical,
        required_method: TactileMethod::MultiButton {
            button_count: 3,
            buttons: vec!["ctrl".to_string(), "alt".to_string(), "delete".to_string()],
        },
        progress: 1.0,
        start_time: std::time::Instant::now(),
        user: "user".to_string(),
        sector_id: Uuid::new_v4(),
    };
    let html = manager.render_confirmation(&session);
    assert!(html.contains("MULTI-BUTTON"));
    assert!(html.contains("ctrl"));
    assert!(html.contains("alt"));
    assert!(html.contains("delete"));
}

#[test]
fn test_render_security_dashboard() {
    let mut manager = SecurityManager::new();
    let sector_id = Uuid::new_v4();
    
    // Add some events
    manager.start_confirmation("rm -rf /test", "user1", sector_id);
    manager.start_confirmation("dd if=/dev/zero of=/dev/sdb", "user2", Uuid::new_v4());
    
    let html = manager.render_security_dashboard();
    
    assert!(html.contains("Security Dashboard"));
    assert!(html.contains("Total Events"));
    assert!(html.contains("Active Sessions"));
    assert!(html.contains("Patterns"));
    assert!(html.contains("user1"));
    assert!(html.contains("user2"));
    assert!(html.contains("Recent Events"));
}

#[test]
fn test_security_config_customization() {
    let mut config = SecurityConfig::default();
    
    config.enable_detection = false;
    config.enable_audit_log = false;
    config.confirmation_timeout_secs = 60;
    config.confirm_all_destructive = false;
    config.max_audit_entries = 500;
    
    let manager = SecurityManager::with_config(config);
    
    assert!(!manager.config.enable_detection);
    assert!(!manager.config.enable_audit_log);
    assert_eq!(manager.config.confirmation_timeout_secs, 60);
    assert!(!manager.config.confirm_all_destructive);
    assert_eq!(manager.config.max_audit_entries, 500);
}

#[test]
fn test_no_detection_when_disabled() {
    let mut config = SecurityConfig::default();
    config.enable_detection = false;
    let manager = SecurityManager::with_config(config);
    
    // Should not detect even critical commands
    assert!(manager.check_command("rm -rf /").is_none());
    assert!(manager.check_command("dd if=/dev/zero of=/dev/sda").is_none());
}

#[test]
fn test_no_audit_when_disabled() {
    let mut config = SecurityConfig::default();
    config.enable_audit_log = false;
    let mut manager = SecurityManager::with_config(config);
    let sector_id = Uuid::new_v4();
    
    // Start confirmation
    manager.start_confirmation("rm -rf /", "user", sector_id);
    
    // Should not log
    assert!(manager.audit_log.is_empty());
}
