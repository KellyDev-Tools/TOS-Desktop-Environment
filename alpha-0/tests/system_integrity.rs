use tos_comp::DesktopEnvironment;
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::system::notifications::Priority;
use tos_comp::compositor::SurfaceRole;

#[test]
fn test_vfs_deep_traversal() {
    let mut env = DesktopEnvironment::new(None);
    
    // Create nested dirs
    env.files.create_dir("a");
    env.files.navigate_to("a");
    env.files.create_dir("b");
    env.files.navigate_to("b");
    env.files.create_dir("c");
    env.files.navigate_to("c");
    
    assert_eq!(env.files.current_path, "/home/user/a/b/c");
    
    // Test navigation up
    env.files.navigate_up();
    assert_eq!(env.files.current_path, "/home/user/a/b");
    
    // Boundary test: navigate to root and up again
    env.files.navigate_up(); // -> /home/user/a
    env.files.navigate_up(); // -> /home/user
    env.files.navigate_up(); // -> /home
    env.files.navigate_up(); // -> /
    assert_eq!(env.files.current_path, "/");
    
    env.files.navigate_up(); // Should stay at /
    assert_eq!(env.files.current_path, "/");
}

#[test]
fn test_notification_flooding() {
    let mut env = DesktopEnvironment::new(None);
    
    // Push 100 notifications
    for i in 0..100 {
        env.notifications.push("Alert", &format!("Msg {}", i), Priority::Normal);
    }
    
    assert_eq!(env.notifications.queue.len(), 100);
    
    // Process some
    for _ in 0..10 {
        env.notifications.process_next();
    }
    assert_eq!(env.notifications.queue.len(), 90);
}

#[test]
fn test_status_bar_logic() {
    let env = DesktopEnvironment::new(None);
    
    // Initial state (Root)
    let html = env.status.render_html(ZoomLevel::Level1Root, None);
    assert!(html.contains("LOC: ROOT"));
    assert!(html.contains("SEC: ---"));
    assert!(!html.contains("SWAP SLOTS"));
    
    // Split view state
    let html_split = env.status.render_html(ZoomLevel::Level3Split, Some(2));
    assert!(html_split.contains("LOC: SPLIT"));
    assert!(html_split.contains("SEC: 2"));
    assert!(html_split.contains("SWAP SLOTS"));
}

#[test]
fn test_viewport_html_updates_on_tick() {
    let mut env = DesktopEnvironment::new(None);
    
    let html1 = env.generate_viewport_html();
    
    // Tick several times to update uptime
    for _ in 0..5 {
        env.tick();
    }
    
    let html2 = env.generate_viewport_html();
    assert_ne!(html1, html2);
    assert!(html2.contains("UPTIME: 00:00:05"));
}

#[test]
fn test_concurrent_audio_events() {
    let mut env = DesktopEnvironment::new(None);
    
    env.audio.play_sound("sound1");
    env.audio.play_sound("sound2");
    env.audio.play_sound("sound3");
    
    assert_eq!(env.audio.queue.len(), 3);
    
    let html = env.generate_viewport_html();
    assert!(html.contains("data-sounds='sound1,sound2,sound3'"));
    // Queue should be cleared after generate_viewport_html
    assert_eq!(env.audio.queue.len(), 0);
}

#[test]
fn test_search_no_results() {
    let mut env = DesktopEnvironment::new(None);
    env.surfaces.create_surface("Work App", SurfaceRole::Toplevel, Some(0));
    
    tos_comp::system::commands::CommandParser::process(&mut env, "find non-existent-app");
    let html = env.generate_viewport_html();
    
    // Should render empty surfaces-grid
    assert!(html.contains("<div class='surfaces-grid'></div>"));
}
