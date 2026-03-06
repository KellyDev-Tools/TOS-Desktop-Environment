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
    master: Box<dyn portable_pty::MasterPty + Send>,
    _child: Box<dyn Child + Send + Sync>,
    ai: Arc<crate::services::AiService>,
    heuristic: Arc<crate::services::HeuristicService>,
}

impl ShellApi {
    pub fn new(state: Arc<Mutex<TosState>>, modules: Arc<crate::brain::module_manager::ModuleManager>, ai: Arc<crate::services::AiService>, heuristic: Arc<crate::services::HeuristicService>, sector_id: uuid::Uuid, hub_id: uuid::Uuid) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Resolve shell and cwd from modules if possible
        let (shell_id, cwd) = {
            let lock = state.lock().unwrap();
            let hub_opt = lock.sectors.iter()
                .find(|s| s.id == sector_id)
                .and_then(|s| s.hubs.iter().find(|h| h.id == hub_id));
            
            let id = hub_opt.and_then(|h| h.shell_module.clone()).unwrap_or_else(|| "tos-shell-fish".to_string());
            let current_dir = hub_opt.map(|h| h.current_directory.clone()).unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/")));
            
            (id, current_dir)
        };

        let resolve_from_env = || -> (String, Vec<String>) {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
            let verified = if std::path::Path::new(&shell).exists() {
                shell
            } else {
                let fallbacks = ["/bin/bash", "/bin/zsh", "/bin/sh", "/usr/bin/bash", "/usr/bin/sh"];
                fallbacks.iter()
                    .find(|&&path| std::path::Path::new(path).exists())
                    .map(|&s| s.to_string())
                    .unwrap_or_else(|| "sh".to_string())
            };
            (verified, vec!["--login".to_string()])
        };

        let (verified_shell, args) = if let Ok(shell_mod) = modules.load_shell(&shell_id) {
            let path = shell_mod.get_executable_path().to_string_lossy().to_string();
            let args = shell_mod.get_default_args().to_vec();
            // Verify the module's executable actually exists on this system.
            if std::path::Path::new(&path).exists() {
                (path, args)
            } else {
                let msg = format!("Shell module '{}' resolved to '{}' which does not exist, falling back", shell_id, path);
                tracing::warn!("{}", msg);
                // Surface warning in the Face's System Output layer
                let mut lock = state.lock().unwrap();
                lock.system_log.push(TerminalLine {
                    text: msg,
                    priority: 2,
                    timestamp: Local::now(),
                });
                drop(lock);
                resolve_from_env()
            }
        } else {
            resolve_from_env()
        };

        tracing::info!("SHELL INIT: Using verified binary: {} with args: {:?}", verified_shell, args);
        let mut cmd = CommandBuilder::new(verified_shell);
        cmd.args(args);
        cmd.cwd(cwd); // Set the restored working directory
        let child = pair.slave.spawn_command(cmd)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let state_clone = state.clone();
        let ai_clone = ai.clone();
        let heuristic_clone = heuristic.clone();
        let sid_clone = sector_id;
        let hid_clone = hub_id;
        let handle = tokio::runtime::Handle::current();
        thread::spawn(move || {
            Self::read_loop(reader, state_clone, ai_clone, heuristic_clone, sid_clone, hid_clone, handle);
        });

        Ok(Self {
            _state: state,
            _sector_id: sector_id,
            _hub_id: hub_id,
            writer,
            master: pair.master,
            _child: child,
            ai,
            heuristic,
        })
    }

    pub fn write(&mut self, data: &str) -> anyhow::Result<()> {
        self.writer.write_all(data.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&self, rows: u16, cols: u16) -> anyhow::Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    pub fn send_signal(&mut self, signal: &str) -> anyhow::Result<()> {
        match signal {
            "INT" | "SIGINT" => {
                // Send Ctrl+C sequence to PTY
                self.write("\x03")?;
            }
            "TERM" | "SIGTERM" => {
                // Common to send Ctrl+D for EOF or similar, but SIGTERM normally kills.
                // For PTY, we might need OS-specific handles.
                self.write("\x04")?; // Ctrl+D (EOT)
            }
            _ => return Err(anyhow::anyhow!("Unsupported signal: {}", signal)),
        }
        Ok(())
    }

    fn read_loop(mut reader: Box<dyn Read + Send>, state: Arc<Mutex<TosState>>, ai: Arc<crate::services::AiService>, heuristic: Arc<crate::services::HeuristicService>, sector_id: uuid::Uuid, hub_id: uuid::Uuid, handle: tokio::runtime::Handle) {
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
                        // We no longer hold a mutable reference to `hub` across the entire loop iteration.
                        // Instead, we re-find `sector` and `hub` as needed within each event handler
                        // to avoid borrow conflicts when `state_lock` might be modified.
                        for event in events {
                            match event {
                                OscEvent::Priority(p) => osc_parser.current_priority = p,
                                OscEvent::Cwd(path) => {
                                    let path_buf = std::path::PathBuf::from(path);
                                    if let Some(sector) = state_lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                                        if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
                                            hub.current_directory = path_buf;
                                            
                                            // §10.3: Heuristic Sector Renaming
                                            if sector.name == "Primary" || sector.name == "unnamed" || sector.name.starts_with("Sector ") {
                                                if let Some(name) = hub.current_directory.file_name().and_then(|s| s.to_str()) {
                                                    if !name.is_empty() {
                                                        sector.name = name.to_string();
                                                    }
                                                } else if hub.current_directory.to_string_lossy() == "/" {
                                                    sector.name = "Root".to_string();
                                                }
                                            }
                                        }
                                    }
                                }
                                OscEvent::DirectoryListing(listing) => {
                                    if let Some(sector) = state_lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                                        if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
                                            hub.shell_listing = Some(listing);
                                        }
                                    }
                                }
                                OscEvent::CommandResult { command, status, output } => {
                                    if let Some(sector) = state_lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                                        if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
                                            hub.last_exit_status = Some(status);
                                            hub.is_running = false;
                                        }
                                    }

                                    if status != 0 {
                                        let ai_trigger = ai.clone();
                                        let cmd_trigger = command.clone();
                                        let out_trigger = output.clone();
                                        tokio::spawn(async move {
                                            let _ = ai_trigger.passive_observe(&cmd_trigger, status, out_trigger.as_deref()).await;
                                        });
                                    }

                                    // §145: Handle auto-collapse
                                    let dismiss_behavior = state_lock.settings.resolve("tos.interface.bezel.dismiss", None, None);
                                    if state_lock.bezel_expanded && dismiss_behavior == Some("auto".to_string()) {
                                        state_lock.bezel_expanded = false;
                                        state_lock.version += 1;
                                    }
                                
                                    // §10.2: Implicit Correction Trigger for 127
                                    if status == 127 {
                                        let heuristic_trigger = heuristic.clone();
                                        let state_trigger = state.clone();
                                        let cmd_trigger = command.clone();
                                        handle.spawn(async move {
                                            // Request typo correction from Heuristic Service
                                            let cwd = {
                                                let lock = state_trigger.lock().unwrap();
                                                if let Some(s) = lock.sectors.iter().find(|s| s.id == sector_id) {
                                                    if let Some(h) = s.hubs.iter().find(|h| h.id == hub_id) {
                                                        h.current_directory.to_string_lossy().to_string()
                                                    } else { ".".to_string() }
                                                } else { ".".to_string() }
                                            };
                                            if let Ok(resp) = heuristic_trigger.query(&cmd_trigger, &cwd).await {
                                                if let Ok(suggestions) = serde_json::from_str::<serde_json::Value>(resp.split(" (").next().unwrap_or(&resp)) {
                                                    if let Some(best) = suggestions.as_array().and_then(|a| a.first()) {
                                                        let fix = best["text"].as_str().unwrap_or("");
                                                        if !fix.is_empty() {
                                                            let mut lock = state_trigger.lock().unwrap();
                                                            if let Some(s) = lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                                                                if let Some(h) = s.hubs.iter_mut().find(|h| h.id == hub_id) {
                                                                    h.staged_command = Some(fix.to_string());
                                                                    h.ai_explanation = Some(format!("✦ TYPO? Suggested: {}", fix));
                                                                    lock.version += 1;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    }
                                }
                                OscEvent::JsonContext(json) => {
                                    if let Some(sector) = state_lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                                        if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
                                            hub.json_context = Some(json);
                                        }
                                    }
                                }
                            }
                        }

                        // This part still needs to find the hub to append output
                        if !clean_text.is_empty() {
                            if let Some(sector) = state_lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                                if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
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

        // §1.7: Support Universal OSC 7 (CurrentDir)
        let osc7_re = regex::Regex::new(r"\x1b\]7;file://[a-zA-Z0-9.\-]*([^\x07\x1b]+)(?:\x07|\x1b\\)").unwrap();
        for cap in osc7_re.captures_iter(input) {
            let path = cap[1].to_string();
            // URL decode path if necessary (simple for now)
            let path = path.replace("%20", " ");
            events.push(OscEvent::Cwd(path));
        }
        clean_text = osc7_re.replace_all(&clean_text, "").to_string();

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

        let iterm_cwd_generic_re = regex::Regex::new(r"\x1b\]1337;RemoteHost=[^;]+;CurrentDir=([^\x07]+)\x07").unwrap();
        for cap in iterm_cwd_generic_re.captures_iter(input) {
             events.push(OscEvent::Cwd(cap[1].to_string()));
        }
        clean_text = iterm_cwd_generic_re.replace_all(&clean_text, "").to_string();

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

        // §29.1: Strip remaining CSI sequences (Colors, Cursor control, etc)
        // Match ESC [ followed by any number of parameter bytes (0x30–0x3F),
        // any number of intermediate bytes (0x20–0x2F), then a final byte (0x40–0x7E).
        let csi_re = regex::Regex::new(r"\x1b\[[0-9;?]*[A-Za-z]").unwrap();
        clean_text = csi_re.replace_all(&clean_text, "").to_string();

        // Strip any remaining unhandled OSC sequences (ESC ] ... BEL/ST)
        let misc_osc_re = regex::Regex::new(r"\x1b\].*?(\x07|\x1b\\)").unwrap();
        clean_text = misc_osc_re.replace_all(&clean_text, "").to_string();

        (clean_text, events)
    }
}
