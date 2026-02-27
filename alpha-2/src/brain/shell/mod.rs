use portable_pty::{native_pty_system, CommandBuilder, PtySize, Child};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::common::{TosState, TerminalLine};
use chrono::Local;

pub struct ShellApi {
    _state: Arc<Mutex<TosState>>,
    writer: Box<dyn Write + Send>,
    _child: Box<dyn Child + Send + Sync>,
}

impl ShellApi {
    pub fn new(state: Arc<Mutex<TosState>>) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
        let cmd = CommandBuilder::new(shell);
        let child = pair.slave.spawn_command(cmd)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let state_clone = state.clone();
        thread::spawn(move || {
            Self::read_loop(reader, state_clone);
        });

        Ok(Self {
            _state: state,
            writer,
            _child: child,
        })
    }

    pub fn write(&mut self, data: &str) -> anyhow::Result<()> {
        self.writer.write_all(data.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    fn read_loop(mut reader: Box<dyn Read + Send>, state: Arc<Mutex<TosState>>) {
        let mut osc_parser = OscParser::new();
        let mut line_buffer = String::new();
        let mut buffer = [0u8; 4096];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let data = &buffer[..n];
                    let text = String::from_utf8_lossy(data);
                    line_buffer.push_str(&text);

                    while let Some(pos) = line_buffer.find('\n') {
                        let mut line = line_buffer.drain(..=pos).collect::<String>();
                        line = line.trim_end_matches(['\r', '\n']).to_string();
                        
                        tracing::debug!("PTY LINE: {:?}", line);
                        let (clean_text, priority) = osc_parser.process(&line);
                        tracing::debug!("CLEAN: {:?} PRIO: {}", clean_text, priority);
                        
                        if !clean_text.is_empty() {
                            let mut state_lock = state.lock().unwrap();
                            let idx = state_lock.active_sector_index;
                            if let Some(sector) = state_lock.sectors.get_mut(idx) {
                                let hub_idx = sector.active_hub_index;
                                if let Some(hub) = sector.hubs.get_mut(hub_idx) {
                                    hub.terminal_output.push(TerminalLine {
                                        text: clean_text.to_string(),
                                        priority,
                                        timestamp: Local::now(),
                                    });
                                    
                                    // FIFO enforcement
                                    if hub.terminal_output.len() > hub.buffer_limit {
                                        hub.terminal_output.remove(0);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }
}

pub struct OscParser {
    current_priority: u8,
}

impl OscParser {
    pub fn new() -> Self {
        Self { current_priority: 0 }
    }

    pub fn process(&mut self, input: &str) -> (String, u8) {
        // Very basic OSC 9012 detection: ESC ] 9012 ; <level> BEL
        // Note: Real world needs a robust state machine for escapes
        if let Some(captures) = regex::Regex::new(r"\x1b\]9012;(\d)\x07").unwrap().captures(input) {
            if let Some(level_match) = captures.get(1) {
                self.current_priority = level_match.as_str().parse().unwrap_or(0);
            }
            // Strip the OSC sequence from output
            let clean = regex::Regex::new(r"\x1b\]9012;\d\x07").unwrap().replace_all(input, "");
            (clean.to_string(), self.current_priority)
        } else {
            (input.to_string(), self.current_priority)
        }
    }
}
