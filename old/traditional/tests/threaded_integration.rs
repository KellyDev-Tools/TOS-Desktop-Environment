// Component Test: Multi-threaded Brain/UI Simulation
// Verifies the thread architecture works: Brain sends commands, UI receives them.

use tos_comp::DesktopEnvironment;
use tos_comp::UiCommand;
use tos_comp::ui::dashboard::{ClockWidget, SystemMonitorWidget};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

#[test]
fn test_threaded_brain_sends_to_ui() {
    let (tx, rx) = channel::<UiCommand>();

    // Simulate the Brain thread
    let handle = thread::spawn(move || {
        let mut env = DesktopEnvironment::new(Some(tx.clone()));
        env.dashboard.add_widget(Box::new(ClockWidget));
        env.dashboard.add_widget(Box::new(SystemMonitorWidget { cpu_usage: 50, ram_usage: 80 }));

        // Send initial dashboard
        let _ = tx.send(UiCommand::UpdateDashboard(env.dashboard.render_all_html()));

        // Simulate navigation
        env.navigator.zoom_in(0);
        let _ = tx.send(UiCommand::ZoomLevel(2));

        env.navigator.zoom_in(1);
        let _ = tx.send(UiCommand::ZoomLevel(3));
    });

    // Simulate the UI thread receiving
    handle.join().unwrap();

    let mut received = Vec::new();
    while let Ok(cmd) = rx.try_recv() {
        received.push(cmd);
    }

    assert_eq!(received.len(), 3);
    match &received[0] {
        UiCommand::UpdateDashboard(html) => {
            assert!(html.contains("CLOCK"));
            assert!(html.contains("CPU: 50%"));
        },
        _ => panic!("First command should be UpdateDashboard"),
    }
    match &received[1] {
        UiCommand::ZoomLevel(2) => {},
        _ => panic!("Second command should be ZoomLevel(2)"),
    }
    match &received[2] {
        UiCommand::ZoomLevel(3) => {},
        _ => panic!("Third command should be ZoomLevel(3)"),
    }
}

#[test]
fn test_threaded_shell_integration() {
    let (tx, rx) = channel::<UiCommand>();

    let handle = thread::spawn(move || {
        let env = DesktopEnvironment::new(Some(tx));

        // Simulate shell output arriving over time
        env.shell.parse_stdout("\x1b]1337;ZoomLevel=2\x07");
        thread::sleep(Duration::from_millis(10));
        env.shell.parse_stdout("\x1b]1337;ZoomLevel=3\x07");
        thread::sleep(Duration::from_millis(10));
        env.shell.parse_stdout("\x1b]1337;ZoomLevel=1\x07");
    });

    handle.join().unwrap();

    let mut levels = Vec::new();
    while let Ok(cmd) = rx.try_recv() {
        if let UiCommand::ZoomLevel(lvl) = cmd {
            levels.push(lvl);
        }
    }

    assert_eq!(levels, vec![2, 3, 1]);
}

#[test]
fn test_channel_disconnect_is_graceful() {
    let (tx, rx) = channel::<UiCommand>();

    // Drop the receiver immediately
    drop(rx);

    // Brain should not panic when sending to a dropped channel
    let _env = DesktopEnvironment::new(Some(tx.clone()));
    let result = tx.send(UiCommand::ZoomLevel(1));
    
    // Should return Err (disconnected), not panic
    assert!(result.is_err());
}
