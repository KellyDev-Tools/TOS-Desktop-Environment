use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_dynamic_service_registration() {
    // 1. Initialize Brain environment
    // Use a unique name for each test if possible
    let brain = tos_common::brain::Brain::new().unwrap();
    let ipc = brain.ipc.clone();
    
    // 2. Start UDS Server (Discovery Gate)
    let server = tos_common::platform::RemoteServer::new(ipc.clone());
    tokio::spawn(async move {
        // Use a test-specific port to avoid collisions
        let _ = server.run(27000).await;
    });

    // Wait for UDS to be ready
    sleep(Duration::from_millis(1500)).await;

    // 3. Simulate a daemon registering (e.g. settingsd)
    let daemon_name = "test-daemon";
    let daemon_port = 12345;
    
    let result = tos_common::register_with_brain(daemon_name, daemon_port).await;
    if let Err(e) = &result {
        eprintln!("DEBUG: Registration error: {:?}", e);
    }
    assert!(result.is_ok(), "Registration failed: {:?}", result.err());

    // 4. Verify registry in Brain
    let registry = brain.services.registry.lock().unwrap();
    let entry = registry.get(daemon_name).expect("Daemon not found in registry");
    assert_eq!(entry.port, daemon_port);
    assert!(entry.alive);
}
