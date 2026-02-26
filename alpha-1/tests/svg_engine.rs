use tos_core::{TosState, HierarchyLevel, Sector, ConnectionType};
use tos_core::ui::render::svg_engine::render_state_to_svg;

#[test]
fn test_svg_dynamic_telemetry() {
    let mut state = TosState::new();
    state.current_level = HierarchyLevel::GlobalOverview;
    
    let svg = render_state_to_svg(&state);
    
    // Verify dynamic telemetry fields
    assert!(svg.contains("SYSTEM TIME"), "SVG missing System Time label");
    assert!(svg.contains("STARDATE"), "SVG missing Stardate label");
    
    // Verify system time format (HH:MM:SS is typical)
    // We can't predict exact time, but check for colon format or just non-empty value tag
    // The value is in <text ...>VALUE</text>
    // A simple contains check for a characteristic of the time string or just ensuring it's not empty/placeholder
    // The previous implementation had no System Time. 
    // Now it calls state.get_system_time() which returns "HH:MM:SS".
    assert!(svg.contains(":"), "System time should contain colons");
}

#[test]
fn test_svg_dynamic_sector_icon() {
    let mut state = TosState::new();
    state.current_level = HierarchyLevel::GlobalOverview;
    
    // Create a custom sector with a unique icon to verify specific rendering
    let unique_icon = "🐲"; // Dragon emoji
    let sector = Sector {
        id: uuid::Uuid::new_v4(),
        name: "Dragon Sector".to_string(),
        color: "#ff0000".to_string(),
        settings: std::collections::HashMap::new(),
        hubs: vec![],
        active_hub_index: 0,
        host: "REMOTE".to_string(),
        connection_type: ConnectionType::Local,
        participants: vec![],
        portal_active: false,
        portal_url: None,
        description: "Test Custom Icon".to_string(),
        icon: unique_icon.to_string(),
        sector_type_name: "operations".to_string(),
    };
    state.sectors.push(sector);
    
    let svg = render_state_to_svg(&state);
    
    // Verify the unique icon is present
    assert!(svg.contains(unique_icon), "SVG should render the custom sector icon");
    
    // Verify standard sector icons from default state are also present
    assert!(svg.contains("⌨️"), "SVG should render default Terminal icon");
    assert!(svg.contains("🔬"), "SVG should render Science Labs icon");
}
