use std::io::{self, Read, Write};
use std::sync::mpsc::Sender;
use crate::UiCommand;

// Based on "Refining Shell Integration" Context

// OSC Sequence Structure
// \x1b]1337;Key=Value\x07

pub struct ShellIntegrator {
    // Channel to send UI updates based on shell events
    ui_tx: Option<Sender<UiCommand>>,
}

impl ShellIntegrator {
    pub fn new(ui_tx: Option<Sender<UiCommand>>) -> Self {
        Self { ui_tx }
    }

    // Parse a chunk of stdout from the shell PTY
    pub fn parse_stdout(&self, data: &str) {
        // Simple state machine for OSC sequences
        // In reality, we'd use a virtual terminal emulator like 'alacritty_terminal' or 'vte'
        // This is a naive implementation for the prototype.

        if let Some(start) = data.find("\x1b]1337;") {
            if let Some(end) = data[start..].find("\x07") {
                let osc_content = &data[start + 7 .. start + end];
                self.handle_osc(osc_content);
            }
        }
    }

    fn handle_osc(&self, content: &str) {
        println!("[Shell] Received OSC Command: {}", content);
        
        let parts: Vec<&str> = content.split('=').collect();
        if parts.len() != 2 { return; }

        let key = parts[0];
        let value = parts[1];

        match key {
            "CurrentDir" => {
                println!("[Shell] Directory Changed to: {}", value);
                // Trigger file browser update?
            }
            "ZoomLevel" => {
                if let Ok(level) = value.parse::<u8>() {
                    println!("[Shell] Requesting Zoom Level: {}", level);
                    if let Some(tx) = &self.ui_tx {
                        let _ = tx.send(UiCommand::ZoomLevel(level));
                    }
                }
            }
            "SetLayout" => {
                println!("[Shell] Layout Shift: {}", value);
                // "Split", "Full", etc.
            }
            _ => {
                println!("[Shell] Unknown Key: {}", key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn test_parse_valid_osc_zoom() {
        let (tx, rx) = channel::<UiCommand>();
        let shell = ShellIntegrator::new(Some(tx));

        shell.parse_stdout("\x1b]1337;ZoomLevel=3\x07");
        
        let cmd = rx.try_recv().unwrap();
        match cmd {
            UiCommand::ZoomLevel(level) => assert_eq!(level, 3),
            _ => panic!("Expected ZoomLevel command"),
        }
    }

    #[test]
    fn test_parse_valid_osc_current_dir() {
        // CurrentDir doesn't send a UiCommand, just prints
        let shell = ShellIntegrator::new(None);
        // Should not panic
        shell.parse_stdout("\x1b]1337;CurrentDir=/home/user/projects\x07");
    }

    #[test]
    fn test_parse_no_osc() {
        let (tx, rx) = channel::<UiCommand>();
        let shell = ShellIntegrator::new(Some(tx));

        shell.parse_stdout("just some normal terminal output");

        // Nothing should have been sent
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_parse_incomplete_osc() {
        let (tx, rx) = channel::<UiCommand>();
        let shell = ShellIntegrator::new(Some(tx));

        // Missing BEL terminator
        shell.parse_stdout("\x1b]1337;ZoomLevel=2");

        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_without_channel() {
        let shell = ShellIntegrator::new(None);
        // Should not panic even without a channel
        shell.parse_stdout("\x1b]1337;ZoomLevel=1\x07");
    }
}
