use tos_comp::DesktopEnvironment;
use tos_comp::system::commands::CommandParser;
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::compositor::SurfaceRole;

#[test]
fn test_command_parser_spawn_nuances() {
    let mut env = DesktopEnvironment::new(None);
    
    // Spawn with sector
    CommandParser::process(&mut env, "spawn CustomApp 0");
    assert!(env.surfaces.get_surfaces_in_sector(0).iter().any(|s| s.title == "CustomApp"));
    
    // Spawn without sector (defaults to None or 0 depending on impl, currently 0 in logic)
    CommandParser::process(&mut env, "spawn GenericApp");
    assert!(env.surfaces.get_surfaces_in_sector(0).iter().any(|s| s.title == "GenericApp"));
}

#[test]
fn test_command_parser_zoom_nuances() {
    let mut env = DesktopEnvironment::new(None);
    
    // Zoom 1
    CommandParser::process(&mut env, "zoom 1");
    assert_eq!(env.navigator.current_level, ZoomLevel::Level1Root);
    
    // Zoom 2 (into sector 0)
    CommandParser::process(&mut env, "zoom 2");
    assert_eq!(env.navigator.current_level, ZoomLevel::Level2Sector);
    assert_eq!(env.navigator.active_sector_index, Some(0));
    
    // Zoom 3 (into first app)
    env.surfaces.create_surface("App1", SurfaceRole::Toplevel, Some(0));
    CommandParser::process(&mut env, "zoom 3");
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Focus);
}

#[test]
fn test_command_parser_kill_logic() {
    let mut env = DesktopEnvironment::new(None);
    let id = env.surfaces.create_surface("KillMe", SurfaceRole::Toplevel, Some(0));
    
    assert!(env.surfaces.get_surface(id).is_some());
    CommandParser::process(&mut env, &format!("kill {}", id));
    assert!(env.surfaces.get_surface(id).is_none());
}

#[test]
fn test_command_parser_alert_multi_word() {
    let mut env = DesktopEnvironment::new(None);
    CommandParser::process(&mut env, "alert Red Alert: Reactor Leak!");
    
    let n = env.notifications.process_next().unwrap();
    assert_eq!(n.title, "COMM-LINK");
    assert_eq!(n.message, "Red Alert: Reactor Leak!");
}

#[test]
fn test_command_parser_search_clears_on_zoom() {
    let mut env = DesktopEnvironment::new(None);
    CommandParser::process(&mut env, "find Terminal");
    assert!(env.search_query.is_some());
    
    CommandParser::process(&mut env, "zoom 1");
    assert!(env.search_query.is_none());
}

#[test]
fn test_command_parser_settings_toggles() {
    let mut env = DesktopEnvironment::new(None);
    
    CommandParser::process(&mut env, "config audio off");
    assert!(!env.settings.audio_enabled);
    
    CommandParser::process(&mut env, "config audio on");
    assert!(env.settings.audio_enabled);
    
    CommandParser::process(&mut env, "config chirps true");
    assert!(env.settings.chirps_enabled);
    
    CommandParser::process(&mut env, "config chirps false");
    assert!(!env.settings.chirps_enabled);
}

#[test]
fn test_command_parser_split_and_inspect() {
    let mut env = DesktopEnvironment::new(None);
    let _s1 = env.surfaces.create_surface("S1", SurfaceRole::Toplevel, Some(0));
    let s2 = env.surfaces.create_surface("S2", SurfaceRole::Toplevel, Some(0));
    
    env.navigator.zoom_in(0); // Sector 0
    env.navigator.zoom_in(0); // Focus S1
    
    CommandParser::process(&mut env, &format!("split {}", s2));
    assert_eq!(env.navigator.current_level, ZoomLevel::Level3Split);
    
    CommandParser::process(&mut env, "inspect");
    assert_eq!(env.navigator.current_level, ZoomLevel::Level4Detail);
}
