use tos_alpha2::brain::Brain;
use tos_alpha2::face::{Face, MockFace};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_ai_staged_command_loop() -> anyhow::Result<()> {
    // 1. Initialize Brain & Face
    // Note: Brain::new() now handles ServiceManager initialization internally
    let brain = Brain::new()?;
    let ipc = brain.ipc.clone();
    let state = brain.state.clone();
    
    // Face doesn't need to be mutable for MockFace simulations if it only sends IPC
    let face = Face::new(state.clone(), ipc.clone());
    let mock = MockFace(face);

    // 2. Switch to AI Mode
    ipc.handle_request("set_mode:ai");
    
    // 3. Submit AI Query
    mock.simulate_ai_submit("list all files");
    
    // 4. Wait for AI processing (asynchronous spawn in IpcHandler)
    sleep(Duration::from_millis(500)).await;
    
    // 5. Verify staged command in state
    {
        let lock = state.lock().unwrap();
        let hub = &lock.sectors[0].hubs[0];
        assert!(hub.staged_command.is_some(), "AI should have staged a command");
        assert_eq!(hub.staged_command.as_ref().unwrap(), "ls -la");
        println!("AI Staged Command verified: {}", hub.staged_command.as_ref().unwrap());
        println!("AI Rationale: {:?}", hub.ai_explanation);
    }

    // 6. Accept suggestion via MockFace
    mock.simulate_ai_accept();
    
    // 7. Verify prompt update and cleanup
    {
        let lock = state.lock().unwrap();
        let hub = &lock.sectors[0].hubs[0];
        assert_eq!(hub.prompt, "ls -la", "Prompt should be updated with staged command");
        assert!(hub.staged_command.is_none(), "Staged command should be cleared after acceptance");
        assert!(hub.ai_explanation.is_none(), "AI explanation should be cleared after acceptance");
        println!("Promotion verified. New prompt: {}", hub.prompt);
    }

    Ok(())
}

