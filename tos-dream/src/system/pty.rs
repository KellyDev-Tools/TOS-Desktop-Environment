use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PtyEvent {
    Output(String),
    DirectoryChanged(String),
    ShellReady,
    ProcessExited(i32),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum PtyCommand {
    Write(String),
    WriteLine(String),
    Resize(u16, u16),
    Close,
}

pub struct PtyHandle {
    pub cmd_tx: Sender<PtyCommand>,
    pub event_rx: Receiver<PtyEvent>,
    pub child_pid: u32,
}

pub struct PtyParser;

impl PtyParser {
    pub fn parse_data(data: &str) -> Vec<PtyEvent> {
        let mut events = Vec::new();
        
        // Basic OSC 1337 extraction for CurrentDir
        if data.contains("\x1b]1337;CurrentDir=") {
            let parts: Vec<&str> = data.split("\x1b]1337;").collect();
            for part in parts {
                if part.starts_with("CurrentDir=") {
                    let rest = &part[11..];
                    if let Some(end) = rest.find('\x07').or(rest.find('\x1b')) {
                        let dir = rest[..end].to_string();
                        events.push(PtyEvent::DirectoryChanged(dir));
                    }
                }
            }
        }
        
        events.push(PtyEvent::Output(data.to_string()));
        events
    }
}

impl PtyHandle {
    pub fn spawn(shell: &str, cwd: &str) -> Option<Self> {
        Self::spawn_with_args(shell, &[], cwd)
    }

    pub fn spawn_with_args(shell: &str, args: &[&str], cwd: &str) -> Option<Self> {
        let (cmd_tx, cmd_rx) = channel::<PtyCommand>();
        let (event_tx, event_rx) = channel::<PtyEvent>();

        let mut winsize = libc::winsize {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let mut master_fd: libc::c_int = 0;

        let pid = unsafe {
            libc::forkpty(
                &mut master_fd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut winsize,
            )
        };

        if pid < 0 { return None; }

        if pid == 0 {
            // Child
            let _ = std::env::set_current_dir(cwd);
            std::env::set_var("TERM", "xterm-256color");
            std::env::set_var("TOS_DREAM", "1");

            let shell_path = std::ffi::CString::new(shell).unwrap();
            
            // Prepare arguments: [shell, ...args, NULL]
            let mut arg_cstrs = Vec::new();
            arg_cstrs.push(shell_path.clone());
            for arg in args {
                arg_cstrs.push(std::ffi::CString::new(*arg).unwrap());
            }
            
            let mut arg_ptrs: Vec<*const libc::c_char> = arg_cstrs.iter().map(|s| s.as_ptr()).collect();
            arg_ptrs.push(std::ptr::null());

            unsafe {
                libc::execv(shell_path.as_ptr(), arg_ptrs.as_ptr());
                libc::_exit(127);
            }
        }

        // Parent
        let child_pid = pid as u32;

        // Set non-blocking
        unsafe {
            let flags = libc::fcntl(master_fd, libc::F_GETFL);
            libc::fcntl(master_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        let reader_tx = event_tx.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                let n = unsafe {
                    libc::read(master_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
                };
                if n > 0 {
                    let data = String::from_utf8_lossy(&buf[..n as usize]).to_string();
                    let events = PtyParser::parse_data(&data);
                    for event in events {
                        let _ = reader_tx.send(event);
                    }
                } else if n == 0 {
                    let _ = reader_tx.send(PtyEvent::ProcessExited(0));
                    break;
                } else {
                    let errno = unsafe { *libc::__errno_location() };
                    if errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
                        thread::sleep(Duration::from_millis(10));
                    } else {
                        break;
                    }
                }
            }
        });

        thread::spawn(move || {
            loop {
                match cmd_rx.recv() {
                    Ok(PtyCommand::Write(s)) => {
                        let _ = unsafe { libc::write(master_fd, s.as_ptr() as *const libc::c_void, s.len()) };
                    }
                    Ok(PtyCommand::WriteLine(s)) => {
                        let line = format!("{}\n", s);
                        let _ = unsafe { libc::write(master_fd, line.as_ptr() as *const libc::c_void, line.len()) };
                    }
                    Ok(PtyCommand::Resize(cols, rows)) => {
                        let ws = libc::winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
                        unsafe { libc::ioctl(master_fd, libc::TIOCSWINSZ, &ws); }
                    }
                    Ok(PtyCommand::Close) | Err(_) => {
                        unsafe { libc::close(master_fd); }
                        break;
                    }
                }
            }
        });

        Some(Self { cmd_tx, event_rx, child_pid })
    }

    pub fn write(&self, s: &str) {
        let _ = self.cmd_tx.send(PtyCommand::Write(s.to_string()));
    }

    pub fn poll_all(
        state: Arc<Mutex<crate::TosState>>, 
        ptys: Arc<Mutex<HashMap<uuid::Uuid, PtyHandle>>>
    ) {
        thread::spawn(move || {
            loop {
                let mut ptys_lock = ptys.lock().unwrap();
                for (hub_id, pty) in ptys_lock.iter_mut() {
                    while let Ok(event) = pty.event_rx.try_recv() {
                        let mut state_lock = state.lock().unwrap();
                        
                        // Find the target hub
                        let (sector_idx, hub_idx) = if let Some(indices) = state_lock.sectors.iter().enumerate().find_map(|(s_idx, s)| {
                            s.hubs.iter().enumerate().find_map(|(h_idx, h)| {
                                if h.id == *hub_id { Some((s_idx, h_idx)) } else { None }
                            })
                        }) {
                            indices
                        } else {
                            continue;
                        };

                        match event {
                            PtyEvent::Output(data) => {
                                // Use ShellApi to process output and handle OSC sequences
                                let clean_output = state_lock.process_shell_output(&data);
                                
                                // Direct access to the hub to update terminal output after ShellApi processed it
                                let hub = &mut state_lock.sectors[sector_idx].hubs[hub_idx];
                                if !clean_output.is_empty() {
                                    hub.terminal_output.push(clean_output);
                                    if hub.terminal_output.len() > 100 {
                                        hub.terminal_output.remove(0);
                                    }
                                }
                            }
                            PtyEvent::DirectoryChanged(path) => {
                                // Manual override or legacy support
                                let hub = &mut state_lock.sectors[sector_idx].hubs[hub_idx];
                                tracing::debug!("Hub {} directory changed to: {}", hub.id, path);
                            }
                            PtyEvent::ProcessExited(code) => {
                                let hub = &mut state_lock.sectors[sector_idx].hubs[hub_idx];
                                hub.terminal_output.push(format!("\n[PROCESS EXITED WITH CODE {}]", code));
                            }
                            _ => {}
                        }
                    }
                }
                drop(ptys_lock);
                thread::sleep(Duration::from_millis(50));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_parser_osc() {
        let data = "normal text\x1b]1337;CurrentDir=/home/user\x07more text";
        let events = PtyParser::parse_data(data);
        
        assert!(events.iter().any(|e| matches!(e, PtyEvent::DirectoryChanged(ref d) if d == "/home/user")));
        assert!(events.iter().any(|e| matches!(e, PtyEvent::Output(ref o) if o == data)));
    }

    #[test]
    fn test_pty_parser_no_osc() {
        let data = "just some text";
        let events = PtyParser::parse_data(data);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], PtyEvent::Output(data.to_string()));
    }
}
