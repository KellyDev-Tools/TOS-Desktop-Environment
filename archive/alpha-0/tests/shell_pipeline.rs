// Component Test: Shell -> Navigation -> UI Pipeline  
// Verifies that OSC sequences from the shell propagate through to UiCommands.

use tos_comp::DesktopEnvironment;
use tos_comp::UiCommand;
use tos_comp::navigation::zoom::ZoomLevel;
use std::sync::mpsc::channel;

#[test]
fn test_shell_zoom_reaches_ui_channel() {
    let (tx, rx) = channel::<UiCommand>();
    let env = DesktopEnvironment::new(Some(tx));

    // Simulate shell emitting an OSC zoom command
    env.shell.parse_stdout("\x1b]1337;ZoomLevel=2\x07");

    // The UI channel should have received the command
    let cmd = rx.try_recv().expect("Expected UiCommand from shell");
    match cmd {
        UiCommand::ZoomLevel(level) => assert_eq!(level, 2),
        _ => panic!("Expected ZoomLevel, got {:?}", cmd),
    }
}

#[test]
fn test_shell_multiple_commands_in_sequence() {
    let (tx, rx) = channel::<UiCommand>();
    let env = DesktopEnvironment::new(Some(tx));

    env.shell.parse_stdout("\x1b]1337;ZoomLevel=1\x07");
    env.shell.parse_stdout("\x1b]1337;ZoomLevel=3\x07");

    let cmd1 = rx.try_recv().unwrap();
    let cmd2 = rx.try_recv().unwrap();

    match cmd1 {
        UiCommand::ZoomLevel(1) => {},
        _ => panic!("Expected ZoomLevel(1)"),
    }
    match cmd2 {
        UiCommand::ZoomLevel(3) => {},
        _ => panic!("Expected ZoomLevel(3)"),
    }
}

#[test]
fn test_shell_garbage_does_not_produce_commands() {
    let (tx, rx) = channel::<UiCommand>();
    let env = DesktopEnvironment::new(Some(tx));

    // Normal text, no OSC
    env.shell.parse_stdout("user@host:~$ ls -la\ntotal 42\n");

    assert!(rx.try_recv().is_err(), "No commands should be emitted for normal text");
}

#[test]
fn test_shell_and_navigator_independent() {
    // Shell sends zoom commands to the UI channel, but the Navigator 
    // state is updated separately by the Brain. They should not interfere.
    let (tx, rx) = channel::<UiCommand>();
    let mut env = DesktopEnvironment::new(Some(tx));

    // Shell says "zoom to 3", but navigator is still at Level1
    env.shell.parse_stdout("\x1b]1337;ZoomLevel=3\x07");
    assert_eq!(env.navigator.current_level, ZoomLevel::Level1Root);

    // Navigator zooms independently
    env.navigator.zoom_in(0);
    assert_eq!(env.navigator.current_level, ZoomLevel::Level2Sector);

    // The UI channel still got the shell command
    let cmd = rx.try_recv().unwrap();
    match cmd {
        UiCommand::ZoomLevel(3) => {},
        _ => panic!("Shell command should still be in channel"),
    }
}
