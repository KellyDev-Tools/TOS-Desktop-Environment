use tos_core::{TosState};
use tos_core::system::log::{LogType};

#[test]
fn test_log_manager_basic() {
    let mut state = TosState::new_fresh();
    
    // Log an event
    state.log_manager.log(
        LogType::System,
        "Global",
        "System initialized",
        None
    );
    
    // Verify log entry
    assert_eq!(state.log_manager.entries.len(), 1);
    let entry = &state.log_manager.entries[0];
    assert_eq!(entry.event_type, LogType::System);
    assert_eq!(entry.region, "Global");
    assert_eq!(entry.message, "System initialized");
}

#[test]
fn test_log_query() {
    let mut state = TosState::new_fresh();
    
    state.log_manager.log(LogType::Command, "Shell", "ls -la", None);
    state.log_manager.log(LogType::System, "Kernel", "Memory OK", None);
    state.log_manager.log(LogType::Security, "Auth", "Login success", None);
    
    // Query "Shell"
    let results = state.log_manager.query("Shell");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].message, "ls -la");
    
    // Query "OK"
    let results = state.log_manager.query("OK");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].message, "Memory OK");
    
    // Query that matches nothing
    let results = state.log_manager.query("Invalid");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_log_rotation() {
    let mut state = TosState::new_fresh();
    state.log_manager.max_entries = 5;
    
    for i in 0..10 {
        state.log_manager.log(LogType::Telemetry, "Sensor", &format!("Data {}", i), None);
    }
    
    assert_eq!(state.log_manager.entries.len(), 5);
    // Should contain Data 5 to Data 9 (FIFO)
    assert_eq!(state.log_manager.entries.front().unwrap().message, "Data 5");
    assert_eq!(state.log_manager.entries.back().unwrap().message, "Data 9");
}
