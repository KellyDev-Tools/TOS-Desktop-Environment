// Component Test: File Browser + Navigation Integration
// Verifies that file browsing works correctly alongside navigation state.

use tos_comp::DesktopEnvironment;
use tos_comp::navigation::zoom::ZoomLevel;

#[test]
fn test_file_browser_during_navigation() {
    let mut env = DesktopEnvironment::new(None);

    // At root level, file browser starts at /home/user
    assert_eq!(env.files.current_path, "/home/user");
    let entries = env.files.get_current_entries().unwrap();
    assert_eq!(entries.len(), 2); // documents + notes.txt

    // Navigate zoom to sector — file browser is independent
    env.navigator.zoom_in(0);
    assert_eq!(env.navigator.current_level, ZoomLevel::Level2Sector);
    assert_eq!(env.files.current_path, "/home/user"); // Unchanged

    // Navigate files up
    env.files.navigate_up();
    assert_eq!(env.files.current_path, "/home");

    // Zoom state hasn't changed
    assert_eq!(env.navigator.current_level, ZoomLevel::Level2Sector);
}

#[test]
fn test_file_browser_traversal_to_root_and_back() {
    let mut env = DesktopEnvironment::new(None);

    // /home/user -> /home -> / -> / (stays)
    env.files.navigate_up();
    assert_eq!(env.files.current_path, "/home");
    let entries = env.files.get_current_entries().unwrap();
    assert_eq!(entries[0].name, "user");

    env.files.navigate_up();
    assert_eq!(env.files.current_path, "/");
    let entries = env.files.get_current_entries().unwrap();
    assert_eq!(entries.len(), 2); // home + etc

    env.files.navigate_up();
    assert_eq!(env.files.current_path, "/"); // Stays at root
}

#[test]
fn test_notifications_during_session() {
    use tos_comp::system::notifications::Priority;

    let mut env = DesktopEnvironment::new(None);

    // Queue up notifications while navigating
    env.navigator.zoom_in(0);
    env.notifications.push("USB Device", "New keyboard detected", Priority::Normal);
    env.notifications.push("Battery Low", "10% remaining", Priority::Critical);

    env.navigator.zoom_in(1);
    assert_eq!(env.notifications.queue.len(), 2);

    // Process one
    let n = env.notifications.process_next().unwrap();
    assert_eq!(n.title, "USB Device"); // FIFO
    assert_eq!(env.notifications.queue.len(), 1);

    // Navigate does not affect notification queue
    env.intelligent_zoom_out();
    assert_eq!(env.notifications.queue.len(), 1);

    let n = env.notifications.process_next().unwrap();
    assert_eq!(n.title, "Battery Low");
    assert!(env.notifications.process_next().is_none());
}
