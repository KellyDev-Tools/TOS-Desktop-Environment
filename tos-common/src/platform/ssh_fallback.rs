use std::sync::{Arc, Mutex};
use uuid::Uuid;
use portable_pty::{native_pty_system, CommandBuilder, PtySize, MasterPty};
use std::io::{Read, Write};
use std::thread;
use crate::{TosState, TerminalLine};

/// §27.3: SSH Fallback Session for non-TOS remotes.
///
/// This provides an interactive bridge between the TOS Command Hub and a
/// standard SSH process, allowing the Brain to control remote legacy servers.
pub struct SshSession {
    pub id: Uuid,
    pub host: String,
    writer: Box<dyn Write + Send>,
    _master: Box<dyn MasterPty + Send>,
}

impl SshSession {
    /// Establish an interactive SSH connection and bridge it to a Hub's terminal output.
    pub fn connect(
        host: &str,
        state: Arc<Mutex<TosState>>,
        sector_id: Uuid,
        hub_id: Uuid,
    ) -> anyhow::Result<Self> {
        tracing::info!("Starting interactive SSH session for host: {}", host);

        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new("ssh");
        // -t forces TTY allocation; BatchMode=no allows password prompts if needed (Face will see them)
        cmd.args(["-t", host]);

        let _child = pair.slave.spawn_command(cmd)?;
        
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        // Spawn read loop to pipe SSH output to TosState
        let state_clone = state.clone();
        thread::spawn(move || {
            read_loop(reader, state_clone, sector_id, hub_id);
        });

        Ok(Self {
            id: Uuid::new_v4(),
            host: host.to_string(),
            writer,
            _master: pair.master,
        })
    }

    /// Send input strings to the remote SSH shell.
    pub fn write(&mut self, data: &str) -> anyhow::Result<()> {
        self.writer.write_all(data.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }
}

fn read_loop(
    mut reader: Box<dyn Read + Send>,
    state: Arc<Mutex<TosState>>,
    sector_id: Uuid,
    hub_id: Uuid,
) {
    let mut buffer = [0u8; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) | Err(_) => {
                tracing::info!("SSH session for {}/{} closed.", sector_id, hub_id);
                break;
            },
            Ok(n) => {
                let text = String::from_utf8_lossy(&buffer[..n]).to_string();
                let mut lock = state.lock().unwrap();
                if let Some(sector) = lock.sectors.iter_mut().find(|s| s.id == sector_id) {
                    if let Some(hub) = sector.hubs.iter_mut().find(|h| h.id == hub_id) {
                        hub.terminal_output.push(TerminalLine {
                            text,
                            priority: 1,
                            timestamp: chrono::Local::now(),
                        });
                        if hub.terminal_output.len() > hub.buffer_limit {
                            hub.terminal_output.remove(0);
                        }
                        hub.version += 1;
                        lock.version += 1;
                    }
                }
            }
        }
    }
}
