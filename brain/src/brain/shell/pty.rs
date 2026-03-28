use portable_pty::{native_pty_system, CommandBuilder, PtySize, Child, MasterPty};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::common::{TosState, TerminalLine};
use tos_common::shell::{OscEvent, OscParser};
use chrono::Local;

/// §15.2: PTY-backed Shell Implementation (Desktop Only)
pub struct PtyShell {
    _state: Arc<Mutex<TosState>>,
    _sector_id: uuid::Uuid,
    _hub_id: uuid::Uuid,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    _child: Box<dyn Child + Send + Sync>,
}

impl PtyShell {
    pub fn new(
        state: Arc<Mutex<TosState>>, 
        modules: Arc<crate::brain::module_manager::ModuleManager>, 
        ai: Arc<crate::services::AiService>, 
        heuristic: Arc<crate::services::HeuristicService>, 
        sector_id: uuid::Uuid, 
        hub_id: uuid::Uuid
    ) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

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
            if std::path::Path::new(&path).exists() {
                (path, args)
            } else {
                resolve_from_env()
            }
        } else {
            resolve_from_env()
        };

        let mut cmd = CommandBuilder::new(verified_shell);
        cmd.args(args);
        cmd.cwd(cwd);
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
            read_loop(reader, state_clone, ai_clone, heuristic_clone, sid_clone, hid_clone, handle);
        });

        Ok(Self {
            _state: state,
            _sector_id: sector_id,
            _hub_id: hub_id,
            writer,
            master: pair.master,
            _child: child,
        })
    }

    pub fn write(&mut self, data: &str) -> anyhow::Result<()> {
        self.writer.write_all(data.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&self, rows: u16, cols: u16) -> anyhow::Result<()> {
        self.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })?;
        Ok(())
    }

    pub fn send_signal(&mut self, signal: &str) -> anyhow::Result<()> {
        match signal {
            "INT" | "SIGINT" => self.write("\x03")?,
            "TERM" | "SIGTERM" => self.write("\x04")?,
            _ => return Err(anyhow::anyhow!("Unsupported signal: {}", signal)),
        }
        Ok(())
    }

    pub fn force_kill(&mut self) -> anyhow::Result<()> {
        self._child.kill()?;
        Ok(())
    }
}

fn read_loop(
    mut reader: Box<dyn Read + Send>, 
    state: Arc<Mutex<TosState>>, 
    _ai: Arc<crate::services::AiService>, 
    _heuristic: Arc<crate::services::HeuristicService>, 
    sector_id: uuid::Uuid, 
    hub_id: uuid::Uuid, 
    _handle: tokio::runtime::Handle
) {
    let mut osc_parser = OscParser::new();
    let mut line_buffer = String::new();
    let mut buffer = [0u8; 4096];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let data = &buffer[..n];
                let text = String::from_utf8_lossy(data);
                line_buffer.push_str(&text);

                while let Some(pos) = line_buffer.find('\n') {
                    let mut line = line_buffer.drain(..=pos).collect::<String>();
                    line = line.trim_end_matches(['\r', '\n']).to_string();
                    
                    let (clean_text, events) = osc_parser.process(&line);
                    
                    let mut state_lock = state.lock().unwrap();
                    for event in events {
                        match event {
                            OscEvent::Priority(p) => osc_parser.current_priority = p,
                            OscEvent::Cwd(path) => {
                                let path_buf = std::path::PathBuf::from(path);
                                if let Some(sector) = state_lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                                    if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
                                        hub.current_directory = path_buf;
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
                            OscEvent::CommandResult { command: _, status, output: _ } => {
                                if let Some(sector) = state_lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                                    if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
                                        hub.last_exit_status = Some(status);
                                        hub.is_running = false;
                                    }
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
