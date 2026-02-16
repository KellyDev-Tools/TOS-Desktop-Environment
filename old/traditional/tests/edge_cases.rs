use tos_comp::compositor::{SurfaceManager, SurfaceRole};
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::DesktopEnvironment;
use tos_comp::system::commands::CommandParser;

#[test]
fn test_app_class_generation() {
    let mut mgr = SurfaceManager::new();
    let id1 = mgr.create_surface("Firefox Browser", SurfaceRole::Toplevel, Some(0));
    let id2 = mgr.create_surface("Terminal - zsh", SurfaceRole::Toplevel, Some(0));
    let id3 = mgr.create_surface("Files", SurfaceRole::Toplevel, Some(0));
    let id4 = mgr.create_surface("", SurfaceRole::Toplevel, Some(0)); // Empty

    assert_eq!(mgr.get_surface(id1).unwrap().app_class, "Firefox");
    assert_eq!(mgr.get_surface(id2).unwrap().app_class, "Terminal");
    assert_eq!(mgr.get_surface(id3).unwrap().app_class, "Files");
    assert_eq!(mgr.get_surface(id4).unwrap().app_class, "App");
}

#[test]
fn test_history_limit() {
    let mut mgr = SurfaceManager::new();
    let id = mgr.create_surface("Test Node", SurfaceRole::Toplevel, Some(0));
    
    // Initial event + 15 more
    for i in 0..15 {
        mgr.add_event(id, &format!("Event {}", i));
    }
    
    let surface = mgr.get_surface(id).unwrap();
    assert_eq!(surface.history.len(), 10);
    // Should contain the last ones
    assert!(surface.history.contains(&"Event 14".to_string()));
    assert!(!surface.history.contains(&"Surface created: Test Node".to_string()));
}

#[test]
fn test_invalid_commands() {
    let mut env = DesktopEnvironment::new(None);
    
    let res1 = CommandParser::process(&mut env, "invalid_cmd");
    assert!(res1.contains("Unknown command"));
    
    let res2 = CommandParser::process(&mut env, "kill abc");
    assert!(res2.contains("Usage"));
    
    let res3 = CommandParser::process(&mut env, "zoom focus"); // Needs number
    assert!(res3.contains("Usage"));
}

#[test]
fn test_audio_disabled_state() {
    let mut env = DesktopEnvironment::new(None);
    
    // Disable audio
    CommandParser::process(&mut env, "config audio off");
    assert!(!env.audio.enabled);
    
    // Trigger sound
    env.audio.play_sound("test_sound");
    
    // Queue should be empty because audio is disabled
    assert_eq!(env.audio.queue.len(), 0);
}

#[test]
fn test_split_view_invalid_target() {
    let mut env = DesktopEnvironment::new(None);
    let id = env.surfaces.create_surface("S1", SurfaceRole::Toplevel, Some(0));
    
    env.navigator.zoom_in(0); // Sector
    env.navigator.zoom_in(0); // Focus S1
    
    // Attempt split with non-existent ID
    CommandParser::process(&mut env, "split 9999");
    
    // Should still be at Level 3 Focus or Split (depending on impl, current impl allows it but layout will skip it)
    // Actually navigator allows it: self.secondary_app_id = Some(secondary_id); self.current_level = ZoomLevel::Level3Split;
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Split);
    
    // But layout should only have 1 item (S1) because 9999 is missing
    let layouts = tos_comp::compositor::SpatialMapper::get_layout(
        &env.surfaces,
        ZoomLevel::Level3Split,
        Some(0),
        Some(id),
        Some(9999),
        4
    );
    assert_eq!(layouts.len(), 1);
}

#[test]
fn test_remove_surface_in_split_view() {
    let mut env = DesktopEnvironment::new(None);
    let s1 = env.surfaces.create_surface("S1", SurfaceRole::Toplevel, Some(0));
    let s2 = env.surfaces.create_surface("S2", SurfaceRole::Toplevel, Some(0));
    
    env.navigator.zoom_in(0);
    env.navigator.zoom_in(0);
    env.navigator.split_view(s2);
    
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Split);
    
    // Kill s2
    env.surfaces.remove_surface(s2);
    
    // Layout should now only have s1
    let layouts = tos_comp::compositor::SpatialMapper::get_layout(
        &env.surfaces,
        ZoomLevel::Level3Split,
        Some(0),
        Some(s1),
        Some(s2),
        4
    );
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].surface.id, s1);
}
