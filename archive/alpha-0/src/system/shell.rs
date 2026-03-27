use std::sync::mpsc::Sender;
use crate::UiCommand;

// Based on "Refining Shell Integration" Context

// OSC Sequence Structure
// \x1b]1337;Key=Value\x07

#[derive(Debug, Clone, PartialEq)]
pub enum ShellCommand {
    Zoom(u8),
    ChangeDir(String),
    SetLayout(String),
}

pub struct ShellIntegrator {
    ui_tx: Option<Sender<crate::UiCommand>>,
}

impl ShellIntegrator {
    pub fn new(ui_tx: Option<Sender<crate::UiCommand>>) -> Self {
        Self { ui_tx }
    }

    // Parse a chunk of stdout from the shell PTY
    pub fn parse_stdout(&self, data: &str) -> Vec<ShellCommand> {
        let mut results = Vec::new();
        let mut current = data;

        while let Some(start) = current.find("\x1b]1337;") {
            if let Some(end) = current[start..].find("\x07") {
                let osc_content = &current[start + 7 .. start + end];
                if let Some(cmd) = self.handle_osc(osc_content) {
                    results.push(cmd);
                }
                current = &current[start + end + 1..];
            } else {
                break;
            }
        }
        results
    }

    fn handle_osc(&self, content: &str) -> Option<ShellCommand> {
        println!("[Shell] Received OSC Command: {}", content);
        
        let parts: Vec<&str> = content.split('=').collect();
        if parts.len() != 2 { return None; }

        let key = parts[0];
        let value = parts[1];

        match key {
            "CurrentDir" => {
                println!("[Shell] Directory Changed to: {}", value);
                Some(ShellCommand::ChangeDir(value.to_string()))
            }
            "ZoomLevel" => {
                if let Ok(level) = value.parse::<u8>() {
                    println!("[Shell] Requesting Zoom Level: {}", level);
                    if let Some(tx) = &self.ui_tx {
                        let _ = tx.send(UiCommand::ZoomLevel(level));
                    }
                    Some(ShellCommand::Zoom(level))
                } else {
                    None
                }
            }
            "SetLayout" => {
                println!("[Shell] Layout Shift: {}", value);
                Some(ShellCommand::SetLayout(value.to_string()))
            }
            _ => {
                println!("[Shell] Unknown Key: {}", key);
                None
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
    fn test_parse_embedded_osc() {
        let (tx, rx) = channel::<UiCommand>();
        let shell = ShellIntegrator::new(Some(tx));

        shell.parse_stdout("Setting directory...\r\n\x1b]1337;CurrentDir=/tmp\x07Done.");
        // Should not panic, should acknowledge the dir
    }

    #[test]
    fn test_parse_multiple_osc() {
        let (tx, rx) = channel::<UiCommand>();
        let shell = ShellIntegrator::new(Some(tx));

        // Current implementation only finds the FIRST one in a chunk.
        // Let's verify that behavior or improve it.
        // For now, verify it handles the first one.
        shell.parse_stdout("\x1b]1337;ZoomLevel=2\x07\x1b]1337;ZoomLevel=3\x07");
        
        let cmd = rx.try_recv().unwrap();
        match cmd {
            UiCommand::ZoomLevel(lvl) => assert_eq!(lvl, 2),
            _ => panic!("Expected Level 2"),
        }
    }

    #[test]
    fn test_without_channel() {
        let shell = ShellIntegrator::new(None);
        // Should not panic even without a channel
        shell.parse_stdout("\x1b]1337;ZoomLevel=1\x07");
    }
}
