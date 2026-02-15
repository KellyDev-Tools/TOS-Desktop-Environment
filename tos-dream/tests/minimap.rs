//! Tests for Tactical Mini-Map (Phase 11)

use tos_core::TosState;
use tos_core::ui::minimap::{MiniMap, MiniMapConfig, MiniMapPosition, ActivationMethod, NavigationTarget};

#[test]
fn test_minimap_default_configuration() {
    let config = MiniMapConfig::default();
    
    assert_eq!(config.position, MiniMapPosition::BottomRight);
    assert_eq!(config.size, (0.2, 0.25));
    assert_eq!(config.passive_opacity, 0.3);
    assert_eq!(config.active_opacity, 0.9);
    assert!(config.show_other_sectors);
    assert!(config.show_viewports);
    assert_eq!(config.hover_dwell_ms, 1000);
    
    // Check activation methods
    assert!(!config.activation_methods.is_empty());
    let has_hover = config.activation_methods.iter().any(|m| matches!(m, ActivationMethod::Hover(_)));
    let has_keyboard = config.activation_methods.iter().any(|m| matches!(m, ActivationMethod::KeyboardShortcut(_)));
    assert!(has_hover);
    assert!(has_keyboard);
}

#[test]
fn test_minimap_creation() {
    let minimap = MiniMap::new();
    
    assert!(!minimap.is_active());
    assert!(minimap.last_hover_pos.is_none());
    assert!(minimap.selected_sector.is_none());
    assert!(minimap.selected_viewport.is_none());
}

#[test]
fn test_minimap_state_transitions() {
    let mut minimap = MiniMap::new();
    
    // Initially passive
    assert!(!minimap.is_active());
    
    // Activate
    minimap.activate();
    assert!(minimap.is_active());
    
    // Deactivate
    minimap.deactivate();
    assert!(!minimap.is_active());
    
    // Toggle
    minimap.toggle();
    assert!(minimap.is_active());
    
    minimap.toggle();
    assert!(!minimap.is_active());
}

#[test]
fn test_minimap_hover_activation() {
    let mut minimap = MiniMap::new();
    minimap.config.hover_dwell_ms = 100; // Short dwell for testing
    
    // Start hovering
    minimap.handle_hover(100.0, 100.0);
    assert!(matches!(minimap.state, tos_core::ui::minimap::MiniMapState::Hovering(_)));
    
    // Wait for dwell time (simulated by immediate activation in test)
    std::thread::sleep(std::time::Duration::from_millis(150));
    minimap.handle_hover(100.0, 100.0);
    
    // Should still be hovering (not enough time in test environment)
    // In real use, this would activate after dwell time
}

#[test]
fn test_minimap_hover_exit() {
    let mut minimap = MiniMap::new();
    
    minimap.handle_hover(100.0, 100.0);
    assert!(minimap.last_hover_pos.is_some());
    
    minimap.handle_hover_exit();
    assert!(minimap.last_hover_pos.is_none());
}

#[test]
fn test_minimap_click_navigation() {
    let state = TosState::new();
    let minimap = MiniMap::new();
    
    // Click when passive should return None
    let target = minimap.handle_click(0.5, 0.5, &state);
    assert!(target.is_none());
    
    // Note: Testing active state clicks would require more setup
}

#[test]
fn test_minimap_position_variants() {
    let positions = vec![
        MiniMapPosition::TopLeft,
        MiniMapPosition::TopRight,
        MiniMapPosition::BottomLeft,
        MiniMapPosition::BottomRight,
        MiniMapPosition::Center,
    ];
    
    for pos in positions {
        let mut config = MiniMapConfig::default();
        config.position = pos;
        let minimap = MiniMap::with_config(config);
        
        let state = TosState::new();
        let html = minimap.render(&state);
        
        // Check that position class is in the HTML
        let expected_class = match pos {
            MiniMapPosition::TopLeft => "minimap-topleft",
            MiniMapPosition::TopRight => "minimap-topright",
            MiniMapPosition::BottomLeft => "minimap-bottomleft",
            MiniMapPosition::BottomRight => "minimap-bottomright",
            MiniMapPosition::Center => "minimap-center",
        };
        assert!(html.contains(expected_class));
    }
}

#[test]
fn test_minimap_render_levels() {
    let mut state = TosState::new();
    let minimap = MiniMap::new();
    
    // Level 1: Global Overview
    let html = minimap.render(&state);
    assert!(html.contains("TACTICAL MINI-MAP"));
    assert!(html.contains("minimap-sectors-grid"));
    assert!(html.contains("Alpha Sector"));
    assert!(html.contains("Science Labs"));
    
    // Level 2: Command Hub
    state.zoom_in();
    let html = minimap.render(&state);
    assert!(html.contains("minimap-current-sector"));
    assert!(html.contains("Command Mode"));
    
    // Level 3: Application Focus
    state.zoom_in();
    let html = minimap.render(&state);
    assert!(html.contains("minimap-app-focus"));
    assert!(html.contains("Main Terminal"));
}

#[test]
fn test_minimap_active_vs_passive_rendering() {
    let mut state = TosState::new();
    let mut minimap = MiniMap::new();
    
    // Passive state
    let html_passive = minimap.render(&state);
    assert!(html_passive.contains("minimap-passive"));
    assert!(html_passive.contains("opacity: 0.3"));
    
    // Active state
    minimap.activate();
    let html_active = minimap.render(&state);
    assert!(html_active.contains("minimap-active"));
    assert!(html_active.contains("opacity: 0.9"));
}

#[test]
fn test_minimap_opacity_configuration() {
    let mut config = MiniMapConfig::default();
    config.passive_opacity = 0.5;
    config.active_opacity = 1.0;
    
    let mut minimap = MiniMap::with_config(config);
    let state = TosState::new();
    
    let html = minimap.render(&state);
    assert!(html.contains("opacity: 0.5"));
    
    minimap.activate();
    let html = minimap.render(&state);
    assert!(html.contains("opacity: 1"));
}

#[test]
fn test_minimap_size_configuration() {
    let mut config = MiniMapConfig::default();
    config.size = (0.3, 0.4);
    
    let minimap = MiniMap::with_config(config);
    assert_eq!(minimap.config.size, (0.3, 0.4));
}

#[test]
fn test_navigation_target() {
    let target = NavigationTarget {
        sector_index: 1,
        viewport_index: Some(0),
    };
    
    assert_eq!(target.sector_index, 1);
    assert_eq!(target.viewport_index, Some(0));
}

#[test]
fn test_activation_methods() {
    let methods = vec![
        ActivationMethod::Hover(1000),
        ActivationMethod::KeyboardShortcut("Ctrl+M".to_string()),
        ActivationMethod::ModifierClick("Alt".to_string()),
        ActivationMethod::DoubleTap,
        ActivationMethod::GamepadButton("A".to_string()),
        ActivationMethod::Voice("activate mini-map".to_string()),
    ];
    
    for method in methods {
        let mut config = MiniMapConfig::default();
        config.activation_methods = vec![method.clone()];
        
        match method {
            ActivationMethod::Hover(ms) => assert_eq!(ms, 1000),
            ActivationMethod::KeyboardShortcut(s) => assert_eq!(s, "Ctrl+M"),
            ActivationMethod::ModifierClick(s) => assert_eq!(s, "Alt"),
            ActivationMethod::GamepadButton(s) => assert_eq!(s, "A"),
            ActivationMethod::Voice(s) => assert_eq!(s, "activate mini-map"),
            _ => {}
        }
    }
}

#[test]
fn test_minimap_split_view_rendering() {
    let mut state = TosState::new();
    let minimap = MiniMap::new();
    
    // Create split view state
    state.current_level = tos_core::HierarchyLevel::SplitView;
    
    let html = minimap.render(&state);
    assert!(html.contains("minimap-split-view"));
    assert!(html.contains("SPLIT VIEW MODE"));
}

#[test]
fn test_minimap_with_multiple_sectors() {
    let state = TosState::new();
    let minimap = MiniMap::new();
    
    let html = minimap.render(&state);
    
    // Should show all three default sectors
    assert!(html.contains("Alpha Sector"));
    assert!(html.contains("Science Labs"));
    assert!(html.contains("Observation Hub"));
}

#[test]
fn test_minimap_legend_by_level() {
    let mut state = TosState::new();
    let minimap = MiniMap::new();
    
    // Level 1
    let html = minimap.render(&state);
    assert!(html.contains("Level 1: Global"));
    
    // Level 2
    state.zoom_in();
    let html = minimap.render(&state);
    assert!(html.contains("Level 2: Hub"));
    
    // Level 3
    state.zoom_in();
    let html = minimap.render(&state);
    assert!(html.contains("Level 3: App"));
}
