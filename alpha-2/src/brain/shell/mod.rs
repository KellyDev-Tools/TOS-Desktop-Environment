use portable_pty::{native_pty_system, CommandBuilder, PtySize, Child};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::common::{TosState, TerminalLine};
use chrono::Local;

pub struct ShellApi {
    _state: Arc<Mutex<TosState>>,
    _sector_id: uuid::Uuid,
    _hub_id: uuid::Uuid,
    writer: Box<dyn Write + Send>,
    _child: Box<dyn Child + Send + Sync>,
}

impl ShellApi {
    pub fn new(state: Arc<Mutex<TosState>>, sector_id: uuid::Uuid, hub_id: uuid::Uuid) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let verified_shell = if std::path::Path::new(&shell).exists() {
            shell
        } else {
            let fallbacks = ["/bin/bash", "/bin/zsh", "/bin/sh", "/usr/bin/bash", "/usr/bin/sh"];
            fallbacks.iter()
                .find(|&&path| std::path::Path::new(path).exists())
                .map(|&s| s.to_string())
                .unwrap_or_else(|| "sh".to_string())
        };

        tracing::info!("SHELL INIT: Using verified binary: {}", verified_shell);
        let cmd = CommandBuilder::new(verified_shell);
        let child = pair.slave.spawn_command(cmd)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let state_clone = state.clone();
        let sid_clone = sector_id;
        let hid_clone = hub_id;
        let handle = tokio::runtime::Handle::current();
        thread::spawn(move || {
            Self::read_loop(reader, state_clone, sid_clone, hid_clone, handle);
        });

        Ok(Self {
            _state: state,
            _sector_id: sector_id,
            _hub_id: hub_id,
            writer,
            _child: child,
        })
    }

    pub fn write(&mut self, data: &str) -> anyhow::Result<()> {
        self.writer.write_all(data.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    fn read_loop(mut reader: Box<dyn Read + Send>, state: Arc<Mutex<TosState>>, sector_id: uuid::Uuid, hub_id: uuid::Uuid, handle: tokio::runtime::Handle) {
        let mut osc_parser = OscParser::new();
        let mut line_buffer = String::new();
        let mut buffer = [0u8; 4096];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) | Err(_) => {
                    let mut state_lock = state.lock().unwrap();
                    if let Some(sector) = state_lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                        if sector.is_remote {
                            sector.disconnected = true;
                            state_lock.version += 1;
                            let state_clone = state.clone();
                            let sid = sector_id;
                            
                            // §12.1: Graceful ICE Teardown window
                            handle.spawn(async move {
                                tracing::info!("REMOTE: Initiating graceful teardown for sector: {}", sid);
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                
                                let mut lock = state_clone.lock().unwrap();
                                if let Some(pos) = lock.sectors.iter().position(|s| s.id == sid) {
                                    if lock.sectors[pos].disconnected {
                                        tracing::info!("REMOTE: Sector {} teardown complete.", sid);
                                        lock.sectors.remove(pos);
                                        lock.version += 1;
                                    }
                                }
                            });
                        }
                    }
                    break;
                }
                Ok(n) => {
                    let data = &buffer[..n];
                    let text = String::from_utf8_lossy(data);
                    line_buffer.push_str(&text);

                    while let Some(pos) = line_buffer.find('\n') {
                        let mut line = line_buffer.drain(..=pos).collect::<String>();
                        line = line.trim_end_matches(['\r', '\n']).to_string();
                        
                        let (clean_text, events) = osc_parser.process(&line);
                        
                        let mut state_lock = state.lock().unwrap();
                        if let Some(sector) = state_lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                            if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
                                for event in events {
                                    match event {
                                        OscEvent::Priority(p) => osc_parser.current_priority = p,
                                        OscEvent::Cwd(path) => {
                                            hub.current_directory = std::path::PathBuf::from(path);
                                        }
                                        OscEvent::DirectoryListing(listing) => {
                                            hub.shell_listing = Some(listing);
                                        }
                                        OscEvent::CommandResult { .. } => {}
                                        OscEvent::JsonContext(json) => {
                                            hub.json_context = Some(json);
                                        }
                                    }
                                }

                                if !clean_text.is_empty() {
                                    hub.terminal_output.push(TerminalLine {
                                        text: clean_text.to_string(),
                                        priority: osc_parser.current_priority,
                                        timestamp: Local::now(),
                                    });
                                    if hub.terminal_output.len() > hub.buffer_limit {
                                        hub.terminal_output.remove(0);
                                    }
                                    hub.version += 1;
                                    state_lock.version += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub enum OscEvent {
    Priority(u8),
    Cwd(String),
    DirectoryListing(crate::common::DirectoryListing),
    CommandResult {
        command: String,
        status: i32,
        output: Option<String>,
    },
    JsonContext(serde_json::Value),
}

pub struct OscParser {
    pub current_priority: u8,
}

impl OscParser {
    pub fn new() -> Self {
        Self { current_priority: 0 }
    }

    pub fn process(&mut self, input: &str) -> (String, Vec<OscEvent>) {
        let mut clean_text = input.to_string();
        let mut events = Vec::new();

        let priority_re = regex::Regex::new(r"\x1b\]9012;(\d)\x07").unwrap();
        for cap in priority_re.captures_iter(input) {
            if let Ok(p) = cap[1].parse::<u8>() {
                events.push(OscEvent::Priority(p));
            }
        }
        clean_text = priority_re.replace_all(&clean_text, "").to_string();

        let cwd_re = regex::Regex::new(r"\x1b\]9003;([^\x07]+)\x07").unwrap();
        for cap in cwd_re.captures_iter(input) {
            events.push(OscEvent::Cwd(cap[1].to_string()));
        }
        clean_text = cwd_re.replace_all(&clean_text, "").to_string();

        let result_re = regex::Regex::new(r"\x1b\]9002;([^;]+);(\d+)(?:;([^\x07]+))?\x07").unwrap();
        for cap in result_re.captures_iter(input) {
            let command = cap[1].to_string();
            let status = cap[2].parse::<i32>().unwrap_or(0);
            let output = cap.get(3).map(|m| {
                let decoded = base64::Engine::decode(&base64::prelude::BASE64_STANDARD, m.as_str()).unwrap_or_default();
                String::from_utf8_lossy(&decoded).to_string()
            });
            events.push(OscEvent::CommandResult { command, status, output });
        }
        clean_text = result_re.replace_all(&clean_text, "").to_string();

        let dl_re = regex::Regex::new(r"\x1b\]9001;([^;]+);([^\x07]+)\x07").unwrap();
        for cap in dl_re.captures_iter(input) {
            if let Ok(decoded) = base64::Engine::decode(&base64::prelude::BASE64_STANDARD, &cap[2]) {
                if let Ok(listing) = serde_json::from_slice::<crate::common::DirectoryListing>(&decoded) {
                    events.push(OscEvent::DirectoryListing(listing));
                }
            }
        }
        clean_text = dl_re.replace_all(&clean_text, "").to_string();

        let iterm_cwd_re = regex::Regex::new(r"\x1b\]1337;CurrentDir=([^\x07]+)\x07").unwrap();
        for cap in iterm_cwd_re.captures_iter(input) {
            events.push(OscEvent::Cwd(cap[1].to_string()));
        }
        clean_text = iterm_cwd_re.replace_all(&clean_text, "").to_string();

        let json_context_re = regex::Regex::new(r"\x1b\]9004;([^\x07]+)\x07").unwrap();
        for cap in json_context_re.captures_iter(input) {
            if let Ok(decoded) = base64::Engine::decode(&base64::prelude::BASE64_STANDARD, &cap[1]) {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                    events.push(OscEvent::JsonContext(json));
                }
            } else if let Ok(json) = serde_json::from_str::<serde_json::Value>(&cap[1]) {
                // Support both base64 and plain JSON
                events.push(OscEvent::JsonContext(json));
            }
        }
        clean_text = json_context_re.replace_all(&clean_text, "").to_string();

        (clean_text, events)
    }
}
