// Component Test: Navigation + Dashboard Coordination
// Verifies that navigation state changes coordinate with dashboard rendering.

use tos_comp::DesktopEnvironment;
use tos_comp::UiCommand;
use tos_comp::ui::dashboard::{ClockWidget, SystemMonitorWidget};
use tos_comp::navigation::zoom::ZoomLevel;
use std::sync::mpsc::channel;

#[test]
fn test_dashboard_renders_at_each_zoom_level() {
    let (tx, rx) = channel::<UiCommand>();
    let mut env = DesktopEnvironment::new(Some(tx.clone()));

    env.dashboard.add_widget(Box::new(ClockWidget));
    env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 42, ram_usage: 77 }));

    // Render at Level 1 (Root)
    let html_l1 = env.dashboard.render_all_html();
    assert!(html_l1.contains("CLOCK"));
    assert!(html_l1.contains("CPU: 42%"));

    // Navigate to Level 2
    env.navigator.zoom_in(0);
    assert_eq!(env.navigator.current_level, ZoomLevel::Level2Sector);

    // Dashboard still renders the same widgets (they are global)
    let html_l2 = env.dashboard.render_all_html();
    assert!(html_l2.contains("CLOCK"));

    // Send dashboard update through the channel
    let _ = tx.send(UiCommand::UpdateDashboard(html_l2.clone()));
    let cmd = rx.try_recv().unwrap();
    match cmd {
        UiCommand::UpdateDashboard(html) => {
            assert!(html.contains("SYSTEM STATUS"));
        },
        _ => panic!("Expected UpdateDashboard"),
    }
}

#[test]
fn test_zoom_level_sent_after_navigation() {
    let (tx, rx) = channel::<UiCommand>();
    let mut env = DesktopEnvironment::new(Some(tx.clone()));

    // Simulate Brain logic: navigate then send zoom command
    env.navigator.zoom_in(0); // -> Level 2
    let level: u8 = match env.navigator.current_level {
        ZoomLevel::Level1Root => 1,
        ZoomLevel::Level2Sector => 2,
        ZoomLevel::Level3Focus => 3,
        ZoomLevel::Level3aPicker => 3,
    };
    let _ = tx.send(UiCommand::ZoomLevel(level));

    let cmd = rx.try_recv().unwrap();
    match cmd {
        UiCommand::ZoomLevel(lvl) => assert_eq!(lvl, 2),
        _ => panic!("Expected ZoomLevel"),
    }
}

#[test]
fn test_full_user_session_simulation() {
    // Simulates a complete user session: startup, navigate, split, back out
    let (tx, rx) = channel::<UiCommand>();
    let mut env = DesktopEnvironment::new(Some(tx.clone()));

    // 1. Startup: Add widgets and render
    env.dashboard.add_widget(Box::new(ClockWidget));
    env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 5, ram_usage: 30 }));
    let _ = tx.send(UiCommand::UpdateDashboard(env.dashboard.render_all_html()));

    // 2. Zoom into Work Sector
    env.navigator.zoom_in(0);
    let _ = tx.send(UiCommand::ZoomLevel(2));

    // 3. Launch Terminal
    env.navigator.zoom_in(1); // Odd index = single window
    let _ = tx.send(UiCommand::ZoomLevel(3));

    // 4. Tick to update dashboard data
    env.tick();
    let _ = tx.send(UiCommand::UpdateDashboard(env.dashboard.render_all_html()));

    // 5. Zoom back out
    env.navigator.zoom_out(); // Single window -> back to sector
    assert_eq!(env.navigator.current_level, ZoomLevel::Level2Sector);
    let _ = tx.send(UiCommand::ZoomLevel(2));

    env.navigator.zoom_out(); // Back to root
    assert_eq!(env.navigator.current_level, ZoomLevel::Level1Root);
    let _ = tx.send(UiCommand::ZoomLevel(1));

    // Verify all 6 commands were sent
    let mut commands = Vec::new();
    while let Ok(cmd) = rx.try_recv() {
        commands.push(cmd);
    }
    assert_eq!(commands.len(), 6);

    // Verify order: Dashboard, Zoom2, Zoom3, Dashboard, Zoom2, Zoom1
    match &commands[0] { UiCommand::UpdateDashboard(_) => {}, _ => panic!("Expected Dashboard") }
    match &commands[1] { UiCommand::ZoomLevel(2) => {}, _ => panic!("Expected Zoom 2") }
    match &commands[2] { UiCommand::ZoomLevel(3) => {}, _ => panic!("Expected Zoom 3") }
    match &commands[3] { UiCommand::UpdateDashboard(html) => { assert!(html.contains("CPU: 10%")); }, _ => panic!("Expected Dashboard with updated CPU") }
    match &commands[4] { UiCommand::ZoomLevel(2) => {}, _ => panic!("Expected Zoom 2") }
    match &commands[5] { UiCommand::ZoomLevel(1) => {}, _ => panic!("Expected Zoom 1") }
}
