use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::time::Duration;
use std::os::unix::io::RawFd;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl PtyHandle {
    pub fn spawn(shell: &str, cwd: &str) -> Option<Self> {
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
            unsafe {
                libc::execl(shell_path.as_ptr(), shell_path.as_ptr(), std::ptr::null::<libc::c_char>());
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
                    // Basic OSC 1337 extraction for CurrentDir
                    if data.contains("\x1b]1337;CurrentDir=") {
                        if let Some(start) = data.find("CurrentDir=") {
                            let rest = &data[start + 11..];
                            if let Some(end) = rest.find('\x07').or(rest.find('\x1b')) {
                                let dir = rest[..end].to_string();
                                let _ = reader_tx.send(PtyEvent::DirectoryChanged(dir));
                            }
                        }
                    }
                    let _ = reader_tx.send(PtyEvent::Output(data));
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
}
