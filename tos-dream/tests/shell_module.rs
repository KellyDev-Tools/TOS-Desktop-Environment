use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tos_core::TosState;
use tos_core::system::shell::ShellProvider;

#[test]
fn test_shell_module_spawning_logic() {
    let state = TosState::new();
    
    // 1. Verify Fish provider is registered by default
    let fish = state.shell_registry.get("fish");
    assert!(fish.is_some());
    
    let fish = fish.unwrap();
    assert_eq!(fish.name(), "fish");
    assert_eq!(fish.default_path(), "/usr/bin/fish");
    
    // 2. Verify integration script generation is functional
    let script = fish.get_integration_script();
    assert!(script.contains("TOS Shell Integration for Fish"));
}

#[test]
fn test_shell_api_state_transition() {
    // Test that process_shell_output correctly handles state mutations
    // while managing the mem::take of the shell_api
    let mut state = TosState::new();
    
    // Simulate an OSC sequence for CWD update
    let osc_cwd = "\x1b]9003;/tmp/tos_test\x07";
    let clean = state.process_shell_output(osc_cwd);
    
    // Output should be empty (it was a silent OSC command)
    assert_eq!(clean, "");
    
    // Further check: verify state was updated via the OSC handler
    // In our current handle_sequence, we just trace for CWD, 
    // but we can test suggestions/directory transitions easily.
    
    let osc_suggestions = "\x1b]9000;ls;ls -la;List;builtin\x07";
    state.process_shell_output(osc_suggestions);
    
    // If the integration is working, the internal shell_api state 
    // (stashed back into TosState) should be updated or the 
    // hub state should be updated.
}

#[test]
fn test_multiple_shell_registry() {
    let mut state = TosState::new();
    
    #[derive(Debug)]
    struct MockShell;
    impl ShellProvider for MockShell {
        fn name(&self) -> &str { "mock" }
        fn default_path(&self) -> &str { "/bin/mock" }
        fn get_integration_script(&self) -> String { "echo mock".to_string() }
        fn spawn(&self, _cwd: &str) -> Option<tos_core::system::pty::PtyHandle> { None }
    }
    
    state.shell_registry.register(Box::new(MockShell));
    assert!(state.shell_registry.get("mock").is_some());
    assert_eq!(state.shell_registry.get("mock").unwrap().name(), "mock");
}
