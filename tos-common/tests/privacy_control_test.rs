use std::sync::{Arc, Mutex};
use tos_common::TosState;
use tos_common::services::session::SessionService;
use tos_common::services::registry::ServiceRegistry;

#[test]
fn test_incognito_blocks_session_save() {
    let registry = Arc::new(Mutex::new(ServiceRegistry::new(7000)));
    let session_svc = SessionService::new(registry);
    
    let mut state = TosState::default();
    
    // 1. Normal mode: save should succeed (or at least not be blocked by privacy)
    // We expect success because default paths are usually writable in tests
    assert!(session_svc.save_live(&state).is_ok());
    
    // 2. Enable Incognito
    state.settings.global.insert("tos.privacy.incognito".to_string(), "true".to_string());
    
    // 3. Incognito mode: save_live should return Ok(()) but do nothing (silent skip)
    assert!(session_svc.save_live(&state).is_ok());
    
    // 4. Named save should be explicitly blocked
    let res = session_svc.save("sector-1", "test-session", &state);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("SESSION_BLOCKED"));
}
