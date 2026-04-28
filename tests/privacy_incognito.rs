use tos_common::state::*;
use tos_common::services::ServiceManager;

#[tokio::test]
async fn test_incognito_mode_blocks_persistence() -> anyhow::Result<()> {
    let mut state = TosState::default();
    let config = tos_common::TosConfig::default();
    let services = ServiceManager::with_config(&config);
    
    // 1. Enable incognito mode (§19.4)
    state.settings.global.insert("tos.privacy.incognito".to_string(), "true".to_string());
    
    let sessions_dir = services.session.sessions_dir();
    let live_file = sessions_dir.join("_live.tos-session");
    if live_file.exists() {
        std::fs::remove_file(&live_file)?;
    }

    // 2. Try to save live session
    // Should return Ok(()) but NOT write to disk
    let res = services.session.save_live(&state);
    assert!(res.is_ok());
    assert!(!live_file.exists(), "Live session should NOT be saved in incognito mode");

    // 3. Try to save named session
    // Should return Err (Blocked)
    let res = services.session.save("PRIMARY", "test-incognito", &state);
    assert!(res.is_err(), "Named session save should fail in incognito mode");
    assert!(res.unwrap_err().to_string().contains("SESSION_BLOCKED"));

    Ok(())
}
