use tos_core::system::security::{SecurityManager, SecurityEvent};

#[test]
fn test_deep_inspection_flow() {
    let mut manager = SecurityManager::new();
    
    // Default: not active, not allowed
    assert!(!manager.deep_inspection_active);
    assert!(!manager.config.allow_deep_inspection);
    
    // Try to enable (should fail)
    assert!(!manager.enable_deep_inspection("user"));
    assert!(!manager.deep_inspection_active);
    
    // Allow it
    manager.config.allow_deep_inspection = true;
    
    // Enable (should succeed)
    assert!(manager.enable_deep_inspection("user"));
    assert!(manager.deep_inspection_active);
    
    // Check access
    assert!(manager.check_deep_inspection_access("user", "0x1000"));
    
    // Verify log
    let logs = manager.get_user_audit_log("user");
    assert!(logs.iter().any(|e| matches!(e, SecurityEvent::DeepInspectionEnabled { .. })));
    assert!(logs.iter().any(|e| matches!(e, SecurityEvent::DeepInspectionAccessed { .. })));
    
    // Disable
    manager.disable_deep_inspection("user");
    assert!(!manager.deep_inspection_active);
    
    // Check access (should fail)
    assert!(!manager.check_deep_inspection_access("user", "0x1000"));
    
    // Verify disable log
    let logs = manager.get_user_audit_log("user");
    assert!(logs.iter().any(|e| matches!(e, SecurityEvent::DeepInspectionDisabled { .. })));
}
