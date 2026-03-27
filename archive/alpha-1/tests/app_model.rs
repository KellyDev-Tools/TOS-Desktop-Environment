use tos_core::*;

#[test]
fn test_app_model_fields_integration() {
    let state = TosState::new();
    
    // 1. Verify Default Application (Terminal) in Alpha Sector
    let alpha = &state.sectors[0];
    let terminal = &alpha.hubs[0].applications[0];
    
    // Check new fields on default app
    assert!(terminal.thumbnail.is_none(), "Terminal shouldn't have thumbnail default");
    assert_eq!(terminal.decoration_policy, DecorationPolicy::Native, "Terminal should be Native decoration");
    assert!(terminal.bezel_actions.is_empty(), "Terminal has no bezel actions");
    
    // 2. Verify Science App (Spectrometer) has Overlay policy and Bezel Actions
    // Science Labs is the second sector typically (index 1)
    let science_sector = &state.sectors[1]; 
    
    // Ensure we are looking at the right sector just in case order changes
    if science_sector.name != "Science Labs" {
        // Fallback search
        let _ = state.sectors.iter().find(|s| s.name == "Science Labs").expect("Science Labs sector not found");
        // But for this test, we assume standard order from TosState::new()
    }
    
    let spectrometer = &science_sector.hubs[0].applications[0];
    
    assert_eq!(spectrometer.title, "Spectrometer");
    assert_eq!(spectrometer.decoration_policy, DecorationPolicy::Overlay, "Spectrometer should use Overlay");
    assert_eq!(spectrometer.bezel_actions.len(), 2, "Spectrometer should have 2 bezel actions");
    assert_eq!(spectrometer.bezel_actions[0].label, "SCAN");
    assert_eq!(spectrometer.bezel_actions[0].command, "scan_start");
    
    // 3. Verify Telemetry App (DataFeed) has Suppress policy
    let telemetry = &science_sector.hubs[0].applications[1];
    assert_eq!(telemetry.title, "DataFeed");
    assert_eq!(telemetry.decoration_policy, DecorationPolicy::Suppress, "DataFeed should Suppress decoration");
}

#[test]
fn test_manual_app_creation_component() {
    let action = BezelAction {
        label: "FIRE".to_string(),
        command: "torpedo_fire".to_string(),
    };
    
    let app = Application {
        id: uuid::Uuid::new_v4(),
        title: "Tactical".to_string(),
        app_class: "Tactical".to_string(),
        is_minimized: false,
        pid: None,
        icon: None,
        is_dummy: true,
        settings: std::collections::HashMap::new(),
        thumbnail: Some(vec![0xFF, 0x00, 0xFF]),
        decoration_policy: DecorationPolicy::Overlay,
        bezel_actions: vec![action],
    };
    
    assert_eq!(app.decoration_policy, DecorationPolicy::Overlay);
    assert!(app.thumbnail.is_some());
    assert_eq!(app.thumbnail.unwrap(), vec![0xFF, 0x00, 0xFF]);
    assert_eq!(app.bezel_actions[0].label, "FIRE");
}
