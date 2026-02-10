// Component Test: Full Desktop Environment Lifecycle
// Tests the DesktopEnvironment struct with all subsystems wired together.

use tos_comp::DesktopEnvironment;
use tos_comp::UiCommand;
use tos_comp::ui::dashboard::{ClockWidget, SystemMonitorWidget};
use tos_comp::navigation::zoom::ZoomLevel;
use tos_comp::system::notifications::Priority;
use std::sync::mpsc::channel;

#[test]
fn test_environment_initializes_cleanly() {
    let (tx, _rx) = channel::<UiCommand>();
    let env = DesktopEnvironment::new(Some(tx));

    assert_eq!(env.navigator.current_level, ZoomLevel::Level1Root);
    assert!(env.notifications.queue.is_empty());
    assert_eq!(env.files.current_path, "/home/user");
    assert!(env.dashboard.widgets.is_empty()); // No widgets added yet
}

#[test]
fn test_environment_without_channel() {
    // Should work headless (no UI connected)
    let env = DesktopEnvironment::new(None);
    assert_eq!(env.navigator.current_level, ZoomLevel::Level1Root);
}

#[test]
fn test_tick_updates_widgets() {
    let env_result = {
        let mut env = DesktopEnvironment::new(None);
        env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 10, ram_usage: 50 }));
        env.tick();
        // After tick, cpu should be 15 (10 + 5)
        env.dashboard.render_all_html()
    };
    assert!(env_result.contains("CPU: 15%"));
}

#[test]
fn test_multiple_ticks_accumulate() {
    let mut env = DesktopEnvironment::new(None);
    env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 90, ram_usage: 50 }));

    env.tick(); // 95
    env.tick(); // 0 (wraps at 100)
    env.tick(); // 5

    let html = env.dashboard.render_all_html();
    assert!(html.contains("CPU: 5%"));
}
