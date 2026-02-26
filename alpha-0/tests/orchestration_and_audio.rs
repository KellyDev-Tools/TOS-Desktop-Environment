use tos_comp::DesktopEnvironment;
use tos_comp::system::commands::CommandParser;
use tos_comp::compositor::SurfaceRole;

#[test]
fn test_task_orchestration_movement() {
    let mut env = DesktopEnvironment::new(None);
    let id = env.surfaces.create_surface("Logic Core", SurfaceRole::Toplevel, Some(0));
    
    // Command: move <id> <sector>
    let resp = CommandParser::process(&mut env, &format!("move {} 2", id));
    assert!(resp.contains("moved to Sector 2"));
    
    let surface = env.surfaces.get_surface(id).unwrap();
    assert_eq!(surface.sector_id, Some(2));
    assert!(surface.history.iter().any(|h| h.contains("Moved to Sector 2")));
}

#[test]
fn test_audio_sequencer_ambient() {
    let mut env = DesktopEnvironment::new(None);
    
    // Initial state: ambient enabled
    assert!(env.audio.ambient_enabled);
    
    // Tick many times to trigger a sound
    for _ in 0..50 {
        env.tick();
    }
    
    let queue = env.audio.consume_queue();
    assert!(queue.contains(&"console_pulse".to_string()));
    
    // Test disabling ambient
    CommandParser::process(&mut env, "config ambient off");
    assert!(!env.audio.ambient_enabled);
    
    // Clear queue and tick again
    env.audio.consume_queue();
    for _ in 0..100 {
        env.tick();
    }
    assert_eq!(env.audio.consume_queue().len(), 0);
}
